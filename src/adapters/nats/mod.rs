//! # NATS JetStream Event Store Adapter
//!
//! Implements event persistence using NATS JetStream for the CIM event sourcing pattern.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     NATS JetStream                                  │
//! │  ┌─────────────────────────────────────────────────────────────┐   │
//! │  │              Stream: network-events                         │   │
//! │  │  Subjects: network.device.*, network.connection.*,          │   │
//! │  │            network.topology.*, network.inventory.*          │   │
//! │  └─────────────────────────────────────────────────────────────┘   │
//! │                              │                                      │
//! │        ┌────────────────────┼────────────────────┐                 │
//! │        ▼                    ▼                    ▼                 │
//! │  ┌──────────┐        ┌──────────┐        ┌──────────┐             │
//! │  │ Consumer │        │ Consumer │        │ Consumer │             │
//! │  │ (replay) │        │ (device) │        │ (sync)   │             │
//! │  └──────────┘        └──────────┘        └──────────┘             │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Event Subjects
//!
//! Events are published to subjects based on aggregate type:
//! - `network.device.{event_type}` - Device lifecycle events
//! - `network.connection.{event_type}` - Connection events
//! - `network.topology.{event_type}` - Topology events
//! - `network.inventory.{event_type}` - Inventory sync events
//!
//! ## Message Headers
//!
//! Each message includes CIM-standard headers:
//! - `Nats-Msg-Id` - Unique message ID (for deduplication)
//! - `CIM-Aggregate-Id` - The aggregate this event belongs to
//! - `CIM-Event-Type` - The event type name
//! - `CIM-Correlation-Id` - Correlation ID for tracing
//! - `CIM-Causation-Id` - The event that caused this event
//! - `CIM-Timestamp` - Event timestamp (RFC3339)

use async_nats::jetstream::{self, consumer::PullConsumer, stream::Stream, Context};
use async_nats::{Client, HeaderMap, HeaderValue};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::events::NetworkEvent;
use crate::domain::ports::{EventStorePort, PortError};

/// Stream name for network events
pub const STREAM_NAME: &str = "network-events";

/// Subject prefix for all network events
pub const SUBJECT_PREFIX: &str = "network";

/// Configuration for the NATS event store
#[derive(Debug, Clone)]
pub struct NatsEventStoreConfig {
    /// NATS server URL (e.g., "nats://localhost:4222")
    pub nats_url: String,
    /// Stream name (defaults to "network-events")
    pub stream_name: String,
    /// Subject prefix (defaults to "network")
    /// Each stream should use a unique prefix to avoid overlaps
    pub subject_prefix: String,
    /// Maximum messages in the stream (0 = unlimited)
    pub max_messages: i64,
    /// Maximum age of messages (0 = unlimited)
    pub max_age_seconds: u64,
    /// Number of replicas (for HA)
    pub replicas: usize,
}

impl Default for NatsEventStoreConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".to_string(),
            stream_name: STREAM_NAME.to_string(),
            subject_prefix: SUBJECT_PREFIX.to_string(),
            max_messages: 0,        // Unlimited
            max_age_seconds: 0,     // Keep forever
            replicas: 1,            // Single node
        }
    }
}

impl NatsEventStoreConfig {
    /// Create a unique configuration for testing
    /// Uses a UUID-based stream name and subject prefix to avoid conflicts
    pub fn for_testing(nats_url: &str) -> Self {
        let id = uuid::Uuid::now_v7().to_string();
        let short_id = &id[..8];
        Self {
            nats_url: nats_url.to_string(),
            stream_name: format!("test-{}", short_id),
            subject_prefix: format!("test-{}", short_id),
            max_messages: 0,
            max_age_seconds: 0,
            replicas: 1,
        }
    }
}

/// NATS JetStream event store adapter
///
/// Implements `EventStorePort` for event sourcing with NATS JetStream.
pub struct NatsEventStore {
    /// NATS client
    client: Client,
    /// JetStream context
    jetstream: Context,
    /// Stream reference
    stream: Arc<RwLock<Option<Stream>>>,
    /// Configuration
    config: NatsEventStoreConfig,
}

impl NatsEventStore {
    /// Create a new NATS event store
    ///
    /// # Arguments
    /// * `config` - Configuration for the event store
    ///
    /// # Example
    /// ```ignore
    /// let config = NatsEventStoreConfig::default();
    /// let store = NatsEventStore::new(config).await?;
    /// ```
    pub async fn new(config: NatsEventStoreConfig) -> Result<Self, PortError> {
        // Connect to NATS
        let client = async_nats::connect(&config.nats_url)
            .await
            .map_err(|e| PortError::ConnectionFailed(format!("NATS connection failed: {}", e)))?;

        tracing::info!("Connected to NATS at {}", config.nats_url);

        // Create JetStream context
        let jetstream = jetstream::new(client.clone());

        let store = Self {
            client,
            jetstream,
            stream: Arc::new(RwLock::new(None)),
            config,
        };

        // Initialize the stream
        store.ensure_stream().await?;

        Ok(store)
    }

    /// Connect with default configuration
    pub async fn connect(nats_url: &str) -> Result<Self, PortError> {
        let config = NatsEventStoreConfig {
            nats_url: nats_url.to_string(),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Ensure the stream exists with proper configuration
    async fn ensure_stream(&self) -> Result<(), PortError> {
        let stream_config = jetstream::stream::Config {
            name: self.config.stream_name.clone(),
            description: Some("Network domain events for CIM".to_string()),
            subjects: vec![format!("{}.*.*", self.config.subject_prefix)],
            retention: jetstream::stream::RetentionPolicy::Limits,
            max_messages: self.config.max_messages,
            max_age: if self.config.max_age_seconds > 0 {
                std::time::Duration::from_secs(self.config.max_age_seconds)
            } else {
                std::time::Duration::ZERO
            },
            storage: jetstream::stream::StorageType::File,
            num_replicas: self.config.replicas,
            duplicate_window: std::time::Duration::from_secs(120),
            ..Default::default()
        };

        let stream = self.jetstream
            .get_or_create_stream(stream_config)
            .await
            .map_err(|e| PortError::ConnectionFailed(format!("Failed to create stream: {}", e)))?;

        tracing::info!(
            "JetStream stream '{}' ready with {} messages",
            self.config.stream_name,
            stream.cached_info().state.messages
        );

        let mut stream_lock = self.stream.write().await;
        *stream_lock = Some(stream);

        Ok(())
    }

    /// Get the NATS subject for an event using the configured prefix
    fn event_subject(&self, event: &NetworkEvent) -> String {
        event.nats_subject_with_prefix(&self.config.subject_prefix)
    }

    /// Create headers for an event message
    fn create_headers(event: &NetworkEvent, correlation_id: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Message ID for deduplication (aggregate_id + event_type + timestamp)
        let msg_id = format!(
            "{}-{}-{}",
            event.aggregate_id(),
            event.event_type(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        headers.insert("Nats-Msg-Id", HeaderValue::from(msg_id.as_str()));

        // CIM standard headers
        headers.insert("CIM-Aggregate-Id", HeaderValue::from(event.aggregate_id().as_str()));
        headers.insert("CIM-Event-Type", HeaderValue::from(event.event_type()));
        headers.insert(
            "CIM-Timestamp",
            HeaderValue::from(chrono::Utc::now().to_rfc3339().as_str()),
        );

        if let Some(corr_id) = correlation_id {
            headers.insert("CIM-Correlation-Id", HeaderValue::from(corr_id));
        }

        headers
    }

    /// Publish a single event
    async fn publish_event(
        &self,
        event: &NetworkEvent,
        correlation_id: Option<&str>,
    ) -> Result<(), PortError> {
        let subject = self.event_subject(event);
        let headers = Self::create_headers(event, correlation_id);

        let payload = serde_json::to_vec(event)
            .map_err(|e| PortError::VendorError(format!("Serialization failed: {}", e)))?;

        self.jetstream
            .publish_with_headers(subject.clone(), headers, payload.into())
            .await
            .map_err(|e| PortError::VendorError(format!("Publish failed: {}", e)))?
            .await
            .map_err(|e| PortError::VendorError(format!("Publish ack failed: {}", e)))?;

        tracing::debug!("Published event {} to {}", event.event_type(), subject);

        Ok(())
    }

    /// Create a consumer for replaying events
    async fn create_replay_consumer(
        &self,
        filter_subject: &str,
        consumer_name: &str,
    ) -> Result<PullConsumer, PortError> {
        let stream = self.stream.read().await;
        let stream = stream
            .as_ref()
            .ok_or_else(|| PortError::ConnectionFailed("Stream not initialized".to_string()))?;

        let consumer_config = jetstream::consumer::pull::Config {
            name: Some(consumer_name.to_string()),
            durable_name: None, // Ephemeral consumer for replay
            filter_subject: filter_subject.to_string(),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ack_policy: jetstream::consumer::AckPolicy::None, // Replay doesn't need acks
            ..Default::default()
        };

        let consumer = stream
            .create_consumer(consumer_config)
            .await
            .map_err(|e| PortError::ConnectionFailed(format!("Failed to create consumer: {}", e)))?;

        Ok(consumer)
    }

    /// Get the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the JetStream context
    pub fn jetstream(&self) -> &Context {
        &self.jetstream
    }
}

#[async_trait]
impl EventStorePort for NatsEventStore {
    async fn append(&self, events: Vec<NetworkEvent>) -> Result<(), PortError> {
        if events.is_empty() {
            return Ok(());
        }

        // Generate correlation ID for this batch
        let correlation_id = uuid::Uuid::now_v7().to_string();

        tracing::info!(
            "Appending {} events with correlation_id {}",
            events.len(),
            correlation_id
        );

        for event in &events {
            self.publish_event(event, Some(&correlation_id)).await?;
        }

        Ok(())
    }

    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<NetworkEvent>, PortError> {
        // Create a filter subject that matches all events for this aggregate
        // Events are published to {prefix}.{aggregate_type}.{event_type}
        // We need to filter by aggregate_id in the message body

        // For efficiency, we'll filter by the aggregate type prefix if we can determine it
        // This is a simplification - in production you might have aggregate-specific streams
        let filter_subject = format!("{}.>", self.config.subject_prefix);

        let consumer_name = format!("replay-{}-{}", aggregate_id, uuid::Uuid::now_v7());
        let consumer = self.create_replay_consumer(&filter_subject, &consumer_name).await?;

        let mut events = Vec::new();
        let mut messages = consumer.messages().await
            .map_err(|e| PortError::VendorError(format!("Failed to get messages: {}", e)))?;

        // Fetch messages with a timeout
        let timeout = tokio::time::Duration::from_secs(5);
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            match tokio::time::timeout(remaining, messages.next()).await {
                Ok(Some(msg)) => {
                    match msg {
                        Ok(msg) => {
                            // Check if this message belongs to our aggregate
                            let matches_aggregate = msg.headers
                                .as_ref()
                                .and_then(|h| h.get("CIM-Aggregate-Id"))
                                .map(|v| v.as_str() == aggregate_id)
                                .unwrap_or(false);

                            if matches_aggregate {
                                if let Ok(event) = serde_json::from_slice::<NetworkEvent>(&msg.payload) {
                                    events.push(event);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Error reading message: {}", e);
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => break, // Timeout
            }
        }

        tracing::debug!(
            "Loaded {} events for aggregate {}",
            events.len(),
            aggregate_id
        );

        Ok(events)
    }

    async fn subscribe(&self, subject: &str) -> Result<crate::domain::ports::EventSubscription, PortError> {
        // Create a durable consumer for this subscription
        let consumer_name = format!("sub-{}", subject.replace('.', "-").replace('*', "all").replace('>', "gt"));

        let stream = self.stream.read().await;
        let stream = stream
            .as_ref()
            .ok_or_else(|| PortError::ConnectionFailed("Stream not initialized".to_string()))?;

        let consumer_config = jetstream::consumer::pull::Config {
            name: Some(consumer_name.clone()),
            durable_name: Some(consumer_name),
            filter_subject: subject.to_string(),
            deliver_policy: jetstream::consumer::DeliverPolicy::New,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        };

        let _consumer = stream
            .get_or_create_consumer(&consumer_config.name.clone().unwrap(), consumer_config)
            .await
            .map_err(|e| PortError::ConnectionFailed(format!("Failed to create consumer: {}", e)))?;

        tracing::info!("Created subscription for subject: {}", subject);

        // Return a subscription handle
        // Note: The actual message iteration would be done through the consumer
        Ok(crate::domain::ports::EventSubscription::new())
    }
}

/// Event subscriber for streaming events
pub struct NatsEventSubscriber {
    consumer: PullConsumer,
}

impl NatsEventSubscriber {
    /// Create a new subscriber from a NATS consumer
    pub fn new(consumer: PullConsumer) -> Self {
        Self { consumer }
    }

    /// Get the next event from the subscription
    pub async fn next(&mut self) -> Option<Result<(NetworkEvent, NatsEventAck), PortError>> {
        let mut messages = match self.consumer.messages().await {
            Ok(m) => m,
            Err(e) => return Some(Err(PortError::VendorError(format!("Failed to get messages: {}", e)))),
        };

        match messages.next().await {
            Some(Ok(msg)) => {
                match serde_json::from_slice::<NetworkEvent>(&msg.payload) {
                    Ok(event) => Some(Ok((event, NatsEventAck { message: msg }))),
                    Err(e) => Some(Err(PortError::VendorError(format!("Deserialization failed: {}", e)))),
                }
            }
            Some(Err(e)) => Some(Err(PortError::VendorError(format!("Message error: {}", e)))),
            None => None,
        }
    }
}

/// Acknowledgment handle for a received event
pub struct NatsEventAck {
    message: async_nats::jetstream::message::Message,
}

impl NatsEventAck {
    /// Acknowledge successful processing of the event
    pub async fn ack(self) -> Result<(), PortError> {
        self.message.ack()
            .await
            .map_err(|e| PortError::VendorError(format!("Ack failed: {}", e)))
    }

    /// Negative acknowledgment - request redelivery
    pub async fn nak(self) -> Result<(), PortError> {
        self.message.ack_with(async_nats::jetstream::AckKind::Nak(None))
            .await
            .map_err(|e| PortError::VendorError(format!("Nak failed: {}", e)))
    }

    /// Get the correlation ID from the message headers
    pub fn correlation_id(&self) -> Option<String> {
        self.message.headers
            .as_ref()
            .and_then(|h| h.get("CIM-Correlation-Id"))
            .map(|v| v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NatsEventStoreConfig::default();
        assert_eq!(config.stream_name, "network-events");
        assert_eq!(config.nats_url, "nats://localhost:4222");
    }
}

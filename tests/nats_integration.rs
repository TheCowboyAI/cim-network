//! Integration tests for NATS JetStream EventStore
//!
//! These tests require a running NATS server with JetStream enabled.
//! Set the NATS_URL environment variable to point to your NATS cluster.
//!
//! Default: nats://localhost:4222
//!
//! Run with: NATS_URL=nats://apache_nats:4222 cargo test --test nats_integration

use cim_network::adapters::nats::{NatsEventStore, NatsEventStoreConfig};
use cim_network::domain::events::NetworkEvent;
use cim_network::domain::ports::EventStorePort;
use cim_network::domain::value_objects::{DeviceId, DeviceType, MacAddress};

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cim_network=debug".parse().unwrap())
                .add_directive("nats_integration=debug".parse().unwrap()),
        )
        .try_init();
}

fn get_nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

/// Test basic connectivity to the NATS cluster
#[tokio::test]
async fn test_nats_connection() {
    init_tracing();
    let nats_url = get_nats_url();
    tracing::info!("Testing connection to {}", nats_url);

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    tracing::info!("Using stream: {} with prefix: {}", config.stream_name, config.subject_prefix);

    let store = NatsEventStore::new(config).await;
    assert!(store.is_ok(), "Failed to connect to NATS at {}: {:?}", nats_url, store.err());

    tracing::info!("Successfully connected to NATS cluster");
}

/// Test appending a single event
#[tokio::test]
async fn test_append_single_event() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    // Create a device discovered event
    let device_id = DeviceId::new();
    let mac = MacAddress::parse("00:11:22:33:44:55").unwrap();
    let event = NetworkEvent::DeviceDiscovered {
        device_id,
        mac,
        device_type: DeviceType::Switch,
        ip_address: Some("192.168.1.100".parse().unwrap()),
    };

    // Append the event
    let result = store.append(vec![event.clone()]).await;
    assert!(result.is_ok(), "Failed to append event: {:?}", result.err());

    tracing::info!("Successfully appended event for device {}", device_id);
}

/// Test appending multiple events in a batch
#[tokio::test]
async fn test_append_batch_events() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    let device_id = DeviceId::new();
    let mac = MacAddress::parse("AA:BB:CC:DD:EE:FF").unwrap();

    // Create a sequence of events representing device lifecycle
    let events = vec![
        NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::AccessPoint,
            ip_address: Some("192.168.1.50".parse().unwrap()),
        },
        NetworkEvent::DeviceAdopting {
            device_id,
            vendor_id: mac.to_string(),
        },
        NetworkEvent::DeviceProvisioned {
            device_id,
            model: "U6-Pro".to_string(),
            firmware_version: "6.5.28".to_string(),
        },
    ];

    // Append all events
    let result = store.append(events).await;
    assert!(result.is_ok(), "Failed to append batch: {:?}", result.err());

    tracing::info!("Successfully appended batch of 3 events for device {}", device_id);
}

/// Test loading events for a specific aggregate
#[tokio::test]
async fn test_load_events() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    let device_id = DeviceId::new();
    let mac = MacAddress::parse("11:22:33:44:55:66").unwrap();

    // Create and append events
    let events = vec![
        NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::Gateway,
            ip_address: Some("10.0.0.1".parse().unwrap()),
        },
        NetworkEvent::DeviceRenamed {
            device_id,
            old_name: format!("Device-{}", &device_id.to_string()[..8]),
            new_name: "Main-Gateway".to_string(),
        },
    ];

    store.append(events.clone()).await
        .expect("Failed to append events");

    // Small delay for JetStream to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Load events for this aggregate
    let loaded = store.load_events(&device_id.to_string()).await
        .expect("Failed to load events");

    tracing::info!("Loaded {} events for device {}", loaded.len(), device_id);
    assert!(!loaded.is_empty(), "Expected to load events but got none");
    assert_eq!(loaded.len(), events.len(), "Event count mismatch");

    // Verify event content
    match &loaded[0] {
        NetworkEvent::DeviceDiscovered { device_id: id, mac: m, .. } => {
            assert_eq!(*id, device_id);
            assert_eq!(*m, mac);
        }
        _ => panic!("Expected DeviceDiscovered event"),
    }
}

/// Test full device lifecycle through event sourcing
#[tokio::test]
async fn test_device_lifecycle() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    // Simulate full device lifecycle
    let device_id = DeviceId::new();
    let mac = MacAddress::parse("DE:AD:BE:EF:CA:FE").unwrap();

    // Phase 1: Discovery
    let discovery_event = NetworkEvent::DeviceDiscovered {
        device_id,
        mac,
        device_type: DeviceType::Switch,
        ip_address: Some("172.16.0.10".parse().unwrap()),
    };
    store.append(vec![discovery_event]).await.expect("Discovery failed");
    tracing::info!("Phase 1: Device discovered");

    // Phase 2: Adoption
    let adoption_event = NetworkEvent::DeviceAdopting {
        device_id,
        vendor_id: mac.to_string(),
    };
    store.append(vec![adoption_event]).await.expect("Adoption failed");
    tracing::info!("Phase 2: Device adopting");

    // Phase 3: Provisioning
    let provision_event = NetworkEvent::DeviceProvisioned {
        device_id,
        model: "USW-24-POE".to_string(),
        firmware_version: "6.6.65".to_string(),
    };
    store.append(vec![provision_event]).await.expect("Provisioning failed");
    tracing::info!("Phase 3: Device provisioned");

    // Phase 4: Configuration
    let config_event = NetworkEvent::DeviceConfigured {
        device_id,
        interfaces: vec![],
        vlans: vec![],
    };
    store.append(vec![config_event]).await.expect("Configuration failed");
    tracing::info!("Phase 4: Device configured");

    // Wait for persistence
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify full event history
    let history = store.load_events(&device_id.to_string()).await
        .expect("Failed to load history");

    assert_eq!(history.len(), 4, "Expected 4 lifecycle events");
    tracing::info!("Device lifecycle test passed with {} events", history.len());
}

/// Test event replay for aggregate reconstruction
#[tokio::test]
async fn test_aggregate_reconstruction() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    // Create and persist events
    let device_id = DeviceId::new();
    let mac = MacAddress::parse("CA:FE:BA:BE:00:01").unwrap();

    let events = vec![
        NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::AccessPoint,
            ip_address: Some("192.168.10.50".parse().unwrap()),
        },
        NetworkEvent::DeviceAdopting {
            device_id,
            vendor_id: mac.to_string(),
        },
        NetworkEvent::DeviceProvisioned {
            device_id,
            model: "UAP-AC-Pro".to_string(),
            firmware_version: "6.2.49".to_string(),
        },
        NetworkEvent::DeviceRenamed {
            device_id,
            old_name: format!("Device-{}", &device_id.to_string()[..8]),
            new_name: "Office-AP-1".to_string(),
        },
    ];

    store.append(events).await.expect("Failed to persist events");

    // Wait for JetStream
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Reload events and reconstruct aggregate
    let loaded_events = store.load_events(&device_id.to_string()).await
        .expect("Failed to load events");

    // Use the aggregate's from_events to reconstruct
    use cim_network::domain::aggregates::NetworkDeviceAggregate;
    let aggregate = NetworkDeviceAggregate::from_events(loaded_events);

    assert!(aggregate.is_some(), "Failed to reconstruct aggregate");
    let aggregate = aggregate.unwrap();

    // Verify reconstructed state
    assert_eq!(aggregate.id(), device_id);
    assert_eq!(aggregate.mac(), mac);
    assert_eq!(aggregate.name(), "Office-AP-1");
    assert_eq!(aggregate.state(), cim_network::domain::aggregates::DeviceState::Provisioned);

    tracing::info!(
        "Aggregate reconstructed: {} ({}) in state {:?}",
        aggregate.name(),
        aggregate.id(),
        aggregate.state()
    );
}

/// Test concurrent event appends
#[tokio::test]
async fn test_concurrent_appends() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = std::sync::Arc::new(
        NatsEventStore::new(config).await
            .expect("Failed to connect to NATS")
    );

    // Spawn multiple concurrent tasks
    let mut handles = Vec::new();
    for i in 0..5 {
        let store = store.clone();
        handles.push(tokio::spawn(async move {
            let device_id = DeviceId::new();
            let mac_str = format!("{:02X}:00:00:00:00:{:02X}", i, i);
            let mac = MacAddress::parse(&mac_str).unwrap();

            let event = NetworkEvent::DeviceDiscovered {
                device_id,
                mac,
                device_type: DeviceType::Switch,
                ip_address: Some(format!("10.0.{}.1", i).parse().unwrap()),
            };

            store.append(vec![event]).await
        }));
    }

    // Wait for all tasks
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Verify all succeeded
    for (i, result) in results.into_iter().enumerate() {
        let inner = result.expect("Task panicked");
        assert!(inner.is_ok(), "Task {} failed: {:?}", i, inner.err());
    }

    tracing::info!("All 5 concurrent appends succeeded");
}

/// Test subscription creation
#[tokio::test]
async fn test_subscription() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let prefix = config.subject_prefix.clone();
    let store = NatsEventStore::new(config).await
        .expect("Failed to connect to NATS");

    // Create subscription for device events using the store's prefix
    let subscription = store.subscribe(&format!("{}.device.*", prefix)).await;
    assert!(subscription.is_ok(), "Failed to create subscription: {:?}", subscription.err());

    let sub = subscription.unwrap();
    tracing::info!("Created subscription with ID: {}", sub.id());
}

/// Test with the service layer
#[tokio::test]
async fn test_service_integration() {
    init_tracing();
    let nats_url = get_nats_url();

    let config = NatsEventStoreConfig::for_testing(&nats_url);
    let store = std::sync::Arc::new(
        NatsEventStore::new(config).await
            .expect("Failed to connect to NATS")
    );

    // Create a mock vendor adapter for testing
    use async_trait::async_trait;
    use cim_network::domain::ports::{DeviceControlPort, DeviceStats, PortError, VendorConfig, VendorDevice};

    struct MockVendorAdapter;

    #[async_trait]
    impl DeviceControlPort for MockVendorAdapter {
        fn vendor_name(&self) -> &str { "MockVendor" }
        async fn connect(&self) -> Result<(), PortError> { Ok(()) }
        async fn disconnect(&self) -> Result<(), PortError> { Ok(()) }
        fn is_connected(&self) -> bool { true }
        async fn list_devices(&self) -> Result<Vec<VendorDevice>, PortError> { Ok(vec![]) }
        async fn get_device(&self, _vendor_id: &str) -> Result<VendorDevice, PortError> {
            Err(PortError::DeviceNotFound(DeviceId::new()))
        }
        async fn adopt_device(&self, _vendor_id: &str) -> Result<(), PortError> { Ok(()) }
        async fn apply_config(&self, _vendor_id: &str, _config: VendorConfig) -> Result<(), PortError> { Ok(()) }
        async fn restart_device(&self, _vendor_id: &str) -> Result<(), PortError> { Ok(()) }
        async fn get_device_stats(&self, _vendor_id: &str) -> Result<DeviceStats, PortError> {
            Ok(DeviceStats {
                uptime_seconds: 3600,
                cpu_percent: Some(25.0),
                memory_percent: Some(50.0),
                temperature_celsius: Some(45.0),
                port_stats: vec![],
            })
        }
    }

    // Build the service
    use cim_network::service::NetworkService;

    let service = NetworkService::builder()
        .event_store_arc(store.clone() as std::sync::Arc<dyn EventStorePort>)
        .vendor_adapter(MockVendorAdapter)
        .build()
        .expect("Failed to build service");

    // List devices (should be empty initially)
    let devices = service.list_devices().await;
    assert!(devices.is_empty(), "Expected no devices initially");

    tracing::info!("Service integration test passed");
}

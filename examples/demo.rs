//! # CIM Network Demo
//!
//! Demonstrates the complete device lifecycle workflow:
//! 1. Connect to NATS for event persistence
//! 2. Create mock vendor adapter (simulates UniFi)
//! 3. Discover devices
//! 4. Adopt and provision devices
//! 5. Replay events to reconstruct state
//!
//! Run with: cargo run --example demo

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use cim_network::{
    adapters::nats::{NatsEventStore, NatsEventStoreConfig},
    domain::{
        aggregates::DeviceState,
        ports::{
            DeviceControlPort, DeviceStats, EventStorePort, PortError,
            VendorConfig, VendorDevice,
        },
        value_objects::{DeviceId, MacAddress},
    },
    service::NetworkService,
};

/// Mock vendor adapter that simulates UniFi controller responses
struct MockUniFiAdapter {
    /// Simulated devices "discovered" by the controller
    devices: Vec<VendorDevice>,
}

impl MockUniFiAdapter {
    fn new() -> Self {
        // Create some simulated devices
        let devices = vec![
            VendorDevice {
                vendor_id: "device-001".to_string(),
                device_id: None,
                mac: MacAddress::parse("00:11:22:33:44:55").unwrap(),
                model: "USW-24-POE".to_string(),
                name: "Core-Switch-1".to_string(),
                ip_address: Some("192.168.1.10".parse().unwrap()),
                adopted: false,
                properties: HashMap::new(),
            },
            VendorDevice {
                vendor_id: "device-002".to_string(),
                device_id: None,
                mac: MacAddress::parse("AA:BB:CC:DD:EE:FF").unwrap(),
                model: "U6-Pro".to_string(),
                name: "Office-AP-1".to_string(),
                ip_address: Some("192.168.1.50".parse().unwrap()),
                adopted: false,
                properties: HashMap::new(),
            },
            VendorDevice {
                vendor_id: "device-003".to_string(),
                device_id: None,
                mac: MacAddress::parse("DE:AD:BE:EF:CA:FE").unwrap(),
                model: "UDM-Pro".to_string(),
                name: "Main-Gateway".to_string(),
                ip_address: Some("192.168.1.1".parse().unwrap()),
                adopted: false,
                properties: HashMap::new(),
            },
        ];
        Self { devices }
    }
}

#[async_trait]
impl DeviceControlPort for MockUniFiAdapter {
    fn vendor_name(&self) -> &str {
        "MockUniFi"
    }

    async fn connect(&self) -> Result<(), PortError> {
        println!("  → Connected to MockUniFi controller");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), PortError> {
        println!("  → Disconnected from MockUniFi controller");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }

    async fn list_devices(&self) -> Result<Vec<VendorDevice>, PortError> {
        println!("  → Listing {} devices from controller", self.devices.len());
        Ok(self.devices.clone())
    }

    async fn get_device(&self, vendor_id: &str) -> Result<VendorDevice, PortError> {
        self.devices
            .iter()
            .find(|d| d.vendor_id == vendor_id)
            .cloned()
            .ok_or_else(|| PortError::DeviceNotFound(DeviceId::new()))
    }

    async fn adopt_device(&self, vendor_id: &str) -> Result<(), PortError> {
        println!("  → Adopting device: {}", vendor_id);
        // In real implementation, this would send adopt command to controller
        Ok(())
    }

    async fn apply_config(&self, vendor_id: &str, _config: VendorConfig) -> Result<(), PortError> {
        println!("  → Applying config to device: {}", vendor_id);
        Ok(())
    }

    async fn restart_device(&self, vendor_id: &str) -> Result<(), PortError> {
        println!("  → Restarting device: {}", vendor_id);
        Ok(())
    }

    async fn get_device_stats(&self, _vendor_id: &str) -> Result<DeviceStats, PortError> {
        Ok(DeviceStats {
            uptime_seconds: 86400, // 1 day
            cpu_percent: Some(15.0),
            memory_percent: Some(45.0),
            temperature_celsius: Some(42.0),
            port_stats: vec![],
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cim_network=info".parse()?)
                .add_directive("demo=info".parse()?),
        )
        .init();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           CIM Network - Device Lifecycle Demo                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Step 1: Connect to NATS
    println!("━━━ Step 1: Connecting to NATS JetStream ━━━");
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let config = NatsEventStoreConfig {
        nats_url: nats_url.clone(),
        stream_name: "demo-network-events".to_string(),
        subject_prefix: "demo-network".to_string(),
        ..Default::default()
    };

    let event_store = match NatsEventStore::new(config).await {
        Ok(store) => {
            println!("  ✓ Connected to NATS at {}", nats_url);
            Arc::new(store)
        }
        Err(e) => {
            println!("  ✗ Failed to connect to NATS: {}", e);
            println!("    Make sure NATS is running: nats-server -js");
            return Err(e.into());
        }
    };

    // Step 2: Create service with mock vendor adapter
    println!("\n━━━ Step 2: Initializing Network Service ━━━");
    let vendor_adapter = MockUniFiAdapter::new();

    let service = NetworkService::builder()
        .event_store_arc(event_store.clone() as Arc<dyn EventStorePort>)
        .vendor_adapter(vendor_adapter)
        .build()?;

    println!("  ✓ Network service initialized");

    // Step 3: Discover devices
    println!("\n━━━ Step 3: Discovering Devices ━━━");
    let discovered_ids = service.discover_devices().await?;
    println!("  ✓ Discovered {} new devices:", discovered_ids.len());

    for device_id in &discovered_ids {
        if let Some(device) = service.get_device(*device_id).await {
            println!(
                "    • {} ({}) - {} - State: {:?}",
                device.name(),
                device.mac(),
                device.device_type(),
                device.state()
            );
        }
    }

    // Step 4: Adopt devices
    println!("\n━━━ Step 4: Adopting Devices ━━━");
    for device_id in &discovered_ids {
        match service.adopt_device(*device_id).await {
            Ok(()) => {
                if let Some(device) = service.get_device(*device_id).await {
                    println!(
                        "  ✓ Adopted {} - State: {:?}",
                        device.name(),
                        device.state()
                    );
                }
            }
            Err(e) => println!("  ✗ Failed to adopt {}: {}", device_id, e),
        }
    }

    // Step 5: Mark devices as provisioned (simulating adoption complete callback)
    println!("\n━━━ Step 5: Provisioning Devices ━━━");
    let models = ["USW-24-POE", "U6-Pro", "UDM-Pro"];
    let firmware = "6.6.65";

    for (i, device_id) in discovered_ids.iter().enumerate() {
        let model = models.get(i).unwrap_or(&"Unknown");
        match service.mark_provisioned(*device_id, model.to_string(), firmware.to_string()).await {
            Ok(()) => {
                if let Some(device) = service.get_device(*device_id).await {
                    println!(
                        "  ✓ Provisioned {} ({}) - Firmware: {} - State: {:?}",
                        device.name(),
                        model,
                        firmware,
                        device.state()
                    );
                }
            }
            Err(e) => println!("  ✗ Failed to provision {}: {}", device_id, e),
        }
    }

    // Step 6: List all devices by state
    println!("\n━━━ Step 6: Device Summary ━━━");
    let all_devices = service.list_devices().await;

    let provisioned = all_devices.iter()
        .filter(|d| d.state() == DeviceState::Provisioned)
        .count();
    let adopting = all_devices.iter()
        .filter(|d| d.state() == DeviceState::Adopting)
        .count();
    let discovered = all_devices.iter()
        .filter(|d| d.state() == DeviceState::Discovered)
        .count();

    println!("  Total devices: {}", all_devices.len());
    println!("    • Provisioned: {}", provisioned);
    println!("    • Adopting: {}", adopting);
    println!("    • Discovered: {}", discovered);

    // Step 7: Demonstrate event replay
    println!("\n━━━ Step 7: Event Replay Demo ━━━");
    if let Some(first_device_id) = discovered_ids.first() {
        println!("  Replaying events for device {}...", first_device_id);

        match service.replay_events(&first_device_id.to_string()).await {
            Ok(Some(reconstructed)) => {
                println!("  ✓ Successfully reconstructed aggregate from events:");
                println!("    • Name: {}", reconstructed.name());
                println!("    • MAC: {}", reconstructed.mac());
                println!("    • Type: {}", reconstructed.device_type());
                println!("    • State: {:?}", reconstructed.state());
                println!("    • Version: {}", reconstructed.version());
            }
            Ok(None) => println!("  No events found for device"),
            Err(e) => println!("  ✗ Replay failed: {}", e),
        }
    }

    // Step 8: Decommission one device
    println!("\n━━━ Step 8: Decommissioning Demo ━━━");
    if let Some(device_id) = discovered_ids.last() {
        if let Some(device) = service.get_device(*device_id).await {
            println!("  Decommissioning {}...", device.name());
            match service.decommission_device(*device_id).await {
                Ok(()) => {
                    if let Some(updated) = service.get_device(*device_id).await {
                        println!(
                            "  ✓ {} is now {:?}",
                            updated.name(),
                            updated.state()
                        );
                    }
                }
                Err(e) => println!("  ✗ Failed: {}", e),
            }
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                      Demo Complete!                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    Ok(())
}

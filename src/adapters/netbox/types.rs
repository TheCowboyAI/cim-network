//! NetBox API types
//!
//! Types for interacting with the NetBox DCIM/IPAM system.

use serde::{Deserialize, Serialize};

/// NetBox API response wrapper with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxResponse<T> {
    /// Total count of results
    pub count: u64,
    /// URL to next page
    pub next: Option<String>,
    /// URL to previous page
    pub previous: Option<String>,
    /// Results array
    pub results: Vec<T>,
}

/// NetBox device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxDevice {
    /// Device ID in NetBox
    pub id: u64,
    /// Device name
    pub name: String,
    /// Device type (nested object)
    pub device_type: Option<NetBoxNestedDeviceType>,
    /// Device role
    pub role: Option<NetBoxNestedObject>,
    /// Site
    pub site: Option<NetBoxNestedObject>,
    /// Rack
    pub rack: Option<NetBoxNestedObject>,
    /// Status
    pub status: Option<NetBoxStatus>,
    /// Primary IPv4
    pub primary_ip4: Option<NetBoxNestedIp>,
    /// Primary IPv6
    pub primary_ip6: Option<NetBoxNestedIp>,
    /// Serial number
    pub serial: Option<String>,
    /// Asset tag
    pub asset_tag: Option<String>,
    /// Custom fields
    #[serde(default)]
    pub custom_fields: serde_json::Value,
}

/// Nested device type reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxNestedDeviceType {
    /// Device type ID
    pub id: u64,
    /// Model name
    pub model: String,
    /// Manufacturer
    pub manufacturer: Option<NetBoxNestedObject>,
}

/// Generic nested object (site, rack, role, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxNestedObject {
    /// Object ID
    pub id: u64,
    /// Object name
    pub name: String,
    /// Object slug
    pub slug: Option<String>,
}

/// Status field with value and label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxStatus {
    /// Status value (e.g., "active")
    pub value: String,
    /// Display label
    pub label: String,
}

/// Nested IP address reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxNestedIp {
    /// IP address ID
    pub id: u64,
    /// IP address with prefix (e.g., "192.168.1.1/24")
    pub address: String,
}

/// NetBox IP address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxIpAddress {
    /// IP address ID
    pub id: u64,
    /// IP address with prefix
    pub address: String,
    /// VRF
    pub vrf: Option<NetBoxNestedObject>,
    /// Status
    pub status: Option<NetBoxStatus>,
    /// DNS name
    pub dns_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Assigned object type
    pub assigned_object_type: Option<String>,
    /// Assigned object ID
    pub assigned_object_id: Option<u64>,
}

/// NetBox cable/connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxCable {
    /// Cable ID
    pub id: u64,
    /// Cable type
    #[serde(rename = "type")]
    pub cable_type: Option<String>,
    /// Status
    pub status: Option<NetBoxStatus>,
    /// A-side terminations
    pub a_terminations: Vec<NetBoxTermination>,
    /// B-side terminations
    pub b_terminations: Vec<NetBoxTermination>,
    /// Label
    pub label: Option<String>,
    /// Color
    pub color: Option<String>,
    /// Length
    pub length: Option<f64>,
    /// Length unit
    pub length_unit: Option<String>,
}

/// Cable termination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxTermination {
    /// Object type (e.g., "dcim.interface")
    pub object_type: String,
    /// Object ID
    pub object_id: u64,
}

/// NetBox prefix (IP subnet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetBoxPrefix {
    /// Prefix ID
    pub id: u64,
    /// Prefix in CIDR notation
    pub prefix: String,
    /// Site
    pub site: Option<NetBoxNestedObject>,
    /// VRF
    pub vrf: Option<NetBoxNestedObject>,
    /// Status
    pub status: Option<NetBoxStatus>,
    /// Description
    pub description: Option<String>,
}

/// Request body for creating a device
#[derive(Debug, Clone, Serialize)]
pub struct NetBoxDeviceCreate {
    /// Device name
    pub name: String,
    /// Device type ID
    pub device_type: u64,
    /// Site ID
    pub site: u64,
    /// Device role ID
    pub role: u64,
    /// Status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Serial number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// Custom fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
}

/// Request body for creating a cable
#[derive(Debug, Clone, Serialize)]
pub struct NetBoxCableCreate {
    /// A-side terminations
    pub a_terminations: Vec<NetBoxTermination>,
    /// B-side terminations
    pub b_terminations: Vec<NetBoxTermination>,
    /// Cable type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub cable_type: Option<String>,
    /// Status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Request body for allocating an IP
#[derive(Debug, Clone, Serialize)]
pub struct NetBoxIpAllocate {
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Status (default: active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Assigned object type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    /// Assigned object ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<u64>,
}

/// NetBox API error
#[derive(Debug, Clone, thiserror::Error)]
pub enum NetBoxError {
    /// HTTP/network error
    #[error("HTTP error: {0}")]
    Http(String),
    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),
    /// API error response
    #[error("API error: {0}")]
    Api(String),
    /// Parse/deserialization error
    #[error("Parse error: {0}")]
    Parse(String),
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

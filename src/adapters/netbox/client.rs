//! NetBox API HTTP client
//!
//! Handles communication with NetBox DCIM/IPAM system.

use super::types::*;
use reqwest::Client;
use std::time::Duration;

/// NetBox API client
pub struct NetBoxClient {
    /// HTTP client
    http: Client,
    /// Base URL of NetBox instance
    base_url: String,
    /// API token
    api_token: String,
}

impl NetBoxClient {
    /// Create a new NetBox client
    ///
    /// # Arguments
    /// * `base_url` - NetBox URL (e.g., "https://netbox.example.com")
    /// * `api_token` - API authentication token
    pub fn new(base_url: &str, api_token: &str) -> Result<Self, NetBoxError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| NetBoxError::Http(e.to_string()))?;

        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token: api_token.to_string(),
        })
    }

    // =========================================================================
    // Device Operations
    // =========================================================================

    /// List all devices
    pub async fn list_devices(&self) -> Result<Vec<NetBoxDevice>, NetBoxError> {
        let url = format!("{}/api/dcim/devices/", self.base_url);
        let response: NetBoxResponse<NetBoxDevice> = self.get(&url).await?;
        Ok(response.results)
    }

    /// Get a device by ID
    pub async fn get_device(&self, id: u64) -> Result<NetBoxDevice, NetBoxError> {
        let url = format!("{}/api/dcim/devices/{}/", self.base_url, id);
        self.get(&url).await
    }

    /// Get a device by name
    pub async fn get_device_by_name(&self, name: &str) -> Result<Option<NetBoxDevice>, NetBoxError> {
        let url = format!("{}/api/dcim/devices/?name={}", self.base_url, urlencoding::encode(name));
        let response: NetBoxResponse<NetBoxDevice> = self.get(&url).await?;
        Ok(response.results.into_iter().next())
    }

    /// Create a new device
    pub async fn create_device(&self, device: &NetBoxDeviceCreate) -> Result<NetBoxDevice, NetBoxError> {
        let url = format!("{}/api/dcim/devices/", self.base_url);
        self.post(&url, device).await
    }

    /// Update a device
    pub async fn update_device(&self, id: u64, device: &serde_json::Value) -> Result<NetBoxDevice, NetBoxError> {
        let url = format!("{}/api/dcim/devices/{}/", self.base_url, id);
        self.patch(&url, device).await
    }

    /// Delete a device
    pub async fn delete_device(&self, id: u64) -> Result<(), NetBoxError> {
        let url = format!("{}/api/dcim/devices/{}/", self.base_url, id);
        self.delete(&url).await
    }

    // =========================================================================
    // Cable Operations
    // =========================================================================

    /// Create a cable connection
    pub async fn create_cable(&self, cable: &NetBoxCableCreate) -> Result<NetBoxCable, NetBoxError> {
        let url = format!("{}/api/dcim/cables/", self.base_url);
        self.post(&url, cable).await
    }

    /// Get a cable by ID
    pub async fn get_cable(&self, id: u64) -> Result<NetBoxCable, NetBoxError> {
        let url = format!("{}/api/dcim/cables/{}/", self.base_url, id);
        self.get(&url).await
    }

    /// Delete a cable
    pub async fn delete_cable(&self, id: u64) -> Result<(), NetBoxError> {
        let url = format!("{}/api/dcim/cables/{}/", self.base_url, id);
        self.delete(&url).await
    }

    // =========================================================================
    // IPAM Operations
    // =========================================================================

    /// Get IP addresses for a prefix
    pub async fn get_ip_addresses(&self, prefix: &str) -> Result<Vec<NetBoxIpAddress>, NetBoxError> {
        let url = format!(
            "{}/api/ipam/ip-addresses/?parent={}",
            self.base_url,
            urlencoding::encode(prefix)
        );
        let response: NetBoxResponse<NetBoxIpAddress> = self.get(&url).await?;
        Ok(response.results)
    }

    /// Get a prefix by CIDR
    pub async fn get_prefix(&self, prefix: &str) -> Result<Option<NetBoxPrefix>, NetBoxError> {
        let url = format!(
            "{}/api/ipam/prefixes/?prefix={}",
            self.base_url,
            urlencoding::encode(prefix)
        );
        let response: NetBoxResponse<NetBoxPrefix> = self.get(&url).await?;
        Ok(response.results.into_iter().next())
    }

    /// Allocate an available IP from a prefix
    pub async fn allocate_ip(
        &self,
        prefix_id: u64,
        allocation: &NetBoxIpAllocate,
    ) -> Result<NetBoxIpAddress, NetBoxError> {
        let url = format!("{}/api/ipam/prefixes/{}/available-ips/", self.base_url, prefix_id);
        self.post(&url, allocation).await
    }

    /// Delete an IP address
    pub async fn delete_ip(&self, id: u64) -> Result<(), NetBoxError> {
        let url = format!("{}/api/ipam/ip-addresses/{}/", self.base_url, id);
        self.delete(&url).await
    }

    // =========================================================================
    // HTTP Helpers
    // =========================================================================

    /// Make a GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, NetBoxError> {
        tracing::debug!("NetBox GET {}", url);

        let response = self.http
            .get(url)
            .header("Authorization", format!("Token {}", self.api_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| NetBoxError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a POST request
    async fn post<T, B>(&self, url: &str, body: &B) -> Result<T, NetBoxError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        tracing::debug!("NetBox POST {}", url);

        let response = self.http
            .post(url)
            .header("Authorization", format!("Token {}", self.api_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| NetBoxError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a PATCH request
    async fn patch<T, B>(&self, url: &str, body: &B) -> Result<T, NetBoxError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        tracing::debug!("NetBox PATCH {}", url);

        let response = self.http
            .patch(url)
            .header("Authorization", format!("Token {}", self.api_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| NetBoxError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Make a DELETE request
    async fn delete(&self, url: &str) -> Result<(), NetBoxError> {
        tracing::debug!("NetBox DELETE {}", url);

        let response = self.http
            .delete(url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .map_err(|e| NetBoxError::Http(e.to_string()))?;

        let status = response.status();
        if status == reqwest::StatusCode::NO_CONTENT || status.is_success() {
            Ok(())
        } else if status == reqwest::StatusCode::NOT_FOUND {
            Err(NetBoxError::NotFound("Resource not found".to_string()))
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err(NetBoxError::Auth("Invalid API token".to_string()))
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(NetBoxError::Api(format!("Request failed: {} - {}", status, body)))
        }
    }

    /// Handle API response
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, NetBoxError> {
        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(NetBoxError::NotFound("Resource not found".to_string()));
        }

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(NetBoxError::Auth("Invalid API token".to_string()));
        }

        if status == reqwest::StatusCode::BAD_REQUEST {
            let body = response.text().await.unwrap_or_default();
            return Err(NetBoxError::Validation(body));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(NetBoxError::Api(format!("Request failed: {} - {}", status, body)));
        }

        response.json::<T>()
            .await
            .map_err(|e| NetBoxError::Parse(e.to_string()))
    }
}

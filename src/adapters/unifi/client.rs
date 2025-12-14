//! UniFi Controller HTTP client
//!
//! Handles authentication and API communication with UniFi Network Application.

use super::types::*;
use reqwest::{Client, cookie::Jar};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// UniFi Controller client
pub struct UniFiClient {
    /// HTTP client with cookie jar
    http: Client,
    /// Base URL of the controller
    base_url: String,
    /// Username for authentication
    username: String,
    /// Password for authentication
    password: String,
    /// CSRF token from login response
    csrf_token: RwLock<Option<String>>,
    /// Whether currently authenticated
    authenticated: RwLock<bool>,
}

impl UniFiClient {
    /// Create a new UniFi client
    ///
    /// # Arguments
    /// * `controller_url` - Base URL (e.g., "https://192.168.1.1:8443")
    /// * `username` - Controller username
    /// * `password` - Controller password
    pub async fn new(
        controller_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, UniFiError> {
        let base_url = controller_url.trim_end_matches('/').to_string();

        // Create cookie jar for session management
        let jar = Arc::new(Jar::default());

        // Build HTTP client
        // Note: UniFi controllers often use self-signed certs
        let http = Client::builder()
            .cookie_provider(jar)
            .danger_accept_invalid_certs(true) // UniFi often uses self-signed
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| UniFiError::Http(e.to_string()))?;

        Ok(Self {
            http,
            base_url,
            username: username.to_string(),
            password: password.to_string(),
            csrf_token: RwLock::new(None),
            authenticated: RwLock::new(false),
        })
    }

    /// Login to the controller
    pub async fn login(&self) -> Result<(), UniFiError> {
        let url = format!("{}/api/login", self.base_url);

        let body = serde_json::json!({
            "username": self.username,
            "password": self.password,
        });

        tracing::info!("Logging into UniFi controller at {}", self.base_url);

        let response = self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| UniFiError::Http(e.to_string()))?;

        // Check for CSRF token in headers
        if let Some(csrf) = response.headers().get("x-csrf-token") {
            if let Ok(token) = csrf.to_str() {
                let mut csrf_lock = self.csrf_token.write()
                    .map_err(|_| UniFiError::Auth("Lock poisoned".to_string()))?;
                *csrf_lock = Some(token.to_string());
            }
        }

        let status = response.status();
        if !status.is_success() {
            return Err(UniFiError::Auth(format!("Login failed with status {}", status)));
        }

        // Parse response to check for errors
        let api_response: UniFiResponse<serde_json::Value> = response.json()
            .await
            .map_err(|e| UniFiError::Parse(e.to_string()))?;

        if !api_response.meta.is_ok() {
            return Err(UniFiError::Auth(
                api_response.meta.msg.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        let mut auth = self.authenticated.write()
            .map_err(|_| UniFiError::Auth("Lock poisoned".to_string()))?;
        *auth = true;

        tracing::info!("Successfully logged into UniFi controller");
        Ok(())
    }

    /// Logout from the controller
    pub async fn logout(&self) -> Result<(), UniFiError> {
        let url = format!("{}/api/logout", self.base_url);

        let mut request = self.http.post(&url);

        // Add CSRF token if we have one
        if let Ok(csrf_lock) = self.csrf_token.read() {
            if let Some(ref token) = *csrf_lock {
                request = request.header("x-csrf-token", token);
            }
        }

        let _ = request.send().await; // Ignore errors on logout

        let mut auth = self.authenticated.write()
            .map_err(|_| UniFiError::Auth("Lock poisoned".to_string()))?;
        *auth = false;

        Ok(())
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated.read()
            .map(|auth| *auth)
            .unwrap_or(false)
    }

    /// List all devices for a site
    pub async fn list_devices(&self, site_id: &str) -> Result<Vec<UniFiDevice>, UniFiError> {
        self.ensure_authenticated()?;

        let url = format!("{}/api/s/{}/stat/device", self.base_url, site_id);

        tracing::debug!("Listing devices for site {}", site_id);

        let response = self.make_request(reqwest::Method::GET, &url, None).await?;
        let api_response: UniFiResponse<UniFiDevice> = response.json()
            .await
            .map_err(|e| UniFiError::Parse(e.to_string()))?;

        if !api_response.meta.is_ok() {
            return Err(UniFiError::Api(
                api_response.meta.msg.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(api_response.data)
    }

    /// Get a specific device by MAC address
    pub async fn get_device(&self, site_id: &str, device_mac: &str) -> Result<UniFiDevice, UniFiError> {
        self.ensure_authenticated()?;

        // UniFi API doesn't have a direct get-by-ID, we filter from list
        let devices = self.list_devices(site_id).await?;

        let mac_normalized = device_mac.to_lowercase().replace([':', '-'], "");

        devices.into_iter()
            .find(|d| d.mac.to_string().to_lowercase().replace([':', '-'], "") == mac_normalized)
            .ok_or_else(|| UniFiError::NotFound(device_mac.to_string()))
    }

    /// Adopt a device
    pub async fn adopt_device(&self, site_id: &str, device_mac: &str) -> Result<(), UniFiError> {
        self.ensure_authenticated()?;

        let url = format!("{}/api/s/{}/cmd/devmgr", self.base_url, site_id);

        let body = serde_json::json!({
            "cmd": "adopt",
            "mac": device_mac.to_lowercase().replace([':', '-'], ""),
        });

        tracing::info!("Adopting device {} in site {}", device_mac, site_id);

        let response = self.make_request(reqwest::Method::POST, &url, Some(body)).await?;
        let api_response: UniFiResponse<serde_json::Value> = response.json()
            .await
            .map_err(|e| UniFiError::Parse(e.to_string()))?;

        if !api_response.meta.is_ok() {
            return Err(UniFiError::Api(
                api_response.meta.msg.unwrap_or_else(|| "Adopt failed".to_string())
            ));
        }

        Ok(())
    }

    /// Set device configuration
    pub async fn set_device_config(
        &self,
        site_id: &str,
        device_id: &str,
        config: &serde_json::Value,
    ) -> Result<(), UniFiError> {
        self.ensure_authenticated()?;

        let url = format!("{}/api/s/{}/rest/device/{}", self.base_url, site_id, device_id);

        tracing::info!("Setting config for device {} in site {}", device_id, site_id);

        let response = self.make_request(reqwest::Method::PUT, &url, Some(config.clone())).await?;
        let api_response: UniFiResponse<serde_json::Value> = response.json()
            .await
            .map_err(|e| UniFiError::Parse(e.to_string()))?;

        if !api_response.meta.is_ok() {
            return Err(UniFiError::Api(
                api_response.meta.msg.unwrap_or_else(|| "Config update failed".to_string())
            ));
        }

        Ok(())
    }

    /// Restart a device
    pub async fn restart_device(&self, site_id: &str, device_mac: &str) -> Result<(), UniFiError> {
        self.ensure_authenticated()?;

        let url = format!("{}/api/s/{}/cmd/devmgr", self.base_url, site_id);

        let body = serde_json::json!({
            "cmd": "restart",
            "mac": device_mac.to_lowercase().replace([':', '-'], ""),
        });

        tracing::info!("Restarting device {} in site {}", device_mac, site_id);

        let response = self.make_request(reqwest::Method::POST, &url, Some(body)).await?;
        let api_response: UniFiResponse<serde_json::Value> = response.json()
            .await
            .map_err(|e| UniFiError::Parse(e.to_string()))?;

        if !api_response.meta.is_ok() {
            return Err(UniFiError::Api(
                api_response.meta.msg.unwrap_or_else(|| "Restart failed".to_string())
            ));
        }

        Ok(())
    }

    /// Get device statistics
    pub async fn get_device_stats(
        &self,
        site_id: &str,
        device_mac: &str,
    ) -> Result<UniFiDeviceStats, UniFiError> {
        // Stats are included in the device response
        let device = self.get_device(site_id, device_mac).await?;

        // Extract stats from device properties
        Ok(UniFiDeviceStats {
            uptime: device.properties.get("uptime")
                .and_then(|v| v.as_u64()),
            cpu_usage: device.properties.get("system-stats")
                .and_then(|v| v.get("cpu"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            mem_usage: device.properties.get("system-stats")
                .and_then(|v| v.get("mem"))
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            temperature: device.properties.get("general_temperature")
                .and_then(|v| v.as_f64()),
            port_stats: extract_port_stats(&device),
        })
    }

    /// Make an authenticated request
    async fn make_request(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, UniFiError> {
        let mut request = self.http.request(method, url);

        // Add CSRF token if we have one
        if let Ok(csrf_lock) = self.csrf_token.read() {
            if let Some(ref token) = *csrf_lock {
                request = request.header("x-csrf-token", token);
            }
        }

        if let Some(json_body) = body {
            request = request.json(&json_body);
        }

        let response = request.send()
            .await
            .map_err(|e| UniFiError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UniFiError::Http(format!("Request failed with status {}", response.status())));
        }

        Ok(response)
    }

    fn ensure_authenticated(&self) -> Result<(), UniFiError> {
        if !self.is_authenticated() {
            return Err(UniFiError::Auth("Not authenticated".to_string()));
        }
        Ok(())
    }
}

/// Extract port statistics from device properties
fn extract_port_stats(device: &UniFiDevice) -> Vec<UniFiPortStats> {
    device.properties.get("port_table")
        .and_then(|v| v.as_array())
        .map(|ports| {
            ports.iter()
                .filter_map(|p| {
                    Some(UniFiPortStats {
                        port_idx: p.get("port_idx")?.as_u64()? as u32,
                        up: p.get("up")?.as_bool()?,
                        speed: p.get("speed").and_then(|v| v.as_u64()).map(|v| v as u32),
                        full_duplex: p.get("full_duplex").and_then(|v| v.as_bool()),
                        rx_bytes: p.get("rx_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                        tx_bytes: p.get("tx_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                        rx_errors: p.get("rx_errors").and_then(|v| v.as_u64()),
                        tx_errors: p.get("tx_errors").and_then(|v| v.as_u64()),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

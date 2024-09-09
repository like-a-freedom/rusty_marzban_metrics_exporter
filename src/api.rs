use log::{debug, error, info, warn};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum APIError {
    RequestFailed {
        url: String,
        status: reqwest::StatusCode,
        body: String,
    },
    InvalidResponse(String),
    ReqwestError {
        url: String,
        error: reqwest::Error,
    },
    EnvVarError(String),
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            APIError::RequestFailed { url, status, body } => {
                write!(
                    f,
                    "Request to {} failed with status {}. Response body: {}",
                    url, status, body
                )
            }
            APIError::InvalidResponse(error) => write!(f, "Invalid response: {}", error),
            APIError::ReqwestError { url, error } => {
                write!(f, "Network error during request to {}: {}", url, error)
            }
            APIError::EnvVarError(var) => write!(f, "Environment variable {} is not set", var),
        }
    }
}

impl From<(String, reqwest::Error)> for APIError {
    fn from(err: (String, reqwest::Error)) -> Self {
        let (url, error) = err;
        APIError::ReqwestError { url, error }
    }
}

impl From<reqwest::Error> for APIError {
    fn from(err: reqwest::Error) -> Self {
        APIError::ReqwestError {
            url: "Unknown".to_string(),
            error: err,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub api_port: u16,
    pub usage_coefficient: f64,
    pub xray_version: String,
    pub status: String,
}

#[derive(Deserialize, Debug)]
pub struct NodeUsageResponse {
    pub usages: Vec<NodeUsage>,
}

#[derive(Deserialize, Debug)]
pub struct NodeUsage {
    pub node_name: String,
    pub uplink: u64,
    pub downlink: u64,
}

#[derive(Deserialize, Debug)]
pub struct SystemData {
    pub version: String,
    pub mem_total: u64,
    pub mem_used: u64,
    pub cpu_cores: u8,
    pub cpu_usage: f64,
    pub total_user: u32,
    pub users_active: u32,
    pub incoming_bandwidth: u64,
    pub outgoing_bandwidth: u64,
    pub incoming_bandwidth_speed: u64,
    pub outgoing_bandwidth_speed: u64,
}

#[derive(Deserialize, Debug)]
pub struct CoreData {
    pub version: String,
    pub started: bool,
}

#[derive(Deserialize, Debug)]
pub struct UserResponse {
    pub users: Vec<User>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub status: String,
    pub used_traffic: u64,
}

#[derive(Clone)]
pub struct MarzbanAPI {
    client: Client,
    base_url: String,
    username: String,
    password: String,
    token: Arc<RwLock<Option<String>>>,
}

impl MarzbanAPI {
    // Initializes the MarzbanAPI by reading environment variables and fetching the initial token.
    pub async fn new() -> Result<Self, APIError> {
        let base_url = env::var("URL").map_err(|_| APIError::EnvVarError("URL".to_string()))?;
        let username =
            env::var("USERNAME").map_err(|_| APIError::EnvVarError("USERNAME".to_string()))?;
        let password =
            env::var("PASSWORD").map_err(|_| APIError::EnvVarError("PASSWORD".to_string()))?;

        let client = Client::new();

        let api = MarzbanAPI {
            client,
            base_url: base_url.clone(),
            username,
            password,
            token: Arc::new(RwLock::new(None)),
        };

        // Fetch the initial token
        api.fetch_token().await?;

        Ok(api)
    }

    // Fetches a new token and updates the token store.
    async fn fetch_token(&self) -> Result<(), APIError> {
        info!(
            "Fetching new access token from {}/api/admin/token",
            self.base_url
        );
        let token_response = self
            .client
            .post(format!("{}/api/admin/token", self.base_url))
            .form(&[("username", &self.username), ("password", &self.password)])
            .send()
            .await
            .map_err(|e| ("Fetching token".to_string(), e))?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().await.unwrap_or_default();
            error!("Failed to fetch token. Status: {}, Body: {}", status, body);
            return Err(APIError::RequestFailed {
                url: format!("{}/api/admin/token", self.base_url),
                status,
                body,
            });
        }

        let token_response: TokenResponse = token_response
            .json()
            .await
            .map_err(|e| APIError::InvalidResponse(format!("Token JSON parse error: {}", e)))?;

        debug!("Retrieved access token: {}", token_response.access_token);
        let mut token_guard = self.token.write().await;
        *token_guard = Some(token_response.access_token);
        Ok(())
    }

    // Ensures that a valid token is available. Fetches a new one if necessary.
    async fn ensure_token(&self) -> Result<String, APIError> {
        {
            let token_guard = self.token.read().await;
            if let Some(ref token) = *token_guard {
                return Ok(token.clone());
            }
        }

        // Token is missing, fetch a new one
        self.fetch_token().await?;
        let token_guard = self.token.read().await;
        token_guard.clone().ok_or_else(|| {
            APIError::InvalidResponse("Token was not set after fetching.".to_string())
        })
    }

    // Generic fetch method with automatic token refresh on 401 Unauthorized.
    pub async fn fetch<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, APIError> {
        let full_url = format!("{}/api{}", self.base_url, endpoint);
        let token = self.ensure_token().await?;

        debug!("Making GET request to {}", full_url);
        let response = self
            .client
            .get(&full_url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ("GET request".to_string(), e))?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            warn!("Received 401 Unauthorized. Attempting to refresh token and retry.");
            self.fetch_token().await?;
            let new_token = self.ensure_token().await?;
            let retry_response = self
                .client
                .get(&full_url)
                .bearer_auth(&new_token)
                .send()
                .await
                .map_err(|e| ("GET request after token refresh".to_string(), e))?;

            return self.handle_response(retry_response, &full_url).await;
        }

        self.handle_response(response, &full_url).await
    }

    // Handles the HTTP response, checking for success and deserializing JSON.
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
        url: &str,
    ) -> Result<T, APIError> {
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            error!(
                "Request to {} failed with status {}. Response body: {}",
                url, status, response_body
            );
            return Err(APIError::RequestFailed {
                url: url.to_string(),
                status,
                body: response_body,
            });
        }

        serde_json::from_str(&response_body).map_err(|e| {
            error!("Failed to parse JSON from {}: {}", url, e);
            APIError::InvalidResponse(e.to_string())
        })
    }

    // Fetches nodes data
    pub async fn fetch_nodes_data(&self) -> Result<Vec<Node>, APIError> {
        self.fetch("/nodes").await
    }

    // Fetches nodes usage data
    pub async fn fetch_nodes_usage_data(&self) -> Result<NodeUsageResponse, APIError> {
        self.fetch("/nodes/usage").await
    }

    // Fetches system data
    pub async fn fetch_system_data(&self) -> Result<SystemData, APIError> {
        self.fetch("/system").await
    }

    // Fetches core data
    pub async fn fetch_core_data(&self) -> Result<CoreData, APIError> {
        self.fetch("/core").await
    }

    // Fetches users data
    pub async fn fetch_users_data(&self) -> Result<UserResponse, APIError> {
        self.fetch("/users").await
    }
}

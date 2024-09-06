use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::fmt;

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
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            APIError::RequestFailed { url, status, body } => write!(
                f,
                "Request to {} failed with status {}. Response body: {}",
                url, status, body
            ),
            APIError::InvalidResponse(error) => write!(f, "Invalid response: {}", error),
            APIError::ReqwestError { url, error } => {
                write!(f, "Network error during request to {}: {}", url, error)
            }
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

#[derive(Deserialize)]
pub struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
pub struct Node {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub api_port: u16,
    pub usage_coefficient: f64,
    pub xray_version: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct NodeUsageResponse {
    pub usages: Vec<NodeUsage>,
}

#[derive(Deserialize)]
pub struct NodeUsage {
    pub node_name: String,
    pub uplink: u64,
    pub downlink: u64,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct CoreData {
    pub version: String,
    pub started: bool,
}

#[derive(Deserialize)]
pub struct UserResponse {
    pub users: Vec<User>,
}

#[derive(Deserialize)]
pub struct User {
    pub username: String,
    pub status: String,
    pub used_traffic: u64,
}

pub struct MarzbanAPI {
    client: Client,
    token: String,
}

impl MarzbanAPI {
    pub async fn new() -> Result<Self, APIError> {
        let url = env::var("URL").expect("URL environment variable must be set");
        let username = env::var("USERNAME").expect("USERNAME environment variable must be set");
        let password = env::var("PASSWORD").expect("PASSWORD environment variable must be set");

        let client = Client::new();
        let token_response = client
            .post(&format!("{}/api/admin/token", url))
            .form(&[("username", &username), ("password", &password)])
            .send()
            .await?
            .json::<TokenResponse>()
            .await
            .map_err(|e| {
                let response_text = format!("Failed to decode token response: {}", e);
                APIError::InvalidResponse(response_text)
            })?;

        Ok(Self {
            client,
            token: token_response.access_token,
        })
    }

    pub async fn fetch<T: for<'de> Deserialize<'de>>(&self, endpoint: &str) -> Result<T, APIError> {
        let url = env::var("URL").expect("URL must be set");
        let full_url = format!("{}/api{}", url, endpoint);

        let response = self
            .client
            .get(&full_url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| APIError::ReqwestError {
                url: full_url.clone(),
                error: e,
            })?;

        let status = response.status();
        let response_body = response.text().await.map_err(|e| APIError::ReqwestError {
            url: full_url.clone(),
            error: e,
        })?;

        if !status.is_success() {
            return Err(APIError::RequestFailed {
                url: full_url,
                status,
                body: response_body,
            });
        }

        serde_json::from_str(&response_body).map_err(|e| APIError::InvalidResponse(e.to_string()))
    }

    pub async fn fetch_nodes_data(&self) -> Result<Vec<Node>, APIError> {
        self.fetch("/nodes").await
    }

    pub async fn fetch_nodes_usage_data(&self) -> Result<NodeUsageResponse, APIError> {
        self.fetch("/nodes/usage").await
    }

    pub async fn fetch_system_data(&self) -> Result<SystemData, APIError> {
        self.fetch("/system").await
    }

    pub async fn fetch_core_data(&self) -> Result<CoreData, APIError> {
        self.fetch("/core").await
    }

    pub async fn fetch_users_data(&self) -> Result<UserResponse, APIError> {
        self.fetch("/users").await
    }
}

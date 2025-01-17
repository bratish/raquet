use std::time::Instant;
use crate::models::ResponseMetadata;
use super::state::App;
use log::{debug, error, info};
use std::collections::HashMap;
use chrono::Utc;

pub struct RequestHandler;

impl RequestHandler {
    pub async fn send_request(app: &mut App) {
        // Validate URL
        if !app.url.starts_with("http://") && !app.url.starts_with("https://") {
            app.url = format!("http://{}", app.url);
            debug!("Added http:// prefix to URL: {}", app.url);
        }

        // Update dynamic headers
        if let Ok(url) = reqwest::Url::parse(&app.url) {
            if let Some(host) = url.host_str() {
                app.headers.insert("Host".to_string(), host.to_string());
                debug!("Updated Host header to: {}", host);
            }
        }

        app.headers.insert(
            "Random-Token".to_string(), 
            uuid::Uuid::new_v4().to_string()
        );
        debug!("Generated new Random-Token");

        app.headers.insert(
            "Content-Length".to_string(), 
            app.body.len().to_string()
        );
        debug!("Updated Content-Length to: {}", app.body.len());

        debug!("Preparing to send request to: {}", app.url);
        debug!("Method: {}", app.method.as_str());
        
        // Filter enabled headers
        let enabled_headers: HashMap<_, _> = app.headers.iter()
            .filter(|(key, _)| *app.header_enabled.get(&**key).unwrap_or(&true))
            .map(|(k, v)| {
                let value = match k.as_str() {
                    "Content-Length" if v == "<calculated>" => {
                        app.body.len().to_string()
                    },
                    "Host" if v == "<host of the machine>" => {
                        if let Ok(url) = reqwest::Url::parse(&app.url) {
                            url.host_str().unwrap_or("").to_string()
                        } else {
                            v.clone()
                        }
                    },
                    _ => v.clone()
                };
                (k.clone(), value)
            })
            .collect();
        
        debug!("Enabled headers: {:?}", enabled_headers);

        // Create client with timeouts
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        let mut request = client.request(
            reqwest::Method::from_bytes(app.method.as_str().as_bytes()).unwrap(),
            &app.url
        );

        // Add enabled headers
        for (key, value) in enabled_headers {
            request = request.header(key, value);
        }

        // Add body if present
        if !app.body.is_empty() {
            debug!("Request body: {}", app.body);
            request = request.body(app.body.clone());
        }

        debug!("Sending request to {} with method {}", app.url, app.method.as_str());
        let start_time = Instant::now();
        match request.send().await {
            Ok(response) => {
                debug!("Got response with status: {}", response.status());
                let status = response.status();
                let headers = response.headers().iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                info!("Response received: {} {}", status.as_u16(), status.as_str());
                
                match response.text().await {
                    Ok(body) => {
                        let elapsed = start_time.elapsed();
                        debug!("Response body received, length: {}", body.len());
                        app.response = Some(body.clone());
                        app.response_metadata = Some(ResponseMetadata {
                            status: status.as_u16(),
                            status_text: status.to_string(),
                            time_ms: elapsed.as_millis(),
                            size_bytes: body.len(),
                            response_headers: headers,
                            timestamp: Utc::now(),
                        });
                    }
                    Err(e) => {
                        error!("Failed to read response body: {}", e);
                        app.response = Some(format!("Error reading response: {}", e));
                    }
                }
            }
            Err(e) => {
                error!("Request failed: {}", e);
                if e.is_timeout() {
                    error!("Request timed out");
                } else if e.is_connect() {
                    error!("Connection error: {}", e);
                } else if e.is_request() {
                    error!("Request error: {}", e);
                }
                app.response = Some(format!("Error: {}", e));
            }
        }
    }
} 
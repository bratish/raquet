use serde_json::Value;

pub fn format_response_body(content_type: &str, body: &str) -> String {
    match content_type.split(';').next().unwrap_or("").trim() {
        "application/json" => {
            if let Ok(json) = serde_json::from_str::<Value>(body) {
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string())
            } else {
                body.to_string()
            }
        }
        _ => body.to_string()
    }
}

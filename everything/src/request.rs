use reqwest::Client;
use serde::Deserialize;
use futures_util::StreamExt;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct OllamaStreamResponse {
    response: String,
    done: bool,
}

pub async fn request_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
    // Create a new HTTP client
    let client = Client::new();

    // Define the URL and the payload
    let url = "http://localhost:11434/api/generate";
    let payload = serde_json::json!({
        "model": "llama3",
        "prompt": prompt
    });

    // Send the POST request and get the response stream
    let res = client.post(url)
        .json(&payload)
        .send()
        .await?;

    // Check if the response status is success
    if !res.status().is_success() {
        return Err(Box::from(format!("Request failed with status: {}", res.status())));
    }

    // Stream the response body and collect the "response" fields
    let mut stream = res.bytes_stream();
    let mut full_response = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        // Parse the chunk as JSON
        if let Ok(json) = serde_json::from_slice::<OllamaStreamResponse>(&chunk) {
            full_response.push_str(&json.response);
            if json.done {
                break;
            }
        }
    }

    Ok(full_response)
}
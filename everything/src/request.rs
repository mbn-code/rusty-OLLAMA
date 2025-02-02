use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    response: String,
}


pub async fn request_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
    let context = "You are a professional software developer that knows everything about development. Now this is the prompt from the user: \n";

    let client = Client::new();
    let url = "http://localhost:11434/api/generate";
    let payload = serde_json::json!({
        "model": "llama3",
        "prompt": context.to_owned() + prompt,
        "stream": false
    });

    // println!("Sending request to: {}", url);
    // println!("Payload: {}", payload);

    // Send the POST request
    let res = client.post(url)
        .json(&payload)
        .send()
        .await?;

    // Extract the response status code
    let status = res.status();

    // Collect the response text for debugging
    let text = res.text().await?;

    // Check if the response status is success
    if status.is_success() {
        let response_json: OllamaResponse = serde_json::from_str(&text)?;
        Ok(response_json.response)
    } else {
        Err(Box::from(format!("Request failed with status: {}. Response body: {}", status, text)))
    }
}

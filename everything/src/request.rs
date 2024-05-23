

pub async fn request_ollama(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let res = client.post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": "llama3",
            "prompt": prompt
        }))
        .send()
        .await?;

    let text = res.text().await?;

    Ok(text)
}
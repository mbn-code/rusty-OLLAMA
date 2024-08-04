mod request;

#[tokio::main]
async fn main() {
    match request::request_ollama("Who is the ollama3 monkey?").await {
        Ok(response) => println!("{}", response),
        Err(e) => println!("Request failed: {}", e),
    }
}

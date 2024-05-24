mod request;





#[tokio::main]
async fn main() {
    match request::request_ollama("What's a cow").await {
        Ok(response) => println!("{}", response),
        Err(e) => println!("Request failed: {}", e),
    
    
    }

}
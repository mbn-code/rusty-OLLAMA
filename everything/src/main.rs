mod request;

#[tokio::main]
async fn main() {
    use std::io::{stdin, stdout, Write};

    loop {
        let mut user_input = String::new();
        print!("You: ");
        let _ = stdout().flush();
        stdin().read_line(&mut user_input).expect("Did not enter a correct string");
        if let Some('\n') = user_input.chars().next_back() {
            user_input.pop();
        }
        if let Some('\r') = user_input.chars().next_back() {
            user_input.pop();
        }

        if user_input.to_lowercase() == "exit" {
            break;
        }

        match request::request_ollama(&user_input).await {
            Ok(response) => println!("{}", response),
            Err(e) => println!("Request failed: {}", e),
        }
    }
}

# rusty ollama

A Rust project named "rusty ollama" that leverages asynchronous programming to make requests to an external service, parse the responses, and handle them accordingly. This project is built with a focus on learning and demonstrating the use of various Rust features and libraries, including `reqwest` for making HTTP requests, `serde` for serialization and deserialization, `tokio` for asynchronous runtime, and `futures-util` for working with futures.

## Features

- Asynchronous HTTP requests using `reqwest`
- JSON parsing with `serde` and `serde_json`
- Comprehensive error handling
- Use of Rust's powerful type system and async/await syntax for clean and efficient code

## Dependencies

- `reqwest` for making HTTP requests
- `serde` and `serde_json` for JSON parsing
- `tokio` as the asynchronous runtime
- `futures-util` for additional utilities when working with futures

## Getting Started

To get started with "rusty ollama", ensure you have Rust and Cargo installed on your machine. Then, clone this repository and navigate into the project directory.

```sh
git clone https://github.com/mbn-code/rusty-OLLAMA.git
cd rusty-OLLAMA

cargo build
cargo run
```

# Purpose of the repository

The purpose of this repository is for you to have an easy setup for async integration of OLLAMA models in rust. 


### Contributing

Contributions are welcome! Please feel free to clone or fork the repository and open a pull-request.




This project is licensed under the MIT License - see the LICENSE file for details.


mod request;

use eframe::egui;
use std::sync::{Arc, Mutex};

struct App {
    conversation: Vec<String>,
    input_text: String,
    pending_responses: Arc<Mutex<Vec<String>>>,
}

impl App {
    fn new() -> Self {
        Self {
            conversation: Vec::new(),
            input_text: String::new(),
            pending_responses: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Display conversation history
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in &self.conversation {
                    ui.label(message);
                }
            });

            // Input area with send button
            ui.horizontal(|ui| {
                let text_edit = ui.text_edit_singleline(&mut self.input_text);
                
                // Send on Enter or button click
                let send_clicked = ui.button("Send").clicked() 
                    || text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                
                if send_clicked {
                    let user_input = self.input_text.trim().to_string();
                    if !user_input.is_empty() {
                        // Add user message to conversation
                        self.conversation.push(format!("You: {}", user_input));
                        
                        // Clone needed variables for async closure
                        let pending_responses = self.pending_responses.clone();
                        let input_clone = user_input.clone();
                        self.input_text.clear();

                        // Spawn async request
                        tokio::spawn(async move {
                            match request::request_ollama(&input_clone).await {
                                Ok(response) => {
                                    pending_responses.lock().unwrap().push(response);
                                }
                                Err(e) => {
                                    pending_responses.lock().unwrap().push(
                                        format!("Error: {}", e)
                                    );
                                }
                            }
                        });
                    }
                }
            });
        });

        // Process pending responses
        let mut pending = self.pending_responses.lock().unwrap();
        for response in pending.drain(..) {
            self.conversation.push(format!("Ollama: {}", response));
        }

        // Automatically scroll to bottom
        ctx.request_repaint();
    }
}

fn main() {
    // Initialize tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    // Launch GUI
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Ollama Chat",
        options,
        Box::new(|_cc| Box::new(App::new())),
    );
}
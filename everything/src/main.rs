mod request;

use eframe::egui::{self, RichText, ScrollArea};
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
        // Process pending responses first
        let mut pending = self.pending_responses.lock().unwrap();
        for response in pending.drain(..) {
            self.conversation.push(response);
        }
        drop(pending); // Release the lock immediately

        egui::CentralPanel::default().show(ctx, |ui| {
            // Chat history with scroll area
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for message in &self.conversation {
                        let (sender, content) = message.split_once(':').unwrap_or(("", message));
                        let is_user = sender.trim() == "You";
                        
                        // Message bubble styling
                        let bubble_color = if is_user {
                            egui::Color32::from_rgb(0, 92, 175)
                        } else {
                            egui::Color32::from_rgb(229, 229, 229)
                        };
                        
                        let text_color = if is_user {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::BLACK
                        };

                        ui.with_layout(
                            if is_user {
                                egui::Layout::right_to_left(egui::Align::Center)
                            } else {
                                egui::Layout::left_to_right(egui::Align::Center)
                            },
                            |ui| {
                                egui::Frame::none()
                                    .fill(bubble_color)
                                    .rounding(5.0)
                                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(content.trim_start())
                                                .color(text_color)
                                                .text_style(egui::TextStyle::Body),
                                        );
                                    });
                            },
                        );
                        ui.add_space(8.0);
                    }
                });

            // Input area
            ui.add_space(8.0);
            ui.separator();
            egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let text_edit = ui.add(
                        egui::TextEdit::multiline(&mut self.input_text)
                            .desired_width(f32::INFINITY - 40.0)
                            .hint_text("Type your message...")
                    );

                    let send_btn = ui.add_sized(
                        [40.0, 40.0], 
                        egui::Button::new(RichText::new("➤").size(20.0))
                    );

                    let send_clicked = send_btn.clicked() 
                        || (text_edit.lost_focus() 
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && !ui.input(|i| i.modifiers.shift));

                    if send_clicked {
                        let user_input = self.input_text.trim().to_string();
                        if !user_input.is_empty() {
                            self.conversation.push(format!("You: {}", user_input));
                            self.input_text.clear();
                            
                            // Clone necessary values for async closure
                            let pending = self.pending_responses.clone();
                            let input = user_input.clone();
                            
                            // Spawn async request using the runtime handle
                            tokio::spawn(async move {
                                match request::request_ollama(&input).await {
                                    Ok(response) => {
                                        pending.lock().unwrap().push(
                                            format!("Assistant: {}", response)
                                        );
                                    }
                                    Err(e) => {
                                        pending.lock().unwrap().push(
                                            format!("Error: {}", e)
                                        );
                                    }
                                }
                            });
                        }
                    }
                });
            });
        });

        // Force continuous repaint while waiting for responses
        if self.pending_responses.lock().unwrap().is_empty() {
            ctx.request_repaint();
        }
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let _enter = rt.enter();
    
    let options = eframe::NativeOptions {
        vsync: false, // Disable VSYNC for faster repaints
        ..Default::default()
    };
    
    eframe::run_native(
        "Chat Assistant",
        options,
        Box::new(|_cc| Box::new(App::new())),
    );
}
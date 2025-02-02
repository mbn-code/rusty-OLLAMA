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
            if let Some((_, temp_id)) = response.split_once('(') {
                if let Some(position) = self.conversation.iter().position(|m| m.contains(temp_id)) {
                    self.conversation[position] = response.replace(&format!(" ({})", temp_id), "");
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Chat history with bottom padding
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Reserve space for input panel + padding
                    let available_height = ui.available_height() - 150.0;
                    ui.set_min_height(available_height);

                    for message in &self.conversation {
                        let (sender, content) = message.split_once(':').unwrap_or(("", message));
                        let is_user = sender.trim() == "You";
                        
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
                                egui::Layout::right_to_left(egui::Align::Min)
                            } else {
                                egui::Layout::left_to_right(egui::Align::Min)
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
                    
                    // Add padding at bottom of messages
                    ui.add_space(50.0);
                });
        });

        // Input panel
        egui::TopBottomPanel::bottom("input_panel")
            .exact_height(100.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        // Text input with scroll
                        let text_edit = ui.add(
                            egui::TextEdit::multiline(&mut self.input_text)
                                .desired_width(900.0)
                                .hint_text("Type your message...")
                                .desired_rows(3)
                        );

                        // Send button
                        ui.vertical(|ui| {
                            ui.add_space(10.0);
                            let send_btn = ui.add_sized(
                                [60.0, 60.0],
                                egui::Button::new(RichText::new("➤").size(24.0))
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
                                    
                                    // Add typing indicator
                                    let temp_id = format!("typing_{}", std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis());
                                    
                                    self.conversation.push(format!("Assistant: █ ({})", temp_id));

                                    let pending = self.pending_responses.clone();
                                    let input = user_input.clone();
                                    
                                    tokio::spawn(async move {
                                        match request::request_ollama(&input).await {
                                            Ok(response) => {
                                                pending.lock().unwrap().push(
                                                    format!("Assistant: {} ({})", response, temp_id)
                                                );
                                            }
                                            Err(e) => {
                                                pending.lock().unwrap().push(
                                                    format!("Error: {} ({})", e, temp_id)
                                                );
                                            }
                                        }
                                    });
                                }
                            }
                        });
                    });
                });
            });

        // Force immediate repaint
        ctx.request_repaint();
    }
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let _enter = rt.enter();
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([950.0, 800.0])
            .with_resizable(true),
        vsync: false,
        ..Default::default()
    };
    
    eframe::run_native(
        "Chat Assistant",
        options,
        Box::new(|_cc| Box::new(App::new())),
    );
}

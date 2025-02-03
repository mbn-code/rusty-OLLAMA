mod request;

use eframe::egui::{self, RichText, ScrollArea};
use std::sync::{Arc, Mutex};

struct DebugInfo {
    events: Vec<String>,
    frame_times: Vec<f32>,
    last_frame_time: f64,
    errors: Vec<String>,
}

impl Default for DebugInfo {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            frame_times: Vec::new(),
            last_frame_time: 0.0,
            errors: Vec::new(),
        }
    }
}

struct App {
    conversation: Vec<String>,
    input_text: String,
    pending_responses: Arc<Mutex<Vec<String>>>,
    debug_mode: bool,  // This is our debug switch
    debug_info: DebugInfo,
}

impl App {
    // Add debug_mode parameter to constructor
    fn new(debug_mode: bool) -> Self {
        Self {
            conversation: Vec::new(),
            input_text: String::new(),
            pending_responses: Arc::new(Mutex::new(Vec::new())),
            debug_mode,
            debug_info: DebugInfo::default(),
        }
    }

    // Helper function to safely access pending responses
    fn get_pending_responses(&self) -> Vec<String> {
        self.pending_responses
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Toggle debug mode on F12
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.debug_mode = !self.debug_mode;
        }

        // Collect input events
        let events = ctx.input(|i| i.events.clone());
        for event in &events {
            match event {
                egui::Event::Key { key, pressed, .. } if *pressed => {
                    self.debug_info.events.push(format!("Key pressed: {:?}", key));
                }
                egui::Event::PointerButton { button, pressed, .. } if *pressed => {
                    self.debug_info.events.push(format!("Mouse pressed: {:?}", button));
                }
                _ => {}
            }
            // Keep only last 20 events
            if self.debug_info.events.len() > 20 {
                self.debug_info.events.remove(0);
            }
        }

        // Calculate frame time
        let current_time = ctx.input(|i| i.time);
        if self.debug_info.last_frame_time != 0.0 {
            let delta = (current_time - self.debug_info.last_frame_time) as f32;
            self.debug_info.frame_times.push(delta);
            if self.debug_info.frame_times.len() > 100 {
                self.debug_info.frame_times.remove(0);
            }
        }
        self.debug_info.last_frame_time = current_time;

        // Process pending responses
        let mut pending = match self.pending_responses.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                self.debug_info.errors.push("Mutex poisoned!".to_string());
                poisoned.into_inner()
            }
        };
        let responses: Vec<String> = pending.drain(..).collect(); // Collect all pending responses
        drop(pending); // Release the lock immediately

        // Process responses with more debug details
        for response in responses {
            self.debug_info.events.push(format!("Processing response: {}", response));
            if response.starts_with("Error:") {
                self.debug_info.errors.push(response.clone());
            }

            if let Some((content, temp_id)) = response.rsplit_once('(') {  // Changed line
                let clean_response = content.trim().to_string();
                let temp_id = temp_id.trim_matches(|c| c == ')' || c == ' ');
                
                // Use ends_with to accurately match the typing indicator message
                let search = format!("({})", temp_id);
                if let Some(position) = self.conversation.iter().position(|m| m.trim_end().ends_with(&search)) {
                    self.debug_info.events.push(format!(
                        "Updating conversation at position {} with response: {}",
                        position, clean_response
                    ));
                    self.conversation[position] = format!("Assistant: {}", clean_response);
                } else {
                    self.debug_info.events.push(format!(
                        "No match found in conversation for temp_id: {}",
                        temp_id
                    ));
                }
            } else {
                self.debug_info.events.push(format!(
                    "Response did not contain a temp_id: {}",
                    response
                ));
            }
        }

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
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
                                        ui.add(
                                            egui::Label::new(content.trim_start())
                                            .wrap(true)
                                        )
                                    });
                            },
                        );
                        ui.add_space(8.0);
                    }
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
                        let text_edit = ui.add(
                            egui::TextEdit::multiline(&mut self.input_text)
                                .desired_width(f32::INFINITY)
                                .hint_text("Type your message...")
                                .desired_rows(3)
                        );

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

        // Debug window
        if self.debug_mode {
            egui::Window::new("Debug Panel")
                .default_open(true)
                .resizable(true)
                .scroll2([true, true])
                .show(ctx, |ui| {
                    ui.heading("Debug Mode (F12 to toggle)");
                    
                    egui::Grid::new("debug_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Frame time");
                            ui.label(format!("{:.2} ms", self.debug_info.frame_times.last().unwrap_or(&0.0) * 1000.0));
                            ui.end_row();
                            
                            ui.label("Avg FPS");
                            let avg_fps = if !self.debug_info.frame_times.is_empty() {
                                1.0 / (self.debug_info.frame_times.iter().sum::<f32>() / self.debug_info.frame_times.len() as f32)
                            } else { 0.0 };
                            ui.label(format!("{:.1}", avg_fps));
                            ui.end_row();
                        });

                    ui.collapsing("Conversation State", |ui| {
                        ui.label(format!("Messages: {}", self.conversation.len()));
                        ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (i, msg) in self.conversation.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("[{}]", i));
                                        ui.label(msg);
                                    });
                                }
                            });
                    });

                    ui.collapsing("Pending Responses", |ui| {
                        let pending = self.get_pending_responses();
                        ui.label(format!("Count: {}", pending.len()));
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for resp in &pending {
                                    ui.label(resp);
                                }
                            });
                    });

                    ui.collapsing("Input State", |ui| {
                        ui.label(format!("Length: {}", self.input_text.len()));
                        ui.label("Content:");
                        ui.monospace(&self.input_text);
                    });

                    ui.collapsing("Recent Events", |ui| {
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for event in &self.debug_info.events {
                                    ui.label(event);
                                }
                            });
                    });

                    ui.collapsing("Errors", |ui| {
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for error in &self.debug_info.errors {
                                    ui.label(error);
                                }
                            });
                    });
                });
        }

        ctx.request_repaint();
    }
}

fn main() {
    // Set debug mode from command line or configuration
    let debug_mode = true;  // Force enable debug mode

    
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let _enter = rt.enter();
    
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([600.0, 800.0])
            .with_resizable(true),
        vsync: false,
        ..Default::default()
    };
    
    eframe::run_native(
        "Chat Assistant",
        options,
        // Make closure move so that debug_mode is captured by value
        Box::new(move |_cc| Box::new(App::new(debug_mode))),
    )
    .unwrap();
}
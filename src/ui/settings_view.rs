use crate::services::llm::get_tokio_handle;
use crate::ui::state::{AppMessage, AppState, View};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct SettingsView;

impl SettingsView {
    pub fn show(ctx: &egui::Context, state: &mut AppState, tx: &Arc<Mutex<Option<AppMessage>>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("⚙️ Settings");
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.heading("🤖 LLM Configuration");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Enable LLM:");
                        ui.checkbox(&mut state.settings.llm.enabled, "");
                    });

                    ui.add_space(10.0);

                    if state.settings.llm.enabled {
                        ui.horizontal(|ui| {
                            ui.label("API Key:");
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut state.settings.llm.api_key)
                                    .password(true)
                                    .desired_width(300.0)
                            );
                            if response.changed() {
                                // API key changed
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Model Name:");
                            ui.add(
                                egui::TextEdit::singleline(&mut state.settings.llm.model_name)
                                    .desired_width(300.0)
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Base URL (optional):");
                            let base_url = state.settings.llm.base_url.get_or_insert_default();
                            ui.add(
                                egui::TextEdit::singleline(base_url)
                                    .desired_width(300.0)
                                    .hint_text("https://api.openai.com/v1")
                            );
                        });

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("🧪 Test Connection").clicked() {
                                let settings = state.settings.llm.clone();
                                let llm_service = state.llm_service.clone();
                                let tx_clone = tx.clone();
                                
                                if let Some(handle) = get_tokio_handle() {
                                    handle.spawn(async move {
                                        let result = llm_service.test_connection(&settings).await
                                            .map_err(|e| e.to_string());
                                        if let Ok(mut lock) = tx_clone.lock() {
                                            *lock = Some(AppMessage::LlmTestResult(result));
                                        }
                                    });
                                }
                            }

                            if ui.button("💾 Save LLM Settings").clicked() {
                                if let Err(e) = state.save_settings() {
                                    state.error_message = Some(e);
                                } else {
                                    state.error_message = Some("LLM settings saved successfully!".to_string());
                                }
                            }

                            if ui.button("🗑️ Remove LLM").clicked() {
                                state.settings.llm = Default::default();
                                if let Err(e) = state.save_settings() {
                                    state.error_message = Some(e);
                                } else {
                                    state.error_message = Some("LLM configuration removed.".to_string());
                                }
                            }
                        });

                        if let Some(base_url) = &state.settings.llm.base_url {
                            ui.small(format!("Using base URL: {}", base_url));
                        } else {
                            ui.small("Using default OpenAI-compatible API endpoint");
                        }
                    } else {
                        ui.label("LLM integration is disabled. Enable it to use AI-powered metadata fetching.");
                    }
                });

                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.heading("📁 General");
                    ui.separator();
                    
                    if let Some(folder) = &state.settings.last_opened_folder {
                        ui.label(format!("Last opened folder: {}", folder.display()));
                    }

                    ui.add_space(10.0);
                    
                    if ui.button("🔄 Reset All Settings").clicked() {
                        state.settings = Default::default();
                        if let Err(e) = state.save_settings() {
                            state.error_message = Some(e);
                        } else {
                            state.error_message = Some("All settings reset to defaults.".to_string());
                        }
                    }
                });

                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.heading("ℹ️ About");
                    ui.separator();
                    ui.label("MP3 Tagger v0.1.0");
                    ui.label("A native Rust MP3 metadata editor with LLM-powered metadata fetching.");
                    ui.add_space(10.0);
                    ui.label("Supported formats: MP3, FLAC, OGG, M4A, WAV, AIFF, APE");
                    ui.label("Metadata standard: ID3v2, Vorbis Comments, APE Tags");
                });
            });

            // Back button at bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if ui.button("← Back to Library").clicked() {
                    state.current_view = View::Grid;
                }
            });
        });

        // Test connection result
        let error_msg = state.error_message.clone();
        if let Some(error) = error_msg {
            if error.contains("Test") || error.contains("connection") || error.contains("saved") || error.contains("removed") || error.contains("reset") {
                egui::Window::new("Result")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        let is_error = error.contains("Error") || error.contains("Failed") || error.contains("failed");
                        ui.colored_label(
                            if is_error { egui::Color32::RED } else { egui::Color32::GREEN },
                            error
                        );
                        ui.separator();
                        if ui.button("OK").clicked() {
                            state.error_message = None;
                        }
                    });
            }
        }
    }
}
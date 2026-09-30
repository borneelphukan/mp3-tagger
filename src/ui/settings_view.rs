use crate::models::LLMSettings;
use crate::services::llm::get_tokio_handle;
use crate::ui::state::{AppMessage, AppState};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct SettingsView;

impl SettingsView {
    pub fn show_popup(
        ctx: &egui::Context,
        state: &mut AppState,
        draft: &mut LLMSettings,
        local_mode: &mut bool,
        open: &mut bool,
        tx: &Arc<Mutex<Option<AppMessage>>>,
    ) {
        let mut close = false;
        egui::Window::new("AI Settings")
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                ui.set_height(100.0);
                ui.set_width(460.0);
                ui.heading("LLM Service Configuration");
                ui.label("Configure your preferred AI model provider for Fill Data.");

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.selectable_value(local_mode, false, "API Key");
                        ui.selectable_value(local_mode, true, "Local LLM");
                    });
                    ui.label("Model:");
                    egui::ComboBox::from_id_salt("llm_model_preset")
                        .selected_text(if draft.model_name.is_empty() {
                            "Select your LLM Model"
                        } else {
                            &draft.model_name
                        })
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            for model in ["gpt-4o-mini", "gpt-4o", "Custom"] {
                                if ui
                                    .selectable_label(draft.model_name == model, model)
                                    .clicked()
                                {
                                    if model != "Custom" {
                                        draft.model_name = model.to_owned();
                                    } else if draft.model_name.is_empty() {
                                        draft.model_name.clear();
                                    }
                                }
                            }
                        });
                    ui.add_space(4.0);

                    ui.label("Custom Model Name:");
                    ui.add_sized(
                        [ui.available_width(), 30.0],
                        egui::TextEdit::singleline(&mut draft.model_name).hint_text(
                            if *local_mode {
                                "e.g. llama-3.2-3b"
                            } else {
                                "e.g. gpt-4o-mini"
                            },
                        ),
                    );
                    ui.add_space(4.0);

                    if *local_mode {
                        ui.label("Endpoint URL:");
                        ui.add_sized(
                            [ui.available_width(), 30.0],
                            egui::TextEdit::singleline(
                                draft
                                    .base_url
                                    .get_or_insert_with(|| "http://localhost:1234/v1".to_owned()),
                            )
                            .hint_text("http://localhost:1234/v1"),
                        );
                    } else {
                        ui.label("API Key:");
                        ui.add_sized(
                            [ui.available_width(), 30.0],
                            egui::TextEdit::singleline(&mut draft.api_key)
                                .password(true)
                                .hint_text("sk-..."),
                        );
                        ui.add_space(4.0);
                        ui.label("Endpoint URL:");
                        ui.add_sized(
                            [ui.available_width(), 30.0],
                            egui::TextEdit::singleline(
                                draft
                                    .base_url
                                    .get_or_insert_with(|| "https://api.openai.com/v1".to_owned()),
                            )
                            .hint_text("https://api.openai.com/v1"),
                        );
                    }
                });

                if let Some(error) = &state.error_message {
                    if error.contains("Connection")
                        || error.contains("connection")
                        || error.contains("LLM")
                    {
                        ui.add_space(4.0);
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                }

                ui.add_space(4.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        let credentials_ready = !draft.model_name.trim().is_empty()
                            && (*local_mode || !draft.api_key.trim().is_empty());
                        if ui
                            .add_enabled(credentials_ready, egui::Button::new("Save"))
                            .clicked()
                        {
                            draft.enabled = true;
                            state.settings.llm = draft.clone();
                            if let Err(error) = state.save_settings() {
                                state.error_message = Some(error);
                            } else {
                                close = true;
                            }
                        }
                        if ui
                            .add_enabled(credentials_ready, egui::Button::new("Test connection"))
                            .clicked()
                        {
                            // Temporarily enable settings for testing (if not already enabled)
                            let was_enabled = draft.enabled;
                            draft.enabled = true;
                            let settings = draft.clone();
                            // Only reset if it wasn't already enabled
                            if !was_enabled {
                                draft.enabled = false;
                            }
                            let service = state.llm_service.clone();
                            let sender = tx.clone();
                            if let Some(handle) = get_tokio_handle() {
                                handle.spawn(async move {
                                    let result = service
                                        .test_connection(&settings)
                                        .await
                                        .map_err(|e| e.to_string());
                                    if let Ok(mut slot) = sender.lock() {
                                        *slot = Some(AppMessage::LlmTestResult(result));
                                    }
                                });
                            }
                        }
                    });
                });
            });
        if close {
            *open = false;
        }
    }
}

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
        open: &mut bool,
        tx: &Arc<Mutex<Option<AppMessage>>>,
    ) {
        let mut close = false;
        egui::Window::new("AI Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(470.0)
            .open(open)
            .show(ctx, |ui| {
                ui.label("Configure one OpenAI-compatible LLM for Fill Data.");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("API key");
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.api_key)
                            .password(true)
                            .desired_width(310.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Model name");
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.model_name)
                            .hint_text("e.g. gpt-4o-mini")
                            .desired_width(280.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Base URL");
                    ui.add(
                        egui::TextEdit::singleline(draft.base_url.get_or_insert_default())
                            .hint_text("https://api.openai.com/v1")
                            .desired_width(300.0),
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !draft.api_key.trim().is_empty() && !draft.model_name.trim().is_empty(),
                            egui::Button::new("Test connection"),
                        )
                        .clicked()
                    {
                        let settings = draft.clone();
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
                    if ui.button("Remove LLM").clicked() {
                        state.settings.llm = LLMSettings::default();
                        *draft = LLMSettings::default();
                        if let Err(error) = state.save_settings() {
                            state.error_message = Some(error);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                        if ui
                            .add_enabled(
                                !draft.api_key.trim().is_empty()
                                    && !draft.model_name.trim().is_empty(),
                                egui::Button::new("Save"),
                            )
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
                    });
                });
                if let Some(error) = &state.error_message {
                    if error.contains("Connection")
                        || error.contains("connection")
                        || error.contains("LLM")
                    {
                        ui.add_space(6.0);
                        ui.colored_label(egui::Color32::LIGHT_RED, error);
                    }
                }
            });
        if close {
            *open = false;
        }
    }
}

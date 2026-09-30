pub mod detail_view;
pub mod grid_view;
pub mod settings_view;
pub mod state;

use crate::models::{LLMSettings, ThemePreference};
use crate::ui::state::{AppMessage, AppState, View};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct Mp3TaggerApp {
    state: AppState,
    message_tx: Arc<Mutex<Option<AppMessage>>>,
    show_ai_settings: bool,
    ai_settings_draft: LLMSettings,
    ai_settings_local_mode: bool,
    show_about: bool,
}

impl Mp3TaggerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::new().expect("Failed to initialize app state");
        Self::setup_style(&cc.egui_ctx);
        Self::apply_theme(&cc.egui_ctx, state.settings.theme);
        let message_tx = Arc::new(Mutex::new(None));

        let ai_settings_draft = state.settings.llm.clone();
        let ai_settings_local_mode = state
            .settings
            .llm
            .base_url
            .as_deref()
            .is_some_and(|url| url.contains("localhost") || url.contains("127.0.0.1"));
        Self {
            state,
            message_tx,
            show_ai_settings: false,
            ai_settings_draft,
            ai_settings_local_mode,
            show_about: false,
        }
    }

    fn setup_style(ctx: &egui::Context) {
        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            ctx.style_mut_of(theme, |style| {
                style.spacing.item_spacing = egui::vec2(8.0, 6.0);
                style.spacing.button_padding = egui::vec2(12.0, 6.0);
                style.spacing.interact_size = egui::vec2(100.0, 28.0);
            });
        }
    }

    fn show_about_popup(ctx: &egui::Context, open: &mut bool) {
        let mut close_clicked = false;
        egui::Window::new("About MP3 Tagger")
            .open(open)
            .collapsible(false)
            .resizable(true)
            .default_size([760.0, 560.0])
            .min_size([600.0, 460.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(112, 78, 184))
                        .inner_margin(egui::Margin::same(5.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("♫")
                                    .size(28.0)
                                    .color(egui::Color32::WHITE),
                            );
                        });
                    ui.add_space(14.0);
                    ui.heading(egui::RichText::new("MP3 Tagger").size(34.0));
                });

                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(
                        "MP3 Tagger is a modern, easy-to-use application for editing audio metadata and cover artwork.",
                    )
                    .size(18.0),
                );
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("This release was provided by Borneel B. Phukan.")
                        .size(16.0),
                );
                ui.label(
                    egui::RichText::new("Copyright © 2026-Present Borneel B. Phukan.")
                        .size(16.0),
                );

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    let link_color = egui::Color32::from_rgb(0, 160, 245);
                    for label in ["Credits", "Website", "Release Notes"] {
                        ui.label(egui::RichText::new(label).size(18.0).color(link_color));
                        ui.add_space(12.0);
                    }
                });

                ui.add_space(26.0);
                ui.horizontal(|ui| {
                    ui.heading("Version Information");
                    ui.label("◆");
                });
                ui.add_space(8.0);
                egui::Grid::new("about_version_info")
                    .num_columns(2)
                    .spacing([28.0, 8.0])
                    .show(ui, |ui| {
                        let value_color = ui.visuals().weak_text_color();
                        for (label, value) in [
                            ("Version:", env!("CARGO_PKG_VERSION").to_string()),
                            (
                                "Environment:",
                                format!("OS: {}; Arch: {}", std::env::consts::OS, std::env::consts::ARCH),
                            ),
                            ("Developer:", "Borneel B. Phukan".to_string()),
                        ] {
                            ui.label(egui::RichText::new(label).size(16.0));
                            ui.label(egui::RichText::new(value).size(16.0).color(value_color));
                            ui.end_row();
                        }
                    });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
            });
        if close_clicked {
            *open = false;
        }
    }

    fn apply_theme(ctx: &egui::Context, theme: ThemePreference) {
        ctx.set_theme(match theme {
            ThemePreference::Light => egui::ThemePreference::Light,
            ThemePreference::Dark => egui::ThemePreference::Dark,
            ThemePreference::System => egui::ThemePreference::System,
        });
    }
}

impl eframe::App for Mp3TaggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_theme(ctx, self.state.settings.theme);
        // Process messages
        if let Ok(mut lock) = self.message_tx.lock() {
            if let Some(msg) = lock.take() {
                match msg {
                    AppMessage::LoadSongs(paths) => self.state.add_songs(paths),
                    AppMessage::SongLoaded(idx, result) => {
                        if let Ok(song) = result {
                            if idx < self.state.songs.len() {
                                self.state.songs[idx] = song;
                            }
                        }
                    }
                    AppMessage::MetadataFetched(idx, result) => {
                        self.state.complete_metadata_fetch(idx, result);
                    }
                    AppMessage::SaveSong(idx) => {
                        if let Err(e) = crate::services::metadata::MetadataService::write_metadata(
                            &mut self.state.songs[idx],
                        ) {
                            self.state.error_message = Some(e.to_string());
                        }
                    }
                    AppMessage::DeleteSong(idx) => self.state.delete_song(idx),
                    AppMessage::ChangeView(view) => self.state.current_view = view,
                    AppMessage::UpdateMetadata(idx, field, value) => {
                        self.state.update_metadata_field(idx, field, value);
                    }
                    AppMessage::ToggleFillDataButton(_idx) => {
                        // Handled in detail view
                    }
                    AppMessage::SetError(msg) => self.state.error_message = Some(msg),
                    AppMessage::ClearError => self.state.error_message = None,
                    AppMessage::SettingsSaved => {
                        if let Err(e) = self.state.save_settings() {
                            self.state.error_message = Some(e.to_string());
                        }
                    }
                    AppMessage::TestLlmConnection => {
                        // Handled in settings view
                    }
                    AppMessage::LlmTestResult(result) => match result {
                        Ok(_) => {
                            self.state.error_message =
                                Some("Connection test successful!".to_string())
                        }
                        Err(e) => {
                            self.state.error_message =
                                Some(format!("Connection test failed: {}", e))
                        }
                    },
                }
            }
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Add Songs").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter(
                                "Audio Files",
                                &["mp3", "flac", "ogg", "m4a", "wav", "aiff", "ape"],
                            )
                            .pick_files()
                        {
                            self.state.add_songs(paths);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("AI", |ui| {
                    if ui.button("LLM Settings").clicked() {
                        self.ai_settings_draft = self.state.settings.llm.clone();
                        self.ai_settings_local_mode = self
                            .state
                            .settings
                            .llm
                            .base_url
                            .as_deref()
                            .is_some_and(|url| {
                                url.contains("localhost") || url.contains("127.0.0.1")
                            });
                        self.show_ai_settings = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    const THEME_MENU_WIDTH: f32 = 150.0;

                    ui.menu_button("Theme", |ui| {
                        for (preference, label) in [
                            (ThemePreference::System, "Default"),
                            (ThemePreference::Dark, "Dark"),
                            (ThemePreference::Light, "Light"),
                        ] {
                            if ui
                                .selectable_value(&mut self.state.settings.theme, preference, label)
                                .clicked()
                            {
                                if let Err(error) = self.state.save_settings() {
                                    self.state.error_message = Some(error);
                                }
                                ui.close_menu();
                            }
                        }

                        // Without this the submenu is only as wide as its
                        // longest label, which looks cramped.
                        let left = ui.min_rect().left();
                        ui.expand_to_include_x(left + THEME_MENU_WIDTH);
                    });
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| match &self.state.current_view {
                        View::Grid => ui.label("📚 Library"),
                        View::Detail(_) => ui.label("🎵 Song Details"),
                    },
                );
            });
        });

        // Main content
        match self.state.current_view {
            View::Grid => {
                crate::ui::grid_view::GridView::show(ctx, &mut self.state, &self.message_tx)
            }
            View::Detail(_) => {
                crate::ui::detail_view::DetailView::show(ctx, &mut self.state, &self.message_tx)
            }
        }

        if self.show_ai_settings {
            crate::ui::settings_view::SettingsView::show_popup(
                ctx,
                &mut self.state,
                &mut self.ai_settings_draft,
                &mut self.ai_settings_local_mode,
                &mut self.show_ai_settings,
                &self.message_tx,
            );
        }

        if self.show_about {
            Self::show_about_popup(ctx, &mut self.show_about);
        }

        // Request repaint if fetching
        if self.state.is_fetching {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save settings on exit
        let _ = self.state.save_settings();
    }
}

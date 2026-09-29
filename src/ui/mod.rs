pub mod detail_view;
pub mod grid_view;
pub mod settings_view;
pub mod state;

use crate::ui::state::{AppMessage, AppState, View};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct Mp3TaggerApp {
    state: AppState,
    message_tx: Arc<Mutex<Option<AppMessage>>>,
}

impl Mp3TaggerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Use system fonts (font loading disabled for now due to placeholder font file)
        // Set up visual style
        Self::setup_style(&cc.egui_ctx);

        let state = AppState::new().expect("Failed to initialize app state");
        let message_tx = Arc::new(Mutex::new(None));

        Self { state, message_tx }
    }

    fn setup_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = egui::Color32::from_rgb(28, 28, 30);
        style.visuals.window_fill = egui::Color32::from_rgb(38, 38, 40);
        style.visuals.extreme_bg_color = egui::Color32::from_rgb(48, 48, 50);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(48, 48, 50);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(58, 58, 60);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(68, 68, 70);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(78, 78, 80);
        style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(78, 78, 80);
        
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size = egui::vec2(100.0, 28.0);
        
        ctx.set_style(style);
    }
}

impl eframe::App for Mp3TaggerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                        if let Err(e) = crate::services::metadata::MetadataService::write_metadata(&mut self.state.songs[idx]) {
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
                    AppMessage::LlmTestResult(result) => {
                        match result {
                            Ok(_) => self.state.error_message = Some("Connection test successful!".to_string()),
                            Err(e) => self.state.error_message = Some(format!("Connection test failed: {}", e)),
                        }
                    }
                }
            }
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Add Songs").clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .add_filter("Audio Files", &["mp3", "flac", "ogg", "m4a", "wav", "aiff", "ape"])
                            .pick_files()
                        {
                            self.state.add_songs(paths);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Add Folder").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            let mut audio_files = Vec::new();
                            if let Ok(entries) = std::fs::read_dir(&folder) {
                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    if let Some(ext) = path.extension() {
                                        let ext_str = ext.to_string_lossy().to_lowercase();
                                        if matches!(ext_str.as_str(), "mp3" | "flac" | "ogg" | "m4a" | "wav" | "aiff" | "ape") {
                                            audio_files.push(path);
                                        }
                                    }
                                }
                            }
                            self.state.add_songs(audio_files);
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
                        self.state.current_view = View::Settings;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.state.error_message = Some("MP3 Tagger v0.1.0\nNative Rust MP3 Metadata Editor".to_string());
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match &self.state.current_view {
                        View::Grid => ui.label("📚 Library"),
                        View::Detail(_) => ui.label("🎵 Song Details"),
                        View::Settings => ui.label("⚙️ Settings"),
                    }
                });
            });
        });

        // Main content
        match self.state.current_view {
            View::Grid => crate::ui::grid_view::GridView::show(ctx, &mut self.state, &self.message_tx),
            View::Detail(_) => crate::ui::detail_view::DetailView::show(ctx, &mut self.state, &self.message_tx),
            View::Settings => crate::ui::settings_view::SettingsView::show(ctx, &mut self.state, &self.message_tx),
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
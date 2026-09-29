use crate::ui::state::{AppMessage, AppState};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::sync::{Arc, Mutex};

pub struct GridView;

impl GridView {
    pub fn show(ctx: &egui::Context, state: &mut AppState, _tx: &Arc<Mutex<Option<AppMessage>>>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MP3 Tagger - Song Library");
            ui.separator();

            // Toolbar
            ui.horizontal(|ui| {
                if ui.button("➕ Add Songs").clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Audio Files", &["mp3", "flac", "ogg", "m4a", "wav", "aiff", "ape"])
                        .pick_files()
                    {
                        state.add_songs(paths);
                    }
                }

                if ui.button("➕ Add Folder").clicked() {
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
                        state.add_songs(audio_files);
                    }
                }

                ui.separator();

                if ui.button("🗑️ Clear All").clicked() {
                    state.songs.clear();
                    state.pending_changes = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} songs", state.songs.len()));
                });
            });

            ui.separator();

            if state.songs.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("No songs added yet.");
                    ui.label("Click 'Add Songs' or 'Add Folder' to get started.");
                });
                return;
            }

            // Collect indices that were double-clicked
            let mut double_clicked_idx = None;

            // Song table
            let available_height = ui.available_height();
            egui::ScrollArea::vertical()
                .max_height(available_height)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(40.0))   // #
                        .column(Column::auto().at_least(200.0))  // Title
                        .column(Column::auto().at_least(150.0))  // Artist
                        .column(Column::auto().at_least(150.0))  // Album
                        .column(Column::auto().at_least(60.0))   // Year
                        .column(Column::auto().at_least(100.0))  // Genre
                        .column(Column::auto().at_least(80.0))   // Track
                        .column(Column::auto().at_least(100.0))  // Status
                        .header(20.0, |mut header| {
                            header.col(|ui| { ui.strong("#"); });
                            header.col(|ui| { ui.strong("Title"); });
                            header.col(|ui| { ui.strong("Artist"); });
                            header.col(|ui| { ui.strong("Album"); });
                            header.col(|ui| { ui.strong("Year"); });
                            header.col(|ui| { ui.strong("Genre"); });
                            header.col(|ui| { ui.strong("Track"); });
                            header.col(|ui| { ui.strong("Status"); });
                        })
                        .body(|mut body| {
                            for (idx, song) in state.songs.iter().enumerate() {
                                body.row(28.0, |mut row| {
                                    row.col(|ui| {
                                        ui.label(format!("{}", idx + 1));
                                    });
                                    row.col(|ui| {
                                        let title = if song.metadata.title.is_empty() {
                                            song.file_name()
                                        } else {
                                            song.metadata.title.clone()
                                        };
                                        ui.label(title);
                                    });
                                    row.col(|ui| {
                                        ui.label(&song.metadata.artist);
                                    });
                                    row.col(|ui| {
                                        ui.label(&song.metadata.album);
                                    });
                                    row.col(|ui| {
                                        ui.label(&song.metadata.year);
                                    });
                                    row.col(|ui| {
                                        ui.label(&song.metadata.genre);
                                    });
                                    row.col(|ui| {
                                        let track = if song.metadata.track_number.is_empty() {
                                            "—".to_string()
                                        } else if song.metadata.total_tracks.is_empty() {
                                            song.metadata.track_number.clone()
                                        } else {
                                            format!("{}/{}", song.metadata.track_number, song.metadata.total_tracks)
                                        };
                                        ui.label(track);
                                    });
                                    row.col(|ui| {
                                        if song.has_changes {
                                            ui.colored_label(egui::Color32::YELLOW, "● Modified");
                                        } else if song.metadata.is_empty() {
                                            ui.colored_label(egui::Color32::GRAY, "No metadata");
                                        } else {
                                            ui.colored_label(egui::Color32::GREEN, "✓ Saved");
                                        }
                                    });

                                    // Double-click to open detail
                                    let response = row.response();
                                    if response.double_clicked() {
                                        double_clicked_idx = Some(idx);
                                    }
                                });
                            }
                        });
                });
            
            // Handle double-click after the table
            if let Some(idx) = double_clicked_idx {
                state.open_song_detail(idx);
            }
        });
    }
}
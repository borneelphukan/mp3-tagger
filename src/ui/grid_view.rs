use crate::ui::state::{AppMessage, AppState, LibraryViewMode};
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
                        .add_filter(
                            "Audio Files",
                            &["mp3", "flac", "ogg", "m4a", "wav", "aiff", "ape"],
                        )
                        .pick_files()
                    {
                        state.add_songs(paths);
                    }
                }

                ui.separator();

                if ui.button("🗑️ Clear All").clicked() {
                    state.songs.clear();
                    state.pending_changes = true;
                }

                ui.separator();

                // View mode toggle
                ui.label("View:");
                let list_selected = state.library_view_mode == LibraryViewMode::List;
                let grid_selected = state.library_view_mode == LibraryViewMode::Grid;

                if ui.selectable_label(list_selected, "📋 List").clicked() {
                    state.library_view_mode = LibraryViewMode::List;
                }
                if ui.selectable_label(grid_selected, "🔲 Grid").clicked() {
                    state.library_view_mode = LibraryViewMode::Grid;
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
                    ui.label("Click 'Add Songs' to get started.");
                });
                return;
            }

            // Collect indices that were clicked
            let mut clicked_idx = None;

            let available_height = ui.available_height();
            egui::ScrollArea::vertical()
                .max_height(available_height)
                .show(ui, |ui| {
                    match state.library_view_mode {
                        LibraryViewMode::List => {
                            // List view - single column with more details
                            egui_extras::TableBuilder::new(ui)
                                .striped(true)
                                .resizable(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::remainder())
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("Songs");
                                    });
                                })
                                .body(|mut body| {
                                    for (idx, song) in state.songs.iter().enumerate() {
                                        body.row(40.0, |mut row| {
                                            row.col(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("{}.", idx + 1));
                                                    let title = if song.metadata.title.is_empty() {
                                                        song.file_name()
                                                    } else {
                                                        format!(
                                                            "{} - {}",
                                                            song.metadata.artist,
                                                            song.metadata.title
                                                        )
                                                    };
                                                    if ui.selectable_label(false, title).clicked() {
                                                        clicked_idx = Some(idx);
                                                    }
                                                    ui.separator();
                                                    if !song.metadata.album.is_empty() {
                                                        ui.label(format!(
                                                            "💿 {}",
                                                            song.metadata.album
                                                        ));
                                                        ui.separator();
                                                    }
                                                    if !song.metadata.year.is_empty() {
                                                        ui.label(format!(
                                                            "📅 {}",
                                                            song.metadata.year
                                                        ));
                                                        ui.separator();
                                                    }
                                                    if !song.metadata.genre.is_empty() {
                                                        ui.label(format!(
                                                            "🎵 {}",
                                                            song.metadata.genre
                                                        ));
                                                        ui.separator();
                                                    }
                                                    if song.has_changes {
                                                        ui.colored_label(
                                                            egui::Color32::YELLOW,
                                                            "● Modified",
                                                        );
                                                    } else if song.metadata.is_empty() {
                                                        ui.colored_label(
                                                            egui::Color32::GRAY,
                                                            "No metadata",
                                                        );
                                                    } else {
                                                        ui.colored_label(
                                                            egui::Color32::GREEN,
                                                            "✓ Saved",
                                                        );
                                                    }
                                                });
                                            });
                                        });
                                    }
                                });
                        }
                        LibraryViewMode::Grid => {
                            // Grid view - table with columns
                            TableBuilder::new(ui)
                                .striped(true)
                                .resizable(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::auto().at_least(40.0)) // #
                                .column(Column::auto().at_least(200.0)) // Title
                                .column(Column::auto().at_least(150.0)) // Artist
                                .column(Column::auto().at_least(150.0)) // Album
                                .column(Column::auto().at_least(60.0)) // Year
                                .column(Column::auto().at_least(100.0)) // Genre
                                .column(Column::auto().at_least(80.0)) // Track
                                .column(Column::auto().at_least(100.0)) // Status
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("#");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Title");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Artist");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Album");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Year");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Genre");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Track");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Status");
                                    });
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
                                                if ui.selectable_label(false, title).clicked() {
                                                    clicked_idx = Some(idx);
                                                }
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
                                                let track = if song.metadata.track_number.is_empty()
                                                {
                                                    "—".to_string()
                                                } else if song.metadata.total_tracks.is_empty() {
                                                    song.metadata.track_number.clone()
                                                } else {
                                                    format!(
                                                        "{}/{}",
                                                        song.metadata.track_number,
                                                        song.metadata.total_tracks
                                                    )
                                                };
                                                ui.label(track);
                                            });
                                            row.col(|ui| {
                                                if song.has_changes {
                                                    ui.colored_label(
                                                        egui::Color32::YELLOW,
                                                        "● Modified",
                                                    );
                                                } else if song.metadata.is_empty() {
                                                    ui.colored_label(
                                                        egui::Color32::GRAY,
                                                        "No metadata",
                                                    );
                                                } else {
                                                    ui.colored_label(
                                                        egui::Color32::GREEN,
                                                        "✓ Saved",
                                                    );
                                                }
                                            });
                                        });
                                    }
                                });
                        }
                    }
                });

            // Handle click after the table
            if let Some(idx) = clicked_idx {
                state.open_song_detail(idx);
            }
        });
    }
}

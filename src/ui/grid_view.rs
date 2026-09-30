use crate::ui::state::{AppMessage, AppState, LibraryViewMode};
use eframe::egui;
use egui_extras::Column;
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
                            // List view: metadata columns for scanning the library.
                            egui_extras::TableBuilder::new(ui)
                                .striped(true)
                                .resizable(false)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::auto().at_least(40.0))
                                .column(Column::auto().at_least(200.0))
                                .column(Column::auto().at_least(150.0))
                                .column(Column::auto().at_least(150.0))
                                .column(Column::auto().at_least(60.0))
                                .column(Column::auto().at_least(100.0))
                                .column(Column::auto().at_least(80.0))
                                .header(20.0, |mut header| {
                                    for label in
                                        ["#", "Title", "Artist", "Album", "Year", "Genre", "Track"]
                                    {
                                        header.col(|ui| {
                                            ui.strong(label);
                                        });
                                    }
                                })
                                .body(|mut body| {
                                    for (idx, song) in state.songs.iter().enumerate() {
                                        body.row(28.0, |mut row| {
                                            row.col(|ui| {
                                                ui.label(format!("{}", idx + 1));
                                                track_list_row_click(ui, idx, 0, &mut clicked_idx);
                                            });
                                            row.col(|ui| {
                                                let title = if song.metadata.title.is_empty() {
                                                    song.file_name()
                                                } else {
                                                    song.metadata.title.clone()
                                                };
                                                ui.label(title);
                                                track_list_row_click(ui, idx, 1, &mut clicked_idx);
                                            });
                                            row.col(|ui| {
                                                ui.label(&song.metadata.artist);
                                                track_list_row_click(ui, idx, 2, &mut clicked_idx);
                                            });
                                            row.col(|ui| {
                                                ui.label(&song.metadata.album);
                                                track_list_row_click(ui, idx, 3, &mut clicked_idx);
                                            });
                                            row.col(|ui| {
                                                ui.label(&song.metadata.year);
                                                track_list_row_click(ui, idx, 4, &mut clicked_idx);
                                            });
                                            row.col(|ui| {
                                                ui.label(&song.metadata.genre);
                                                track_list_row_click(ui, idx, 5, &mut clicked_idx);
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
                                                track_list_row_click(ui, idx, 6, &mut clicked_idx);
                                            });
                                        });
                                    }
                                });
                        }
                        LibraryViewMode::Grid => {
                            // Desktop style: uniform song icons laid out in rows.
                            let tile_width = 112.0;
                            let columns =
                                (ui.available_width() / tile_width).floor().max(1.0) as usize;
                            let songs: Vec<(usize, String, String, Option<Vec<u8>>)> = state
                                .songs
                                .iter()
                                .enumerate()
                                .map(|(idx, song)| {
                                    let title = if song.metadata.title.is_empty() {
                                        song.file_name()
                                    } else {
                                        song.metadata.title.clone()
                                    };
                                    (
                                        idx,
                                        title,
                                        song.path.display().to_string(),
                                        song.metadata.album_art.clone(),
                                    )
                                })
                                .collect();
                            egui::Grid::new("song_desktop_grid")
                                .num_columns(columns)
                                .spacing(egui::vec2(12.0, 18.0))
                                .show(ui, |ui| {
                                    for (idx, title, path, album_art) in &songs {
                                        ui.vertical_centered(|ui| {
                                            let icon = if let Some(bytes) = album_art {
                                                state
                                                    .album_art_texture(ui.ctx(), path, bytes)
                                                    .map(|texture| {
                                                        ui.add_sized(
                                                            [72.0, 72.0],
                                                            egui::Image::new(&texture)
                                                                .fit_to_exact_size(egui::vec2(
                                                                    72.0, 72.0,
                                                                )),
                                                        )
                                                        .interact(egui::Sense::click())
                                                    })
                                                    .unwrap_or_else(|| {
                                                        ui.add_sized(
                                                            [72.0, 72.0],
                                                            egui::Button::new(
                                                                egui::RichText::new("♫").size(42.0),
                                                            ),
                                                        )
                                                    })
                                            } else {
                                                ui.add_sized(
                                                    [72.0, 72.0],
                                                    egui::Button::new(
                                                        egui::RichText::new("♫").size(42.0),
                                                    ),
                                                )
                                            };
                                            if icon.clicked() {
                                                clicked_idx = Some(*idx);
                                            }
                                            let label = ui.add_sized(
                                                [tile_width, 36.0],
                                                egui::Label::new(title).truncate().wrap(),
                                            );
                                            if label.interact(egui::Sense::click()).clicked() {
                                                clicked_idx = Some(*idx);
                                            }
                                        });
                                        if (*idx + 1) % columns == 0 {
                                            ui.end_row();
                                        }
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

fn track_list_row_click(
    ui: &mut egui::Ui,
    song_index: usize,
    column_index: usize,
    clicked_idx: &mut Option<usize>,
) {
    let cell = ui.interact(
        ui.max_rect(),
        ui.id().with(("song_list_cell", song_index, column_index)),
        egui::Sense::click(),
    );
    if cell.clicked() {
        *clicked_idx = Some(song_index);
    }
}

use crate::models::SongMetadata;
use crate::services::llm::get_tokio_handle;
use crate::ui::state::{AppMessage, AppState, View};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct DetailView;

impl DetailView {
    pub fn show(ctx: &egui::Context, state: &mut AppState, tx: &Arc<Mutex<Option<AppMessage>>>) {
        let idx = match state.current_view {
            View::Detail(i) => i,
            _ => return,
        };

        if idx >= state.songs.len() {
            state.current_view = View::Grid;
            return;
        }

        // Clone all data we need upfront, then drop the song reference
        let song = &state.songs[idx];
        let has_changes = song.has_changes;
        let display_name = song.display_name();
        let file_name = song.file_name();
        let title = song.metadata.title.clone();
        let artist = song.metadata.artist.clone();
        let title_for_async = title.clone();
        let artist_for_async = artist.clone();
        let can_fill = song.metadata.has_required_for_llm()
            && state.settings.llm.is_configured()
            && !state.is_fetching;
        let album_art = song.metadata.album_art.clone();
        let album_art_mime = song.metadata.album_art_mime.clone();
        let path_display = song.path.display().to_string();
        // Lyrics use a multiline editor below; include every other tag here.
        let fields: Vec<(usize, String, String)> = SongMetadata::field_names()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != 15)
            .map(|(i, name)| (i, name.to_string(), song.metadata.get_field(i).to_string()))
            .collect();
        // Drop the song reference here
        let _ = song;

        egui::TopBottomPanel::top("detail_top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let back = ui.button("\u{2039} Back to Library");
                if back.clicked() {
                    if has_changes {
                        state.confirm_discard_changes = true;
                    } else {
                        state.go_back();
                    }
                }

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(&display_name);
                    ui.small(&file_name);
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fill_enabled = can_fill && !state.is_fetching;
                    let fill_label = if state.is_fetching && state.fetch_index == Some(idx) {
                        "⏳ Fetching..."
                    } else {
                        "🔮 Fill Data"
                    };

                    if ui
                        .add_enabled(fill_enabled, egui::Button::new(fill_label))
                        .clicked()
                    {
                        state.start_metadata_fetch(idx);
                        let llm_service = state.llm_service.clone();
                        let settings = state.settings.llm.clone();
                        let tx_clone = tx.clone();
                        let fetch_title = title_for_async.clone();
                        let fetch_artist = artist_for_async.clone();

                        if let Some(handle) = get_tokio_handle() {
                            handle.spawn(async move {
                                let result = llm_service
                                    .fetch_metadata(&settings, &fetch_title, &fetch_artist)
                                    .await
                                    .map_err(|e| e.to_string());
                                let result = match result {
                                    Ok(mut metadata) => {
                                        let art_title = if metadata.title.is_empty() {
                                            &fetch_title
                                        } else {
                                            &metadata.title
                                        };
                                        let art_artist = if metadata.artist.is_empty() {
                                            &fetch_artist
                                        } else {
                                            &metadata.artist
                                        };
                                        if let Ok((bytes, mime)) =
                                            crate::services::artwork::fetch_cover_art(
                                                art_title,
                                                art_artist,
                                                &metadata.album,
                                            )
                                            .await
                                        {
                                            metadata.album_art = Some(bytes);
                                            metadata.album_art_mime = Some(mime);
                                        }
                                        Ok(metadata)
                                    }
                                    Err(error) => Err(error),
                                };
                                if let Ok(mut lock) = tx_clone.lock() {
                                    *lock = Some(AppMessage::MetadataFetched(idx, result));
                                }
                            });
                        }
                    }

                    if !can_fill && !state.settings.llm.is_configured() {
                        ui.small("⚠ Configure LLM in Settings");
                    } else if !can_fill && (title.is_empty() || artist.is_empty()) {
                        ui.small("⚠ Enter Title and Artist first");
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("detail_save_bar").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    state.clear_current_metadata();
                }
                if ui
                    .add_enabled(has_changes, egui::Button::new("💾 Save"))
                    .clicked()
                {
                    match state.save_current_song() {
                        Ok(()) => state.go_back(),
                        Err(e) => state.error_message = Some(e),
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let content_width = ui.available_width();
            let content_height = ui.available_height();
            ui.horizontal(|ui| {
                // Left panel - Metadata fields
                ui.allocate_ui_with_layout(
                    egui::vec2(content_width * 0.65, content_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            Self::show_metadata_fields(ui, state, idx, &fields);
                        });
                    },
                );

                ui.separator();

                // Right panel - Album artwork
                ui.allocate_ui_with_layout(
                    egui::vec2(content_width * 0.35, content_height),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        Self::show_album_art(
                            ui,
                            state,
                            idx,
                            &album_art,
                            &album_art_mime,
                            &path_display,
                        );
                    },
                );
            });
        });

        // Error toast
        let error_msg = state.error_message.clone();
        if let Some(error) = error_msg {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, &error);
                    ui.separator();
                    if ui.button("OK").clicked() {
                        state.error_message = None;
                    }
                });
        }

        if state.confirm_discard_changes {
            egui::Window::new("Unsaved changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes. Any changes made will be lost. Go back?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            state.confirm_discard_and_go_back();
                        }
                        if ui.button("No").clicked() {
                            state.confirm_discard_changes = false;
                        }
                    });
                });
        }
    }

    fn show_metadata_fields(
        ui: &mut egui::Ui,
        state: &mut AppState,
        idx: usize,
        fields: &[(usize, String, String)],
    ) {
        ui.vertical(|ui| {
            for (field_idx, field_name, current_value) in fields {
                ui.strong(field_name);
                let mut edited_value = current_value.clone();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut edited_value).desired_width(f32::INFINITY),
                );
                if response.changed() && edited_value != *current_value {
                    state.update_metadata_field(idx, *field_idx, edited_value);
                }
                if (*field_idx == 0 || *field_idx == 1) && current_value.is_empty() {
                    ui.small("Required for LLM lookup");
                }
                ui.add_space(5.0);
            }
            ui.strong("Lyrics");
            let mut lyrics_text = state
                .songs
                .get(idx)
                .map(|song| song.metadata.lyrics.clone())
                .unwrap_or_default();
            if ui
                .add(
                    egui::TextEdit::multiline(&mut lyrics_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(5),
                )
                .changed()
            {
                state.update_metadata_field(idx, 15, lyrics_text);
            }
        });
    }

    fn show_album_art(
        ui: &mut egui::Ui,
        state: &mut AppState,
        idx: usize,
        album_art: &Option<Vec<u8>>,
        _album_art_mime: &Option<String>,
        path_display: &str,
    ) {
        let texture = album_art
            .as_ref()
            .and_then(|bytes| state.album_art_texture(ui.ctx(), path_display, bytes));
        ui.vertical_centered(|ui| {
            ui.heading("Album Artwork");
            ui.separator();

            if let Some(texture) = texture {
                ui.add(egui::Image::new(&texture).fit_to_exact_size(egui::vec2(300.0, 300.0)));
            } else if album_art.is_some() {
                ui.label("Failed to load artwork image");
            } else {
                ui.add_space(100.0);
                ui.label("No album artwork");
            }

            ui.add_space(20.0);
            if ui.button("📁 Load Artwork").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp"])
                    .pick_file()
                {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let mime = match path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .as_str()
                            {
                                "png" => "image/png",
                                "gif" => "image/gif",
                                "bmp" => "image/bmp",
                                _ => "image/jpeg",
                            };
                            let song = &mut state.songs[idx];
                            song.metadata.album_art = Some(bytes);
                            song.metadata.album_art_mime = Some(mime.to_string());
                            song.mark_changed();
                            state.pending_changes = true;
                        }
                        Err(error) => {
                            state.error_message = Some(format!("Could not load artwork: {error}"))
                        }
                    }
                }
            }
            if ui
                .add_enabled(album_art.is_some(), egui::Button::new("🗑️ Remove Artwork"))
                .clicked()
            {
                let song = &mut state.songs[idx];
                song.metadata.album_art = None;
                song.metadata.album_art_mime = None;
                song.mark_changed();
                state.pending_changes = true;
            }
        });
    }
}

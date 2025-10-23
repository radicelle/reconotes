use crate::RecogNotesApp;
use eframe::egui;

#[allow(clippy::too_many_lines)]
pub fn draw_ui(app: &mut RecogNotesApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Top bar: Title + Status
        ui.horizontal(|ui| {
            ui.heading("🎵 RecogNotes");

            ui.separator();

            if app.backend_connected {
                ui.colored_label(egui::Color32::GREEN, "● Connected");
            } else {
                ui.colored_label(egui::Color32::RED, "● Offline");
            }

            if app.recording {
                ui.colored_label(egui::Color32::RED, "● Recording");
            }

            ui.separator();

            // Backend URL control in top bar
            ui.label("Backend:");
            ui.text_edit_singleline(&mut app.backend_url);
            if ui.small_button("✓").clicked() {
                let backend_url = app.backend_url.clone();
                tokio::spawn(async move {
                    match crate::backend_client::check_health(&backend_url).await {
                        Ok(()) => log::info!("✓ Backend OK"),
                        Err(e) => log::error!("✗ {e}"),
                    }
                });
                app.backend_connected = true;
            }
        });

        ui.separator();

        // Voice profile and device controls - side by side
        ui.horizontal(|ui| {
            // Voice profile selector
            ui.label("Voice Profile:");

            let available_profiles = [
                "no_profile",
                "soprano",
                "mezzo",
                "alto",
                "tenor",
                "baritone",
                "bass",
            ];

            egui::ComboBox::from_id_source("voice_profile_combo")
                .selected_text(app.selected_profile.as_str())
                .show_ui(ui, |ui| {
                    for profile in &available_profiles {
                        ui.selectable_value(
                            &mut app.selected_profile,
                            (*profile).to_string(),
                            *profile,
                        );
                    }
                });

            // Show profile info
            if app.selected_profile != "no_profile" {
                let profile_info = match app.selected_profile.as_str() {
                    "soprano" => "C4-C6 (261-1047 Hz)",
                    "mezzo" => "A3-A5 (220-880 Hz)",
                    "alto" => "F3-F5 (175-698 Hz)",
                    "tenor" => "C3-C5 (131-523 Hz)",
                    "baritone" => "A2-A4 (110-440 Hz)",
                    "bass" => "C2-C4 (65-261 Hz)",
                    _ => "",
                };
                ui.label(
                    egui::RichText::new(profile_info)
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            }

            ui.separator();

            // Input device selector
            ui.label("Input device:");
            let input_devices = crate::audio::AudioManager::get_input_devices();

            egui::ComboBox::from_id_source("input_device_combo")
                .selected_text(app.selected_input_device.as_deref().unwrap_or("Default"))
                .show_ui(ui, |ui| {
                    // Always show "Default" option
                    ui.selectable_value(&mut app.selected_input_device, None, "Default");

                    // Show all other devices
                    for device in input_devices {
                        if device != "Default" {
                            ui.selectable_value(
                                &mut app.selected_input_device,
                                Some(device.clone()),
                                device,
                            );
                        }
                    }
                });
        });

        ui.separator();

        // Control bar
        ui.horizontal(|ui| {
            if ui
                .button(if app.recording {
                    "⏹ Stop"
                } else {
                    "🎤 Record"
                })
                .clicked()
            {
                if app.recording {
                    app.stop_recording();
                } else {
                    app.start_recording();
                }
            }

            if ui.button("Clear").clicked() {
                app.detected_notes.clear();
                app.detected_notes_history.clear();
                app.last_error = None;
            }
        });

        // Error display
        if let Some(error) = &app.last_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        ui.separator();

        // MAIN AREA: Just notes display at bottom
        let available_width = ui.available_width();
        let available_height = ui.available_height();

        let notes_response = ui.allocate_rect(
            egui::Rect::from_min_size(
                ui.cursor().min,
                egui::Vec2::new(available_width, available_height),
            ),
            egui::Sense::hover(),
        );

        // Draw notes spectrum with vertical bars and fade effect
        crate::visualization::draw_vertical_bars_with_fade(
            ui,
            &app.detected_notes,
            &app.notes_with_timestamps,
            notes_response.rect,
            &app.selected_profile,
        );
    });
}

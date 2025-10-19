use crate::RecogNotesApp;
use eframe::egui;

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
        });

        ui.separator();

        // Device and volume controls
        ui.horizontal(|ui| {
            ui.label("Input device:");
            let input_devices = crate::audio::AudioManager::get_input_devices();
            let selected_input = app.selected_input_device.clone().unwrap_or_else(|| "Default".to_string());
            
            egui::ComboBox::from_id_source("input_device_selector")
                .selected_text(&selected_input)
                .show_ui(ui, |ui| {
                    for device in input_devices {
                        if ui.selectable_label(selected_input == device, &device).clicked() {
                            app.selected_input_device = Some(device);
                        }
                    }
                });

            ui.separator();
            
            ui.label("Volume:");
            ui.add(egui::Slider::new(&mut app.input_volume, 0.0..=1.0)
                .text("vol")
                .step_by(0.05)
                .show_value(true));
        });

        ui.separator();

        // Control bar
        ui.horizontal(|ui| {
            ui.label("Session:");
            ui.text_edit_singleline(&mut app.session_title);
            
            ui.separator();
            
            ui.label("BPM:");
            ui.add(egui::Slider::new(&mut app.bpm, 30..=300).text("bpm").fixed_decimals(0).show_value(true));
            
            ui.checkbox(&mut app.use_metronome, "Metronome");
            
            ui.separator();
            
            if ui.button(if app.recording {
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
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
        }

        ui.separator();

        // MAIN AREA: Just notes display at bottom
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        
        let notes_response = ui.allocate_rect(
            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(available_width, available_height)),
            egui::Sense::hover(),
        );
        
        // Draw notes spectrum with vertical bars
        crate::visualization::draw_vertical_bars(ui, &app.detected_notes, notes_response.rect);
        
        // Compact backend info at very bottom
        ui.horizontal(|ui| {
            ui.small("Backend:");
            ui.text_edit_singleline(&mut app.backend_url);
            if ui.small_button("✓").clicked() {
                let backend_url = app.backend_url.clone();
                tokio::spawn(async move {
                    match crate::backend_client::check_health(&backend_url).await {
                        Ok(_) => log::info!("✓ Backend OK"),
                        Err(e) => log::error!("✗ {}", e),
                    }
                });
                app.backend_connected = true;
            }
        });
    });
}

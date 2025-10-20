use eframe::egui;
use crate::DetectedNote;
use std::time::Instant;

/// All possible musical notes to display
const ALL_NOTES: &[&str] = &[
    "C2", "C#2", "D2", "D#2", "E2", "F2", "F#2", "G2", "G#2", "A2", "A#2", "B2",
    "C3", "C#3", "D3", "D#3", "E3", "F3", "F#3", "G3", "G#3", "A3", "A#3", "B3",
    "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4", "G4", "G#4", "A4", "A#4", "B4",
    "C5", "C#5", "D5", "D#5", "E5", "F5", "F#5", "G5", "G#5", "A5", "A#5", "B5",
    "C6", "C#6", "D6", "D#6", "E6", "F6", "F#6", "G6", "G#6", "A6", "A#6", "B6",
    "C7", "C#7", "D7", "D#7", "E7", "F7", "F#7", "G7", "G#7", "A7", "A#7", "B7",
];

/// Get the note range for a voice profile
#[allow(dead_code)]
pub fn get_profile_range(profile: &str) -> Option<(usize, usize)> {
    match profile {
        "soprano" => Some((24, 48)),     // C4-C6
        "mezzo" => Some((21, 45)),       // A3-A5
        "alto" => Some((17, 41)),        // F3-F5
        "tenor" => Some((12, 36)),       // C3-C5
        "baritone" => Some((9, 33)),     // A2-A4
        "bass" => Some((0, 24)),         // C2-C4
        _ => None,
    }
}

/// Draw vertical bars for all notes with fade effect based on time
#[allow(clippy::too_many_lines)]
pub fn draw_vertical_bars_with_fade(
    ui: &egui::Ui,
    _detected_notes: &[DetectedNote],
    notes_with_timestamps: &[(DetectedNote, Instant)],
    rect: egui::Rect,
    selected_profile: &str,
) {
    let painter = ui.painter();
    
    // Draw background
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));
    
    // Get profile range for highlighting
    let profile_range = get_profile_range(selected_profile);
    
    let now = Instant::now();
    let fade_duration = std::time::Duration::from_millis(600);
    
    // Create a map of notes with their max intensity and fade factor
    let mut note_map: std::collections::HashMap<String, (f32, f32)> = std::collections::HashMap::new();
    
    for (note, timestamp) in notes_with_timestamps {
        let elapsed = now.saturating_duration_since(*timestamp);
        let fade_alpha = if elapsed < fade_duration {
            1.0 - (elapsed.as_secs_f32() / fade_duration.as_secs_f32())
        } else {
            0.0
        };
        
        // Store max intensity and max fade_alpha for this note
        note_map
            .entry(note.note.clone())
            .and_modify(|(intensity, alpha)| {
                if note.intensity > *intensity {
                    *intensity = note.intensity;
                }
                if fade_alpha > *alpha {
                    *alpha = fade_alpha;
                }
            })
            .or_insert((note.intensity, fade_alpha));
    }
    
    let num_notes = ALL_NOTES.len();
    #[allow(clippy::cast_precision_loss)]
    let bar_width = (rect.width() - 10.0) / num_notes as f32;
    let padding_left = 5.0;
    let padding_bottom = 30.0;
    let max_bar_height = rect.height() - padding_bottom - 5.0;
    
    // Draw each note bar
    for (idx, &note_name) in ALL_NOTES.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
        let x = (idx as f32).mul_add(bar_width, rect.min.x + padding_left);
        
        // Check if this note is within the selected profile range
        let in_profile_range = profile_range
            .is_some_and(|(start, end)| idx >= start && idx <= end);
        
        // Draw background track (empty bar) with different color if in profile range
        let bg_color = if in_profile_range {
            egui::Color32::from_rgb(90, 90, 110)  // Slightly brighter for profile range
        } else {
            egui::Color32::from_rgb(60, 60, 80)   // Normal background
        };
        
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x + 1.0, rect.max.y - padding_bottom),
                egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
            ),
            1.0,
            bg_color,
        );
        
        // Draw filled bar if note detected
        if let Some((intensity, fade_alpha)) = note_map.get(note_name) {
            let bar_height = max_bar_height * intensity;
            let base_color = intensity_to_color(*intensity);
            let faded_color = apply_fade_to_color(base_color, *fade_alpha);
            
            let bar_top = rect.max.y - padding_bottom - bar_height;
            
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x + 1.0, bar_top),
                    egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
                ),
                1.0,
                faded_color,
            );
            
            // Draw border with fade - thicker/brighter if in profile range
            let border_color = if in_profile_range {
                apply_fade_to_color(faded_color, 1.2)  // Slightly brighter
            } else {
                faded_color
            };
            
            let border_width = if in_profile_range { 2.0 } else { 1.0 };
            
            painter.rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(x + 1.0, bar_top),
                    egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
                ),
                0.0,
                egui::Stroke::new(border_width, border_color),
            );
        }
        
        // Draw note label at bottom
        let font_size = if num_notes > 48 { 7.0 } else { 9.0 };
        let label_color = if let Some((intensity, _)) = note_map.get(note_name) {
            if *intensity > 0.3 {
                intensity_to_color(*intensity)
            } else {
                egui::Color32::from_rgb(100, 100, 120)
            }
        } else if in_profile_range {
            egui::Color32::from_rgb(150, 150, 180)  // Highlight profile range notes
        } else {
            egui::Color32::from_rgb(100, 100, 120)
        };
        
        painter.text(
            egui::pos2(x + bar_width / 2.0, rect.max.y - 10.0),
            egui::Align2::CENTER_CENTER,
            note_name,
            egui::FontId::monospace(font_size),
            label_color,
        );
    }
    
    // Draw border
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 150)),
    );
}

/// Convert intensity value to color (brighter = more intense)
fn intensity_to_color(intensity: f32) -> egui::Color32 {
    let intensity = intensity.clamp(0.0, 1.0);
    
    if intensity >= 0.8 {
        egui::Color32::GREEN
    } else if intensity >= 0.6 {
        egui::Color32::from_rgb(255, 200, 0) // Yellow
    } else if intensity >= 0.4 {
        egui::Color32::from_rgb(100, 200, 255) // Light Blue
    } else {
        egui::Color32::from_rgb(150, 150, 200) // Light Gray
    }
}

/// Apply fade effect to a color by reducing its alpha
fn apply_fade_to_color(color: egui::Color32, alpha: f32) -> egui::Color32 {
    let alpha_clamp = alpha.clamp(0.0, 1.0);
    let r = color.r();
    let g = color.g();
    let b = color.b();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let a = (f32::from(color.a()) * alpha_clamp) as u8;
    
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Convert confidence value to color (legacy - still used for horizontal bars)
#[allow(dead_code)]
fn confidence_to_color(confidence: f32) -> egui::Color32 {
    let confidence = confidence.clamp(0.0, 1.0);
    
    if confidence >= 0.8 {
        egui::Color32::GREEN
    } else if confidence >= 0.6 {
        egui::Color32::from_rgb(255, 200, 0) // Yellow
    } else if confidence >= 0.5 {
        egui::Color32::from_rgb(100, 200, 255) // Light Blue
    } else {
        egui::Color32::from_rgb(150, 150, 150) // Gray
    }
}

/// Draw horizontal bars for notes in a compact panel (legacy)
#[allow(dead_code)]
pub fn draw_horizontal_bars(ui: &egui::Ui, notes: &[DetectedNote], rect: egui::Rect) {
    if notes.is_empty() {
        return;
    }
    
    // Sort notes by name
    let mut sorted_notes = notes.to_vec();
    sorted_notes.sort_by(|a, b| a.note.cmp(&b.note));
    
    let painter = ui.painter();
    
    let num_notes = sorted_notes.len();
    #[allow(clippy::cast_precision_loss)]
    let bar_height = ((rect.height() - 15.0) / num_notes as f32).clamp(20.0, 35.0);
    let padding_top = 8.0;
    let padding_left = 70.0;
    let padding_right = 50.0;
    let max_bar_width = rect.width() - padding_left - padding_right;
    
    // Draw each bar
    for (idx, note) in sorted_notes.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]
        let y = (idx as f32).mul_add(bar_height, rect.min.y + padding_top);
        
        // Only draw if within visible area
        if y + bar_height > rect.max.y {
            break;
        }
        
        let bar_width = (max_bar_width * note.confidence).max(2.0);
        let color = confidence_to_color(note.confidence);
        
        // Draw note name on left (bold)
        painter.text(
            egui::pos2(rect.min.x + 10.0, y + bar_height / 2.0),
            egui::Align2::LEFT_CENTER,
            &note.note,
            egui::FontId::proportional(13.0),
            egui::Color32::BLACK,
        );
        
        // Draw background track for the bar
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x + padding_left, y + 4.0),
                egui::pos2(rect.min.x + padding_left + max_bar_width, y + bar_height - 4.0),
            ),
            2.0,
            egui::Color32::from_rgb(230, 230, 235),
        );
        
        // Draw horizontal bar (filled)
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x + padding_left, y + 4.0),
                egui::pos2(rect.min.x + padding_left + bar_width, y + bar_height - 4.0),
            ),
            2.0,
            color,
        );
        
        // Draw bar border
        painter.rect_stroke(
            egui::Rect::from_min_max(
                egui::pos2(rect.min.x + padding_left, y + 4.0),
                egui::pos2(rect.min.x + padding_left + bar_width, y + bar_height - 4.0),
            ),
            1.0,
            egui::Stroke::new(1.5, color),
        );
        
        // Draw percentage label on right
        painter.text(
            egui::pos2(rect.max.x - 40.0, y + bar_height / 2.0),
            egui::Align2::RIGHT_CENTER,
            format!("{:.0}%", note.confidence * 100.0),
            egui::FontId::proportional(12.0),
            color,
        );
    }
    
    // Draw subtle border around the entire panel
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(180, 180, 200)),
    );
}

/// Draw note visualization (legacy, not used)
#[allow(dead_code)]
pub const fn draw_note_visualization(ui: &egui::Ui, notes: &[DetectedNote]) {
    let _ = notes; // unused
    let _ = ui; // unused
}

/// Draw spectrum visualization (legacy, not used)
#[allow(dead_code)]
pub const fn draw_spectrum_visualization(ui: &egui::Ui, notes: &[DetectedNote]) {
    let _ = notes; // unused
    let _ = ui; // unused
}

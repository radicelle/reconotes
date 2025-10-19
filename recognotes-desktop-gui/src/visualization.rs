use eframe::egui;
use crate::DetectedNote;

/// All possible musical notes to display
const ALL_NOTES: &[&str] = &[
    "C2", "C#2", "D2", "D#2", "E2", "F2", "F#2", "G2", "G#2", "A2", "A#2", "B2",
    "C3", "C#3", "D3", "D#3", "E3", "F3", "F#3", "G3", "G#3", "A3", "A#3", "B3",
    "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4", "G4", "G#4", "A4", "A#4", "B4",
    "C5", "C#5", "D5", "D#5", "E5", "F5", "F#5", "G5", "G#5", "A5", "A#5", "B5",
    "C6", "C#6", "D6", "D#6", "E6", "F6", "F#6", "G6", "G#6", "A6", "A#6", "B6",
    "C7", "C#7", "D7", "D#7", "E7", "F7", "F#7", "G7", "G#7", "A7", "A#7", "B7",
];

/// Draw vertical bars for all notes
pub fn draw_vertical_bars(ui: &mut egui::Ui, detected_notes: &[DetectedNote], rect: egui::Rect) {
    let painter = ui.painter();
    
    // Draw background
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 40));
    
    // Create a map of detected notes with their confidence
    let mut note_map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    for note in detected_notes {
        note_map.insert(note.note.clone(), note.confidence);
    }
    
    let num_notes = ALL_NOTES.len();
    let bar_width = (rect.width() - 10.0) / num_notes as f32;
    let padding_left = 5.0;
    let padding_bottom = 30.0;
    let max_bar_height = rect.height() - padding_bottom - 5.0;
    
    // Draw each note bar
    for (idx, &note_name) in ALL_NOTES.iter().enumerate() {
        let x = rect.min.x + padding_left + (idx as f32 * bar_width);
        let confidence = note_map.get(note_name).copied().unwrap_or(0.0);
        let bar_height = max_bar_height * confidence;
        
        // Draw background track (empty bar)
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x + 1.0, rect.max.y - padding_bottom),
                egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
            ),
            1.0,
            egui::Color32::from_rgb(60, 60, 80),
        );
        
        // Draw filled bar if note detected
        if confidence > 0.0 {
            let color = confidence_to_color(confidence);
            let bar_top = rect.max.y - padding_bottom - bar_height;
            
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x + 1.0, bar_top),
                    egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
                ),
                1.0,
                color,
            );
            
            // Draw border
            painter.rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(x + 1.0, bar_top),
                    egui::pos2(x + bar_width - 1.0, rect.max.y - 25.0),
                ),
                0.0,
                egui::Stroke::new(1.0, color),
            );
        }
        
        // Draw note label at bottom
        let font_size = if num_notes > 48 { 7.0 } else { 9.0 };
        painter.text(
            egui::pos2(x + bar_width / 2.0, rect.max.y - 10.0),
            egui::Align2::CENTER_CENTER,
            note_name,
            egui::FontId::monospace(font_size),
            if confidence > 0.5 {
                confidence_to_color(confidence)
            } else {
                egui::Color32::from_rgb(100, 100, 120)
            },
        );
    }
    
    // Draw border
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 100, 150)),
    );
}

/// Convert confidence value to color
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
pub fn draw_horizontal_bars(ui: &mut egui::Ui, notes: &[DetectedNote], rect: egui::Rect) {
    if notes.is_empty() {
        return;
    }
    
    // Sort notes by name
    let mut sorted_notes = notes.to_vec();
    sorted_notes.sort_by(|a, b| a.note.cmp(&b.note));
    
    let painter = ui.painter();
    
    let num_notes = sorted_notes.len();
    let bar_height = ((rect.height() - 15.0) / num_notes as f32).min(35.0).max(20.0);
    let padding_top = 8.0;
    let padding_left = 70.0;
    let padding_right = 50.0;
    let max_bar_width = rect.width() - padding_left - padding_right;
    
    // Draw each bar
    for (idx, note) in sorted_notes.iter().enumerate() {
        let y = rect.min.y + padding_top + (idx as f32 * bar_height);
        
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
pub fn draw_note_visualization(ui: &mut egui::Ui, notes: &[DetectedNote]) {
    let _ = notes; // unused
    let _ = ui; // unused
}

/// Draw spectrum visualization (legacy, not used)
#[allow(dead_code)]
pub fn draw_spectrum_visualization(ui: &mut egui::Ui, notes: &[DetectedNote]) {
    let _ = notes; // unused
    let _ = ui; // unused
}

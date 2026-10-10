use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

/// Draw a modern Lucide-style search magnifying glass icon with geometric precision
pub fn draw_search_icon(ui: &mut Ui, size: f32, color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let painter = ui.painter();

    // Circle at top-left, handle at bottom-right
    let center = Pos2::new(rect.min.x + size * 0.42, rect.min.y + size * 0.42);
    let radius = size * 0.28;
    let stroke_width = (size * 0.09).clamp(1.2, 2.4);
    let stroke = Stroke::new(stroke_width, color);

    painter.circle_stroke(center, radius, stroke);

    // Diagonal handle
    let handle_start = Pos2::new(center.x + radius * 0.707, center.y + radius * 0.707);
    let handle_end = Pos2::new(rect.max.x - size * 0.08, rect.max.y - size * 0.08);
    painter.line_segment([handle_start, handle_end], stroke);

    resp
}

/// Draw a Lucide-style maximize/expand icon (⛶)
pub fn draw_expand_icon(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let w = rect.width();
    let h = rect.height();
    let arm = (w * 0.32).min(h * 0.32);

    // Top-left corner
    painter.line_segment([Pos2::new(rect.min.x, rect.min.y + arm), Pos2::new(rect.min.x, rect.min.y)], stroke);
    painter.line_segment([Pos2::new(rect.min.x, rect.min.y), Pos2::new(rect.min.x + arm, rect.min.y)], stroke);

    // Top-right corner
    painter.line_segment([Pos2::new(rect.max.x - arm, rect.min.y), Pos2::new(rect.max.x, rect.min.y)], stroke);
    painter.line_segment([Pos2::new(rect.max.x, rect.min.y), Pos2::new(rect.max.x, rect.min.y + arm)], stroke);

    // Bottom-left corner
    painter.line_segment([Pos2::new(rect.min.x, rect.max.y - arm), Pos2::new(rect.min.x, rect.max.y)], stroke);
    painter.line_segment([Pos2::new(rect.min.x, rect.max.y), Pos2::new(rect.min.x + arm, rect.max.y)], stroke);

    // Bottom-right corner
    painter.line_segment([Pos2::new(rect.max.x - arm, rect.max.y), Pos2::new(rect.max.x, rect.max.y)], stroke);
    painter.line_segment([Pos2::new(rect.max.x, rect.max.y), Pos2::new(rect.max.x, rect.max.y - arm)], stroke);
}

/// Draw a Lucide-style close (×) icon
pub fn draw_close_icon(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let pad = rect.width() * 0.28;
    painter.line_segment(
        [Pos2::new(rect.min.x + pad, rect.min.y + pad), Pos2::new(rect.max.x - pad, rect.max.y - pad)],
        stroke,
    );
    painter.line_segment(
        [Pos2::new(rect.min.x + pad, rect.max.y - pad), Pos2::new(rect.max.x - pad, rect.min.y + pad)],
        stroke,
    );
}

/// Draw a Lucide-style return/enter arrow (↵)
pub fn draw_return_key_icon(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let w = rect.width();
    let h = rect.height();
    let arrow_tip = Pos2::new(rect.min.x + w * 0.2, rect.min.y + h * 0.6);
    let corner = Pos2::new(rect.min.x + w * 0.7, rect.min.y + h * 0.6);
    let top = Pos2::new(rect.min.x + w * 0.7, rect.min.y + h * 0.3);

    // L-line
    painter.line_segment([top, corner], stroke);
    painter.line_segment([corner, arrow_tip], stroke);

    // Arrowhead points
    let head_size = (w * 0.25).min(h * 0.25);
    painter.line_segment([Pos2::new(arrow_tip.x + head_size, arrow_tip.y - head_size), arrow_tip], stroke);
    painter.line_segment([Pos2::new(arrow_tip.x + head_size, arrow_tip.y + head_size), arrow_tip], stroke);
}

/// Draw a Lucide-style filter sliders icon (⚙)
pub fn draw_sliders_icon(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let w = rect.width();
    let h = rect.height();

    // Top slider track
    let y1 = rect.min.y + h * 0.32;
    painter.line_segment([Pos2::new(rect.min.x + w * 0.15, y1), Pos2::new(rect.max.x - w * 0.15, y1)], stroke);
    // Top slider knob
    painter.circle_filled(Pos2::new(rect.min.x + w * 0.40, y1), stroke.width * 1.5, stroke.color);

    // Bottom slider track
    let y2 = rect.min.y + h * 0.68;
    painter.line_segment([Pos2::new(rect.min.x + w * 0.15, y2), Pos2::new(rect.max.x - w * 0.15, y2)], stroke);
    // Bottom slider knob
    painter.circle_filled(Pos2::new(rect.min.x + w * 0.65, y2), stroke.width * 1.5, stroke.color);
}

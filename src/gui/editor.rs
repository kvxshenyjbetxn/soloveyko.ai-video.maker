use crate::localization::{Language, translate};
use eframe::egui;

use std::hash::{Hash, Hasher};

use crate::core::pipeline::timeline::text_splitter::TextRange;

fn get_encoder() -> &'static tiktoken::CoreBpe {
    tiktoken::get_encoding("cl100k_base").unwrap()
}

pub fn count_tokens(text: &str) -> usize {
    get_encoder().count(text)
}

fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Clone)]
pub struct EditorStats {
    pub char_count: usize,
    pub paragraph_count: usize,
    pub token_count: usize,
    pub fragments_paragraphs: usize,
    pub fragments_sentences: usize,
    pub fragments_char_limit: usize,
    pub active_split_ranges: Vec<TextRange>,

    pub last_text_hash: u64,
    pub last_char_limit: usize,
    pub last_split_mode: String,
}

impl Default for EditorStats {
    fn default() -> Self {
        Self {
            char_count: 0,
            paragraph_count: 0,
            token_count: 0,
            fragments_paragraphs: 0,
            fragments_sentences: 0,
            fragments_char_limit: 0,
            active_split_ranges: Vec::new(),
            last_text_hash: 0,
            last_char_limit: 0,
            last_split_mode: String::new(),
        }
    }
}

fn safe_byte_to_char(text: &str, byte_index: usize) -> usize {
    let bound = byte_index.min(text.len());
    let mut safe = bound;
    while safe > 0 && !text.is_char_boundary(safe) {
        safe -= 1;
    }
    text[..safe].chars().count()
}

fn segment_outline_rects(
    galley: &egui::Galley,
    galley_pos: egui::Pos2,
    ranges: &[TextRange],
    clip_rect: egui::Rect,
) -> Vec<egui::Rect> {
    const PAD_X: f32 = 4.0;
    const PAD_Y: f32 = 3.0;
    const GAP_X: f32 = 4.0;
    const GAP_Y: f32 = 4.0;

    let galley_text = galley.text();
    let char_ranges: Vec<(usize, usize)> = ranges
        .iter()
        .filter_map(|range| {
            let start = safe_byte_to_char(galley_text, range.byte_start);
            let end = safe_byte_to_char(galley_text, range.byte_end);
            (start < end).then_some((start, end))
        })
        .collect();

    if char_ranges.is_empty() {
        return Vec::new();
    }

    let mut rects = Vec::new();
    let mut row_char_start = 0;

    for row in &galley.rows {
        let row_start = row_char_start;
        let row_end = row_start + row.char_count_excluding_newline();

        for &(seg_start, seg_end) in &char_ranges {
            let overlap_start = seg_start.max(row_start);
            let overlap_end = seg_end.min(row_end);
            if overlap_start >= overlap_end {
                continue;
            }

            let start_col = overlap_start - row_start;
            let end_col = overlap_end - row_start;

            let min = galley_pos + egui::vec2(row.x_offset(start_col), row.min_y());
            let max = galley_pos + egui::vec2(row.x_offset(end_col), row.max_y());

            let rect = egui::Rect::from_min_max(min, max)
                .expand2(egui::vec2(PAD_X, PAD_Y))
                .shrink2(egui::vec2(GAP_X * 0.5, GAP_Y * 0.5));

            let clipped = rect.intersect(clip_rect);
            if clipped.width() > 2.0 && clipped.height() > 2.0 {
                rects.push(clipped);
            }
        }

        row_char_start += row.char_count_including_newline();
    }

    rects
}

pub fn draw_editor(
    ui: &mut egui::Ui,
    text: &mut String,
    language: Language,
    text_split_mode: &str,
    text_split_char_limit: usize,
    segment_outlines_enabled: &mut bool,
    stats: &mut EditorStats,
) {
    let current_hash = calculate_hash(text);
    if stats.last_text_hash != current_hash
        || stats.last_char_limit != text_split_char_limit
        || stats.last_split_mode != text_split_mode
    {
        stats.last_text_hash = current_hash;
        stats.last_char_limit = text_split_char_limit;
        stats.last_split_mode = text_split_mode.to_string();

        stats.char_count = text.chars().count();
        stats.paragraph_count = text.lines().filter(|line| !line.trim().is_empty()).count();
        stats.token_count = count_tokens(text);
        stats.fragments_paragraphs =
            crate::core::pipeline::timeline::text_splitter::split_text(text, "paragraphs", 0).len();
        stats.fragments_sentences =
            crate::core::pipeline::timeline::text_splitter::split_text(text, "sentences", 0).len();
        stats.fragments_char_limit = crate::core::pipeline::timeline::text_splitter::split_text(
            text,
            "char_limit",
            text_split_char_limit,
        )
        .len();

        let active_ranges =
            crate::core::pipeline::timeline::text_splitter::split_text_preview_ranges(
                text,
                text_split_mode,
                text_split_char_limit,
            );
        stats.active_split_ranges = if active_ranges.len() > 1 {
            active_ranges
        } else {
            Vec::new()
        };
    }

    let char_count = stats.char_count;
    let paragraph_count = stats.paragraph_count;
    let token_count = stats.token_count;
    let fragments_paragraphs = stats.fragments_paragraphs;
    let fragments_sentences = stats.fragments_sentences;
    let fragments_char_limit = stats.fragments_char_limit;
    let highlight_ranges = stats.active_split_ranges.clone();

    // Статистика
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.add_space(12.0);
        let text_color = ui.visuals().widgets.noninteractive.text_color();
        let accent_color = ui.visuals().selection.bg_fill;
        let bullet_color = text_color.linear_multiply(0.3);

        ui.label(
            egui::RichText::new(translate(language, "stats_chars"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", char_count))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        ui.label(
            egui::RichText::new(translate(language, "stats_paragraphs"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", paragraph_count))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        ui.label(
            egui::RichText::new(translate(language, "stats_tokens"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", token_count))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        ui.label(
            egui::RichText::new(translate(language, "stats_fragments_paragraphs"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", fragments_paragraphs))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        ui.label(
            egui::RichText::new(translate(language, "stats_fragments_sentences"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", fragments_sentences))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
        ui.label(egui::RichText::new("  •  ").size(16.0).color(bullet_color));

        ui.label(
            egui::RichText::new(translate(language, "stats_fragments_chars"))
                .size(16.0)
                .color(text_color),
        );
        ui.label(
            egui::RichText::new(format!(" {}", fragments_char_limit))
                .size(16.0)
                .strong()
                .color(accent_color),
        );
    });

    // Тумблер
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.checkbox(
            segment_outlines_enabled,
            translate(language, "editor_segment_outlines_toggle"),
        );
    });
    ui.add_space(4.0);
    ui.separator();

    // Редактор
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(8.0);

            let text_edit = egui::TextEdit::multiline(text)
                .hint_text(translate(language, "editor_hint"))
                .desired_width(f32::INFINITY)
                .desired_rows(40)
                .min_size(ui.available_size())
                .font(egui::TextStyle::Body)
                .frame(false);

            let output = text_edit.show(ui);

            if *segment_outlines_enabled && !highlight_ranges.is_empty() {
                let outline_ranges: Vec<TextRange> = {
                    let new_hash = calculate_hash(text);
                    if stats.last_text_hash != new_hash {
                        let active = crate::core::pipeline::timeline::text_splitter::split_text_preview_ranges(
                            text, text_split_mode, text_split_char_limit,
                        );
                        stats.active_split_ranges = if active.len() > 1 { active } else { Vec::new() };
                        stats.last_text_hash = new_hash;
                        stats.active_split_ranges.clone()
                    } else {
                        highlight_ranges
                    }
                };

                if !outline_ranges.is_empty() {
                    let painter = ui.painter_at(output.response.rect);
                    let accent_color = ui.visuals().selection.bg_fill;
                    let stroke = egui::Stroke::new(1.0, accent_color);

                    for rect in segment_outline_rects(
                        output.galley.as_ref(),
                        output.galley_pos,
                        &outline_ranges,
                        output.text_clip_rect,
                    ) {
                        painter.rect_stroke(rect, 6.0, stroke);
                    }
                }
            }
        });
}

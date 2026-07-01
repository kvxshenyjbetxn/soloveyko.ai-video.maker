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
    const PAD_X: f32 = 3.0;
    const PAD_Y: f32 = 1.0;

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

    // Один шматок на кожен рядок, яким проходить сегмент. Список природно
    // впорядкований зліва направо в межах рядка, бо ranges відсортовані за
    // позицією в тексті, а рядки обходяться згори вниз.
    struct Piece {
        range_idx: usize,
        row_idx: usize,
        rect: egui::Rect,
    }

    let mut pieces: Vec<Piece> = Vec::new();
    let mut row_char_start = 0;

    for (row_idx, row) in galley.rows.iter().enumerate() {
        let row_start = row_char_start;
        let row_end = row_start + row.char_count_excluding_newline();

        // Якщо міжрядковий інтервал збільшено (для читабельності), повна висота
        // рядка включає зайвий простір знизу. Контур тримаємо тісним до самого
        // чорнила тексту, тож беремо межі мешу гліфів, а не всю висоту рядка.
        let ink_bounds = row.visuals.mesh_bounds;
        let (y_min, y_max) = if ink_bounds.is_finite() && ink_bounds.is_positive() {
            (ink_bounds.min.y, ink_bounds.max.y)
        } else {
            (row.min_y(), row.max_y())
        };

        for (range_idx, &(seg_start, seg_end)) in char_ranges.iter().enumerate() {
            let overlap_start = seg_start.max(row_start);
            let overlap_end = seg_end.min(row_end);
            if overlap_start >= overlap_end {
                continue;
            }

            let start_col = overlap_start - row_start;
            let end_col = overlap_end - row_start;

            let min = galley_pos + egui::vec2(row.x_offset(start_col), y_min);
            let max = galley_pos + egui::vec2(row.x_offset(end_col), y_max);

            pieces.push(Piece {
                range_idx,
                row_idx,
                rect: egui::Rect::from_min_max(min, max),
            });
        }

        row_char_start += row.char_count_including_newline();
    }

    if pieces.is_empty() {
        return Vec::new();
    }

    // Перший/останній рядок кожного сегмента — щоб рамки суміжних рядків одного
    // сегмента стикались рівно, без вертикального напуску один на одного.
    let mut row_bounds: std::collections::HashMap<usize, (usize, usize)> =
        std::collections::HashMap::new();
    for p in &pieces {
        row_bounds
            .entry(p.range_idx)
            .and_modify(|(min_r, max_r)| {
                *min_r = (*min_r).min(p.row_idx);
                *max_r = (*max_r).max(p.row_idx);
            })
            .or_insert((p.row_idx, p.row_idx));
    }
    // Якщо зліва/справа впритул є шматок ІНШОГО сегмента на тому ж рядку, не
    // додаємо туди PAD_X — інакше обидва боки "з'їдають" один і той самий
    // пробіл між сегментами і рамки або торкаються, або (при примусовому
    // збільшенні зазору) заходять у текст сусіда. Без підпору з обох боків
    // видима прогалина дорівнює всьому природному пробілу між сегментами.
    for i in 0..pieces.len() {
        let has_left_neighbor = i > 0
            && pieces[i - 1].row_idx == pieces[i].row_idx
            && pieces[i - 1].range_idx != pieces[i].range_idx;
        let has_right_neighbor = i + 1 < pieces.len()
            && pieces[i + 1].row_idx == pieces[i].row_idx
            && pieces[i + 1].range_idx != pieces[i].range_idx;

        let (min_row, max_row) = row_bounds[&pieces[i].range_idx];
        let top_pad = if pieces[i].row_idx == min_row { PAD_Y } else { 0.0 };
        let bottom_pad = if pieces[i].row_idx == max_row { PAD_Y } else { 0.0 };
        let left_pad = if has_left_neighbor { 0.0 } else { PAD_X };
        let right_pad = if has_right_neighbor { 0.0 } else { PAD_X };

        pieces[i].rect = egui::Rect::from_min_max(
            pieces[i].rect.min - egui::vec2(left_pad, top_pad),
            pieces[i].rect.max + egui::vec2(right_pad, bottom_pad),
        );
    }

    pieces
        .into_iter()
        .filter_map(|p| {
            let clipped = p.rect.intersect(clip_rect);
            (clipped.width() > 2.0 && clipped.height() > 2.0).then_some(clipped)
        })
        .collect()
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

            let use_outlines = *segment_outlines_enabled;

            // В режимі контурів явно резервуємо смугу навколо поля тексту, щоб
            // контуру сегментів завжди було де намалюватись і вони не обрізались.
            // editor_bounds — жорстка межа: контур ніколи не малюється за нею,
            // незалежно від того, наскільки текст (наприклад, довге слово, що не
            // розривається) міг вилізти за межі своєї обчисленої ширини.
            const OUTLINE_MARGIN: f32 = 8.0;
            let editor_bounds = ui.available_rect_before_wrap();

            let output = if use_outlines {
                let target_rect = editor_bounds.shrink(OUTLINE_MARGIN);
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(target_rect), |ui| {
                    let mut layouter = |ui: &egui::Ui, text_buf: &str, wrap: f32| {
                        // Зазор між контурами сегментів: замість намагання
                        // "видавити" його стисканням рамок (що ризикує зачепити
                        // текст сусіда), реально розсуваємо сам текст на межах
                        // сегментів — порожнім місцем перед початком наступного
                        // сегмента по горизонталі. По вертикалі так само реально
                        // збільшено міжрядковий інтервал (для всіх рядків — точково
                        // "тільки між сегментами" неможливо визначити до того, як
                        // текст розкладеться по рядках), тож сусідні сегменти на
                        // різних рядках теж отримують помітно більше повітря.
                        const SEGMENT_GAP_X: f32 = 10.0;

                        let mut job = egui::text::LayoutJob::default();
                        job.wrap.max_width = wrap;
                        let font = egui::TextStyle::Body.resolve(ui.style());
                        let fmt = egui::TextFormat {
                            font_id: font.clone(),
                            color: ui.visuals().widgets.inactive.text_color(),
                            line_height: Some(font.size * 1.35 + 10.0),
                            ..Default::default()
                        };

                        let split_ranges = crate::core::pipeline::timeline::text_splitter::split_text_preview_ranges(
                            text_buf, text_split_mode, text_split_char_limit,
                        );

                        if split_ranges.len() > 1 {
                            let mut cursor = 0usize;
                            for (i, range) in split_ranges.iter().enumerate() {
                                let mut crosses_newline = false;
                                if range.byte_start > cursor {
                                    let gap_text = &text_buf[cursor..range.byte_start];
                                    crosses_newline = gap_text.contains('\n');
                                    job.append(gap_text, 0.0, fmt.clone());
                                }
                                // Не додаємо горизонтальний відступ, якщо цей
                                // сегмент починає новий рядок/абзац після
                                // переносу — інакше абзац виглядає як зсунутий
                                // вправо "відступ", а не як реальний перенос.
                                let leading = if i == 0 || crosses_newline { 0.0 } else { SEGMENT_GAP_X };
                                job.append(&text_buf[range.byte_start..range.byte_end], leading, fmt.clone());
                                cursor = range.byte_end;
                            }
                            if cursor < text_buf.len() {
                                job.append(&text_buf[cursor..], 0.0, fmt.clone());
                            }
                        } else {
                            job.append(text_buf, 0.0, fmt);
                        }

                        ui.fonts(|f| f.layout_job(job))
                    };
                    egui::TextEdit::multiline(text)
                        .hint_text(translate(language, "editor_hint"))
                        .desired_width(target_rect.width())
                        .desired_rows(40)
                        .min_size(target_rect.size())
                        .font(egui::TextStyle::Body)
                        .layouter(&mut layouter)
                        .frame(false)
                        .show(ui)
                })
                .inner
            } else {
                egui::TextEdit::multiline(text)
                    .hint_text(translate(language, "editor_hint"))
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .min_size(ui.available_size())
                    .font(egui::TextStyle::Body)
                    .frame(false)
                    .show(ui)
            };

            if use_outlines {
                // Рахуємо діапазони наживо з поточного (вже після редагування
                // цього кадру) тексту — той самий виклик, що й у layouter'і,
                // щоб контур завжди відповідав тому, що реально намальовано, а
                // не відставав на кадр через кеш статистики.
                let active = crate::core::pipeline::timeline::text_splitter::split_text_preview_ranges(
                    text, text_split_mode, text_split_char_limit,
                );
                let outline_ranges: Vec<TextRange> = if active.len() > 1 { active } else { Vec::new() };

                if !outline_ranges.is_empty() {
                    let painter = ui.painter_at(editor_bounds);
                    let accent_color = ui.visuals().selection.bg_fill;
                    let stroke = egui::Stroke::new(1.0, accent_color);

                    for rect in segment_outline_rects(
                        output.galley.as_ref(),
                        output.galley_pos,
                        &outline_ranges,
                        editor_bounds,
                    ) {
                        painter.rect_stroke(rect, 0.0, stroke);
                    }
                }
            }
        });
}

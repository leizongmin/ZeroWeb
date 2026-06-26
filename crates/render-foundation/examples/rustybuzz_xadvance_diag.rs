//! R667 诊断：rustybuzz（HarfBuzz）`glyph_positions.x_advance` 是否匹配 chromium 的 line-count？
//!
//! **背景**：R642 实测 text-indent-wrap-001 ZW=8 行（heuristic，DejaVu 宽）vs CHR=6 行
//! （NotoSansCJK 窄）。R643 用 NotoSansCJK + **fontdue** advance 把 line-count 修到 6，
//! 但 product-smoke net-negative（fontdue advance ≠ HarfBuzz，welcome 16→23%）。R643 的
//! 「combo」仍用 fontdue advance，**rustybuzz x_advance（真 HarfBuzz，含 GPOS）从未测过**。
//!
//! **本诊断**：直接用 rustybuzz::shape 的 `glyph_positions.x_advance`（HarfBuzz 计算的
//! advance）测 text-indent-wrap-001 的 line-count。若 = 6 → Phase A rustybuzz same-source
//! 假设成立（go）；若 ≠ 6 → rustybuzz 也不匹配 chromium（no-go，font-wall 须更深修）。
//! 零生产影响（examples 不入 make test，手动 run）。
//!
//! 用法：cargo run -p zero-render-foundation --example rustybuzz_xadvance_diag

fn main() {
    let font_path = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc";
    let font_data = match std::fs::read(font_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("SKIP: cannot read {font_path}: {e}");
            return;
        }
    };
    let face = match rustybuzz::Face::from_slice(&font_data, 0) {
        Some(f) => f,
        None => {
            eprintln!("SKIP: rustybuzz Face::from_slice failed");
            return;
        }
    };
    let upem = face.units_per_em() as f32;
    let px_per_unit = if upem > 0.0 { 16.0 / upem } else { 0.0 };

    // text-indent-wrap-001 文本（"This is a long piece of text that will wrap to multiple lines. " ×12）
    let text = "This is a long piece of text that will wrap to multiple lines. ".repeat(12);

    // ---- rustybuzz (HarfBuzz) x_advance per glyph ----
    let mut buf = rustybuzz::UnicodeBuffer::new();
    buf.push_str(&text);
    let shaped = rustybuzz::shape(&face, &[], buf);
    let rb_advances: Vec<f32> = shaped
        .glyph_positions()
        .iter()
        .map(|p| p.x_advance as f32 * px_per_unit)
        .collect();
    let rb_total: f32 = rb_advances.iter().sum();

    // ---- heuristic advance per char（复刻 estimate_char_width：Latin 0.55 / space 0.25 / digit 0.5）----
    let font_size = 16.0f32;
    let heur_advances: Vec<f32> = text
        .chars()
        .map(|c| {
            if c.is_ascii_whitespace() {
                font_size * 0.25
            } else if c.is_ascii_punctuation() {
                font_size * 0.4
            } else if c.is_ascii_alphabetic() {
                font_size * 0.55
            } else {
                font_size * 0.5
            }
        })
        .collect();
    let heur_total: f32 = heur_advances.iter().sum();

    // ---- fontdue char-based advance（R643 的方法：metrics(char).advance_width）----
    use zero_render_foundation::font::FontLoader;
    let mut loader = FontLoader::new();
    let fd_total;
    let fd_lines;
    if let Ok(fid) = loader.load_font(&font_data) {
        let fd_font = loader.get(fid).expect("get fontdue font");
        let fd_advances: Vec<f32> = text.chars().map(|c| fd_font.metrics(c, 16.0).advance_width).collect();
        fd_total = fd_advances.iter().sum();
        fd_lines = count_lines_heuristic(&text, &fd_advances);
    } else {
        fd_total = 0.0;
        fd_lines = 0;
    }

    // ---- greedy word-wrap line-count（first line 784-100=684px text-indent，后续 784px）----
    // 用 rustybuzz glyph cluster 切词（cluster 索引映射回 char），遇空格断词。
    let glyph_infos = shaped.glyph_infos();
    // 建立 per-glyph advance + codepoint（用于识别空格断点），按 rustybuzz
    let rb_lines = count_lines_rustybuzz(&text, glyph_infos, &rb_advances);
    let heur_lines = count_lines_heuristic(&text, &heur_advances);

    eprintln!("=== text-indent-wrap-001 line-count diagnostic (R642: ZW=8, CHR=6) ===");
    eprintln!(
        "rustybuzz(HarfBuzz) x_advance: total {:.1}px → {} lines",
        rb_total, rb_lines
    );
    eprintln!(
        "fontdue char metrics (R643 方法): total {:.1}px → {} lines",
        fd_total, fd_lines
    );
    eprintln!(
        "heuristic (estimate_char_width): total {:.1}px → {} lines",
        heur_total, heur_lines
    );
    eprintln!("chromium target: 6 lines");
    let best = [rb_lines, fd_lines, heur_lines]
        .iter()
        .min_by_key(|n| n.abs_diff(6))
        .copied()
        .unwrap();
    eprintln!(
        "→ closest to CHR(6): {} lines | rustybuzz={} fontdue={} heuristic={}",
        best, rb_lines, fd_lines, heur_lines
    );
    eprintln!(
        "→ Phase A rustybuzz same-source 完全匹配 CHR? {}",
        if rb_lines == 6 {
            "YES"
        } else {
            "NO（rustybuzz 也不完全匹配 chromium）"
        }
    );
}

/// rustybuzz greedy line-count：按 glyph cluster 回溯到 char，空格处可断行。
fn count_lines_rustybuzz(text: &str, infos: &[rustybuzz::GlyphInfo], advances: &[f32]) -> usize {
    let line_w_full = 784.0_f32;
    let line_w_first = 784.0 - 100.0; // text-indent 100px
    let chars: Vec<char> = text.chars().collect();
    let mut lines = 1usize;
    let mut cur_w = 0.0_f32;
    let mut capacity = line_w_first;
    let mut last_space_w = 0.0_f32;
    let mut last_space_glyph_idx = None;
    for (i, adv) in advances.iter().enumerate() {
        // cluster → char index → 是否空格
        let cp = infos
            .get(i)
            .and_then(|info| chars.get(info.cluster as usize))
            .copied()
            .unwrap_or(' ');
        if cp == ' ' {
            last_space_glyph_idx = Some(i);
            last_space_w = cur_w;
        }
        let new_w = cur_w + adv;
        if new_w > capacity && cur_w > 0.0 {
            // 断行
            if last_space_glyph_idx.is_some() {
                // 在最近空格处断（line 内容到 last_space_w）
                cur_w -= last_space_w; // 剩余宽 = 当前行宽 - 断点宽
                lines += 1;
                capacity = line_w_full;
                last_space_glyph_idx = None;
                last_space_w = 0.0;
                // 注意：简化处理，剩余宽近似（足够诊断 line-count 数量级）
                cur_w = cur_w.max(0.0);
            } else {
                lines += 1;
                capacity = line_w_full;
                cur_w = *adv;
            }
        } else {
            cur_w = new_w;
        }
    }
    lines
}

/// heuristic greedy line-count（按 char advance + 空格断点）。
fn count_lines_heuristic(text: &str, advances: &[f32]) -> usize {
    let line_w_full = 784.0_f32;
    let line_w_first = 784.0 - 100.0;
    let chars: Vec<char> = text.chars().collect();
    let mut lines = 1usize;
    let mut cur_w = 0.0_f32;
    let mut capacity = line_w_first;
    let mut last_space_w = 0.0_f32;
    let mut last_space_idx = None;
    for (i, adv) in advances.iter().enumerate() {
        let cp = chars.get(i).copied().unwrap_or(' ');
        if cp == ' ' {
            last_space_idx = Some(i);
            last_space_w = cur_w;
        }
        let new_w = cur_w + adv;
        if new_w > capacity && cur_w > 0.0 {
            if last_space_idx.is_some() {
                cur_w -= last_space_w;
                lines += 1;
                capacity = line_w_full;
                last_space_idx = None;
                last_space_w = 0.0;
                cur_w = cur_w.max(0.0);
            } else {
                lines += 1;
                capacity = line_w_full;
                cur_w = *adv;
            }
        } else {
            cur_w = new_w;
        }
    }
    lines
}

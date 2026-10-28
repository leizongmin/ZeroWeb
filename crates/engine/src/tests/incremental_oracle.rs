//! 增量渲染运行时对照 oracle（engine pipeline 运行时面）。
//!
//! 覆盖控制面余项的 engine 侧三件事：
//! 1. 多次导航运行时对照（INC-04）：同管线 `render_html` A→B→A′，A′ 帧
//!    必须与 A 逐位一致且与冷启动参考一致（缓存换代不留旧文档残留）；
//! 2. 命中对照（INC-03）：warm 变更路径（`render_with_dom_mutations_persistent`
//!    复用 cached_doc/样式缓存）与 cold 全量重解析路径的命中测试在采样网格上
//!    逐点一致，并含必须失效（几何变更后旧点位不再命中、新点位命中）与必须
//!    保留（paint-only 变更不动命中）两个方向的显式门；
//! 3. 像素容差契约运行时应用（INC-04）：自对照双路径 warm==cold 同光栅器
//!    逐位相等、不设容差（2026-10-23 契约定案；任何差异即分歧）。
//!
//! 契约依据：m1-layout f32 精确相等契约、m3-b 导航缓存换代静态审计
//! （本文件为其运行时半边）、m3-c 半透明 partial==full 逐位等价先例。
//! 文本臂按 render-foundation cpu/tests.rs 先例用空 `FontLoader`（无字形
//! 光栅化）——对照面为 fill 几何/半透明混合/溢出；字形光栅等价已由
//! render-foundation 与 compositor 侧常驻测试覆盖。

use zero_render_foundation::cpu::render_full_scene;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::image_cache::ImageCache;
use zero_render_foundation::surface::FrameBuffer;

use crate::pipeline::{RenderPipeline, RenderResult};

// ── 导航对照文档（INC-04 A→B→A′）──

const NAV_A_HTML: &str = "<html><body>\
<div id=\"hero\" style=\"background:rgb(18,52,86);width:200px;height:80px;margin:10px\">Hero A</div>\
<a id=\"lnk\" href=\"https://a.example/\" style=\"display:block;width:300px;height:20px\">alpha link</a>\
<div id=\"tail\" style=\"background:rgba(255,0,0,0.5);width:320px;height:40px\">tail</div>\
</body></html>";

/// B 页结构/高度与 A 显著不同（单列 300px 高绿块），保证往返对照两态可区分。
const NAV_B_HTML: &str = "<html><body>\
<p id=\"only\" style=\"background:rgb(0,200,0);width:150px;height:300px;margin:6px\">B page</p>\
<a id=\"blnk\" href=\"https://b.example/\" style=\"display:block;width:300px;height:20px\">beta</a>\
</body></html>";

const NAV_CSS: &str = "body { margin: 0; font-size: 14px; }";

// ── 命中/像素对照文档（INC-03/INC-04 warm 变更序列，逐步累积终态）──

const HIT_HTML_INITIAL: &str = "<html><body>\
<a id=\"l1\" href=\"https://old.example/\" style=\"display:block;width:300px;height:30px;background:rgb(34,34,34)\">old link</a>\
<div id=\"m\" style=\"background:rgb(51,102,153);width:120px;height:120px;margin-top:20px\">mover</div>\
<div id=\"p\" style=\"background:rgba(0,128,0,0.5);width:100%;height:24px\">paint bar</div>\
</body></html>";

/// 步 1 等价终态：`#l1` href 换新（SetAttr）。
const HIT_HTML_STEP1: &str = "<html><body>\
<a id=\"l1\" href=\"https://new.example/\" style=\"display:block;width:300px;height:30px;background:rgb(34,34,34)\">old link</a>\
<div id=\"m\" style=\"background:rgb(51,102,153);width:120px;height:120px;margin-top:20px\">mover</div>\
<div id=\"p\" style=\"background:rgba(0,128,0,0.5);width:100%;height:24px\">paint bar</div>\
</body></html>";

/// 步 2 等价终态：`#m` 下移（SetStyle margin-top 20px→150px，几何必须失效）。
const HIT_HTML_STEP2: &str = "<html><body>\
<a id=\"l1\" href=\"https://new.example/\" style=\"display:block;width:300px;height:30px;background:rgb(34,34,34)\">old link</a>\
<div id=\"m\" style=\"background:rgb(51,102,153);width:120px;height:120px;margin-top:150px\">mover</div>\
<div id=\"p\" style=\"background:rgba(0,128,0,0.5);width:100%;height:24px\">paint bar</div>\
</body></html>";

/// 步 3 等价终态：`#p` 半透明底色换色（paint-only，几何必须保留）。
const HIT_HTML_STEP3: &str = "<html><body>\
<a id=\"l1\" href=\"https://new.example/\" style=\"display:block;width:300px;height:30px;background:rgb(34,34,34)\">old link</a>\
<div id=\"m\" style=\"background:rgb(51,102,153);width:120px;height:120px;margin-top:150px\">mover</div>\
<div id=\"p\" style=\"background:rgba(255,0,0,0.5);width:100%;height:24px\">paint bar</div>\
</body></html>";

/// 步 4 等价终态：`#m` 文本替换（SetText，固定盒尺寸隔离文本度量面）。
const HIT_HTML_STEP4: &str = "<html><body>\
<a id=\"l1\" href=\"https://new.example/\" style=\"display:block;width:300px;height:30px;background:rgb(34,34,34)\">old link</a>\
<div id=\"m\" style=\"background:rgb(51,102,153);width:120px;height:120px;margin-top:150px\">replaced text</div>\
<div id=\"p\" style=\"background:rgba(255,0,0,0.5);width:100%;height:24px\">paint bar</div>\
</body></html>";

const HIT_CSS: &str = "body { margin: 0; font-size: 14px; }";

// ── 共用工具 ──

/// 光栅化上下文：warm/cold 两臂共用同一 FontLoader/GlyphCache/ImageCache，
/// 消除字体解析与缓存状态差异（同光栅器同输入 → 逐位可比）。走
/// `render_full_scene` 单线程确定性路径（与 render-foundation cpu/tests.rs
/// 先例一致；`rasterize_full_scene` 默认分发线程臂，不进常驻对照）。
struct RasterCtx {
    loader: FontLoader,
    glyph_cache: GlyphCache,
    image_cache: ImageCache,
}

impl RasterCtx {
    fn new() -> Self {
        Self {
            loader: FontLoader::new(),
            glyph_cache: GlyphCache::new(64),
            image_cache: ImageCache::new(8, 1 << 20),
        }
    }

    fn rasterize(&mut self, result: &RenderResult) -> FrameBuffer {
        render_full_scene(
            320,
            400,
            1.0,
            result.primitives(),
            &self.loader,
            &mut self.glyph_cache,
            Some(&mut self.image_cache),
            &[],
            &[],
            &[],
            &[],
        )
    }
}

/// 逐位相等断言，分歧时报告首差异像素坐标与两侧 RGBA（首差异定位）。
fn assert_frames_bitwise_equal(a: &FrameBuffer, b: &FrameBuffer, ctx: &str) {
    assert_eq!(a.width, b.width, "{ctx}: 帧宽分歧");
    assert_eq!(a.height, b.height, "{ctx}: 帧高分歧");
    assert_eq!(a.data.len(), b.data.len(), "{ctx}: 缓冲长度分歧");
    let (px_a, _) = a.data.as_chunks::<4>();
    let (px_b, _) = b.data.as_chunks::<4>();
    for (i, (pa, pb)) in px_a.iter().zip(px_b).enumerate() {
        if pa != pb {
            let x = i as u32 % a.width;
            let y = i as u32 / a.width;
            panic!("{ctx}: 首像素差异 ({x},{y}): {pa:?} != {pb:?}");
        }
    }
}

/// 命中网格：viewport 采样点上 `hit_test_element` 的稳定投影
/// （selector + 盒几何，f32 精确相等——跨管线 NodeId 不稳定，不参与比较）。
fn hit_grid(pipeline: &RenderPipeline) -> Vec<(String, f32, f32, f32, f32)> {
    let mut grid = Vec::new();
    for y in (0..400).step_by(16) {
        for x in (0..320).step_by(16) {
            match pipeline.hit_test_element(x as f32, y as f32) {
                Some(hit) => grid.push((hit.selector, hit.x, hit.y, hit.width, hit.height)),
                None => grid.push(("<none>".to_string(), 0.0, 0.0, 0.0, 0.0)),
            }
        }
    }
    grid
}

/// 命中网格逐点相等断言，分歧时报告首差异点位（网格 20 列 × 16px 步进）。
fn assert_hit_grids_equal(warm: &[(String, f32, f32, f32, f32)], cold: &[(String, f32, f32, f32, f32)], ctx: &str) {
    assert_eq!(warm.len(), cold.len(), "{ctx}: 网格点数分歧");
    for (i, (w, c)) in warm.iter().zip(cold.iter()).enumerate() {
        if w != c {
            let x = (i as u32 % 20) * 16;
            let y = (i as u32 / 20) * 16;
            panic!("{ctx}: 首命中差异 ({x},{y}): warm {w:?} != cold {c:?}");
        }
    }
}

/// warm 臂驱动：初始渲染后的持久句柄变更路径，返回该步帧。
fn warm_step(pipeline: &mut RenderPipeline, mutations: &[crate::js_dom_bridge::DomMutation]) -> RenderResult {
    let (frame, _snapshot, _handles) = pipeline
        .render_with_dom_mutations_persistent(mutations, HIT_CSS, None)
        .expect("warm 变更路径不得报错");
    frame.expect("样式/文本变更必须产出帧")
}

/// cold 参考臂：新管线全量 `render_html` 等价终态文档（供命中网格对照）。
fn cold_pipeline(html: &str, css: &str) -> RenderPipeline {
    let mut pipeline = RenderPipeline::new(320.0, 400.0);
    pipeline.render_html(html, css);
    pipeline
}

/// cold 参考帧：全新管线渲染等价终态文档的图元输出（供像素对照）。
fn cold_frame(html: &str, css: &str) -> RenderResult {
    RenderPipeline::new(320.0, 400.0).render_html(html, css)
}

// ── INC-04：多次导航运行时对照（A→B→A′）──

/// 同管线 A→B→A′ 导航往返：A′ 帧、命中面、可滚动溢出与首轮 A 及冷启动参考
/// 逐位一致；B 帧必须真实换血（与 A 帧可区分），证明对照不是同帧自比。
/// 运行时半边对应 m3-b 导航缓存换代静态审计（cached_doc/styles/layout 整体替换）。
#[test]
fn inc04_navigation_a_b_aprime_matches_cold_reference() {
    let mut ctx = RasterCtx::new();
    let mut pipeline = RenderPipeline::new(320.0, 200.0);

    let a1 = pipeline.render_html(NAV_A_HTML, NAV_CSS);
    let px_a1 = ctx.rasterize(&a1);
    let height_a1 = pipeline.document_height().expect("A 布局高度");
    let grid_a1 = hit_grid(&pipeline);

    let b = pipeline.render_html(NAV_B_HTML, NAV_CSS);
    let px_b = ctx.rasterize(&b);
    assert!(
        px_b.data != px_a1.data,
        "导航到 B 后帧必须与 A 可区分（缓存换血 sanity）"
    );

    let a2 = pipeline.render_html(NAV_A_HTML, NAV_CSS);
    let px_a2 = ctx.rasterize(&a2);
    assert_frames_bitwise_equal(&px_a1, &px_a2, "A→B→A′ 往返帧");

    // 冷启动参考：全新管线渲染同一文档——A′ 不得残留首轮 A 之外的任何状态。
    let mut cold = RenderPipeline::new(320.0, 200.0);
    let cold_a = cold.render_html(NAV_A_HTML, NAV_CSS);
    let px_cold = ctx.rasterize(&cold_a);
    assert_frames_bitwise_equal(&px_a1, &px_cold, "A′ 对冷启动参考");

    // 溢出面：A′ 可滚动高度与 A 精确相等（f32 契约），且与 B 可区分。
    assert_eq!(
        pipeline.document_height().expect("A′ 布局高度"),
        height_a1,
        "A′ 溢出面漂移"
    );
    let mut cold_b = RenderPipeline::new(320.0, 200.0);
    cold_b.render_html(NAV_B_HTML, NAV_CSS);
    assert_ne!(
        pipeline.document_height().expect("A′ 布局高度"),
        cold_b.document_height().expect("B 布局高度"),
        "A′ 溢出面与 B 必须可区分（对照非同态自比）"
    );

    // 命中面：A′ 网格与首轮 A 网格逐点一致；B 网格必须不同（导航真实换血）。
    let grid_a2 = hit_grid(&pipeline);
    assert_hit_grids_equal(&grid_a1, &grid_a2, "A→B→A′ 命中网格");
    let grid_b = hit_grid(&cold_b);
    assert_ne!(grid_a1, grid_b, "B 命中网格与 A 必须可区分");

    // 链接命中跨导航换血：A′ 上命中 A 的 href，而非 B 残留。
    assert_eq!(
        pipeline.hit_test_link(20.0, 110.0).as_deref(),
        Some("https://a.example/"),
        "A′ 链接命中必须回到 A 的 href"
    );
}

// ── INC-03：命中对照 + 必须失效/必须保留门 ──

/// warm 变更序列（SetAttr/SetStyle/SetText）每步与 cold 全量重解析参考的
/// 命中网格逐点一致；步 2 显式断言必须失效（旧点位脱靶、新点位命中），
/// 步 3 显式断言必须保留（paint-only 变更命中网格逐点不动）。
#[test]
fn inc03_hit_test_warm_matches_cold_with_invalidation_gates() {
    let mut warm = RenderPipeline::new(320.0, 400.0);
    warm.render_html(HIT_HTML_INITIAL, HIT_CSS);

    // 步 1：SetAttr href——链接命中必须立刻反映新 href（缓存输入真实失效）。
    let _f1 = warm_step(
        &mut warm,
        &[crate::js_dom_bridge::DomMutation::SetAttr {
            selector: "#l1".to_string(),
            name: "href".to_string(),
            value: "https://new.example/".to_string(),
        }],
    );
    let cold1 = cold_pipeline(HIT_HTML_STEP1, HIT_CSS);
    assert_hit_grids_equal(&hit_grid(&warm), &hit_grid(&cold1), "步1 网格");
    assert_eq!(
        warm.hit_test_link(20.0, 15.0).as_deref(),
        Some("https://new.example/"),
        "href 变更后链接命中必须出新值"
    );

    // 步 2：SetStyle margin-top——几何必须失效：旧点位脱靶、新点位命中。
    let _f2 = warm_step(
        &mut warm,
        &[crate::js_dom_bridge::DomMutation::SetStyle {
            selector: "#m".to_string(),
            property: "margin-top".to_string(),
            value: "150px".to_string(),
        }],
    );
    let cold2 = cold_pipeline(HIT_HTML_STEP2, HIT_CSS);
    assert_hit_grids_equal(&hit_grid(&warm), &hit_grid(&cold2), "步2 网格");
    let old_point = warm.hit_test_element(60.0, 100.0).map(|h| h.selector);
    assert_ne!(old_point.as_deref(), Some("#m"), "移动后旧点位不得再命中 #m");
    let new_hit = warm.hit_test_element(60.0, 260.0).expect("移动后新点位必须命中元素");
    assert_eq!(new_hit.selector, "#m", "新点位必须命中移动后的 #m");

    // 步 3：paint-only 换色——几何必须保留：命中网格逐点不动。
    let grid_before = hit_grid(&warm);
    let _f3 = warm_step(
        &mut warm,
        &[crate::js_dom_bridge::DomMutation::SetStyle {
            selector: "#p".to_string(),
            property: "background-color".to_string(),
            value: "rgba(255,0,0,0.5)".to_string(),
        }],
    );
    let cold3 = cold_pipeline(HIT_HTML_STEP3, HIT_CSS);
    assert_hit_grids_equal(&hit_grid(&warm), &hit_grid(&cold3), "步3 网格");
    assert_hit_grids_equal(&grid_before, &hit_grid(&warm), "paint-only 必须不动命中");

    // 步 4：SetText——固定盒尺寸下命中网格不动，内容面换血。
    let _f4 = warm_step(
        &mut warm,
        &[crate::js_dom_bridge::DomMutation::SetText {
            selector: "#m".to_string(),
            text: "replaced text".to_string(),
        }],
    );
    let cold4 = cold_pipeline(HIT_HTML_STEP4, HIT_CSS);
    assert_hit_grids_equal(&hit_grid(&warm), &hit_grid(&cold4), "步4 网格");
}

// ── INC-04：像素容差契约运行时应用（warm==cold 逐位）──

/// 自对照双路径：warm 增量变更臂与 cold 全量重解析臂，同光栅器输出逐位相等、
/// 不设容差（2026-10-23 契约定案的运行时应用；含半透明图元步，
/// 与 m3-c rasterize 层 partial==full 逐位先例同族）。
#[test]
fn inc04_pixel_contract_warm_frames_bitwise_equal_cold() {
    let mut ctx = RasterCtx::new();
    let mut warm = RenderPipeline::new(320.0, 400.0);
    warm.render_html(HIT_HTML_INITIAL, HIT_CSS);

    let steps: &[(&str, &str, Vec<crate::js_dom_bridge::DomMutation>)] = &[
        (
            "步1 href",
            HIT_HTML_STEP1,
            vec![crate::js_dom_bridge::DomMutation::SetAttr {
                selector: "#l1".to_string(),
                name: "href".to_string(),
                value: "https://new.example/".to_string(),
            }],
        ),
        (
            "步2 移动",
            HIT_HTML_STEP2,
            vec![crate::js_dom_bridge::DomMutation::SetStyle {
                selector: "#m".to_string(),
                property: "margin-top".to_string(),
                value: "150px".to_string(),
            }],
        ),
        (
            "步3 半透明换色",
            HIT_HTML_STEP3,
            vec![crate::js_dom_bridge::DomMutation::SetStyle {
                selector: "#p".to_string(),
                property: "background-color".to_string(),
                value: "rgba(255,0,0,0.5)".to_string(),
            }],
        ),
        (
            "步4 文本替换",
            HIT_HTML_STEP4,
            vec![crate::js_dom_bridge::DomMutation::SetText {
                selector: "#m".to_string(),
                text: "replaced text".to_string(),
            }],
        ),
    ];

    for (name, cold_html, mutations) in steps {
        let frame = warm_step(&mut warm, mutations);
        let px_warm = ctx.rasterize(&frame);
        let px_cold = ctx.rasterize(&cold_frame(cold_html, HIT_CSS));
        assert_frames_bitwise_equal(&px_warm, &px_cold, name);
    }
}

// ── S1 文档级指纹 oracle（设计卡 §4-S1 前提门①：同输入重算==复用逐位一致）──

/// 以全预算推进一个独立会话至 Complete 并取结果（模拟一次导航的 styled 帧路径，
/// 与生产 `AsyncPageLoad::advance_render` 的会话生命周期同构：新建 → 推进 → 取帧）。
fn fingerprint_session(pipeline: &mut RenderPipeline, html: &str, css: &str) -> RenderResult {
    let mut session = crate::BudgetedRenderSession::new(html, css);
    loop {
        if matches!(
            pipeline.advance_budgeted_render(&mut session, 5_000.0),
            crate::BudgetAdvance::Complete
        ) {
            break;
        }
    }
    session.take_result().expect("complete session has result")
}

/// S1 热导航命中契约：同输入新会话（新 NodeId 代际）命中指纹缓存后，光栅化输出
/// 必须与首航帧及独立全量冷参考逐位相等；同会话首航（MISS 回填）亦须与冷参考
/// 逐位相等（指纹计算+缓存写入开销不得改变渲染输出）。
#[test]
fn s1_fingerprint_hot_navigation_hit_bitwise_equal_cold() {
    let mut ctx = RasterCtx::new();
    let mut warm = RenderPipeline::new(320.0, 400.0);
    // 冷导航：全新会话全量渲染（键 MISS → 回填缓存）。
    let cold_nav = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    // 热导航：同输入新会话——键相等必须命中，跳过四段整帧复用。
    let hot_nav = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    // 独立冷参考（另一管线全量重算同输入）。
    let reference = cold_frame(NAV_A_HTML, NAV_CSS);
    let px_cold_nav = ctx.rasterize(&cold_nav);
    let px_hot_nav = ctx.rasterize(&hot_nav);
    let px_reference = ctx.rasterize(&reference);
    assert_frames_bitwise_equal(&px_cold_nav, &px_reference, "S1 miss (first nav) vs cold reference");
    assert_frames_bitwise_equal(&px_hot_nav, &px_reference, "S1 hit (hot nav) vs cold reference");
}

/// S1 异输入判异：内容变化后键必须判异（MISS 全量），复用帧不得残留上一内容。
#[test]
fn s1_fingerprint_content_change_misses_and_recomputes() {
    let mut ctx = RasterCtx::new();
    let mut warm = RenderPipeline::new(320.0, 400.0);
    let first = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    let second = fingerprint_session(&mut warm, NAV_B_HTML, NAV_CSS);
    let reference = cold_frame(NAV_B_HTML, NAV_CSS);
    let px_second = ctx.rasterize(&second);
    let px_reference = ctx.rasterize(&reference);
    assert_frames_bitwise_equal(&px_second, &px_reference, "content change recomputes");
    let _ = first;
}

// ── S1 命中臂返修回归钉（DF-1/DF-3，红→绿证据见运行目录 evidence/t21-df-red/green）──

/// DF-1 突变批次：`body` 追加一个探针 div（appendChild 类非幂等操作——每重放
/// 一次多一个节点，效应可计数）。
fn df1_append_probe_mutations() -> Vec<crate::js_dom_bridge::DomMutation> {
    vec![
        crate::js_dom_bridge::DomMutation::CreateElement {
            handle: "__zw_df1_probe".to_string(),
            tag: "div".to_string(),
        },
        crate::js_dom_bridge::DomMutation::AppendChild {
            parent_selector: "body".to_string(),
            child_handle: "__zw_df1_probe".to_string(),
        },
    ]
}

/// body 直接子节点数（探针计数面；NAV_A_HTML 为单行 HTML，body 子节点全为元素）。
fn df1_body_child_count(pipeline: &RenderPipeline) -> usize {
    let doc = pipeline
        .cached_doc
        .as_ref()
        .expect("cached_doc present after render")
        .borrow();
    let body = doc.query_selector(doc.root(), "body").expect("body present");
    doc.get(body).expect("body node").children.len()
}

/// DF-1（blocker 根修钉）：脚本突变不得泄漏进指纹缓存。
///
/// 缺陷时序（返修前红）：冷航 Complete 回填把管线活文档 Rc 共享入缓存 → 脚本
/// 突变路径（`render_with_dom_mutations_persistent`）原位写入缓存条目持有的同一
/// 实例 → 热航同键命中还原含突变树 → 脚本重放 → appendChild 效应翻倍（body 5
/// 子 vs 期望 4）。修后语义 = 改前热导航语义：命中还原**纯净树**，脚本效应单次。
#[test]
fn s1_fingerprint_hit_restores_pristine_tree_script_effects_not_doubled() {
    let mut warm = RenderPipeline::new(320.0, 400.0);
    // 冷导航：MISS 全量渲染并回填（fill 点在脚本执行前 = 纯净树）。
    let cold = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    let _ = cold;
    // 页面脚本执行：非幂等突变一次。
    warm.render_with_dom_mutations_persistent(&df1_append_probe_mutations(), NAV_CSS, None)
        .expect("script mutation applies");
    assert_eq!(df1_body_child_count(&warm), 4, "冷航 + 单次突变：body 3 元素 + 1 探针");
    // 热导航：同输入新会话（新 NodeId 代际）——必须命中（四段零耗）。
    let mut session = crate::BudgetedRenderSession::new(NAV_A_HTML, NAV_CSS);
    loop {
        if matches!(
            warm.advance_budgeted_render(&mut session, 5_000.0),
            crate::BudgetAdvance::Complete
        ) {
            break;
        }
    }
    assert_eq!(
        session.timings().parse_ms,
        0.0,
        "热导航必须命中指纹缓存（parse 段零耗时；MISS 则本钉前提不成立）"
    );
    let _ = session.take_result();
    // 命中还原 + 脚本重放一次：效应必须单次（不翻倍）。
    warm.render_with_dom_mutations_persistent(&df1_append_probe_mutations(), NAV_CSS, None)
        .expect("script mutation re-applies after hit");
    assert_eq!(
        df1_body_child_count(&warm),
        4,
        "命中还原纯净树 + 脚本单次效应：探针不得翻倍（翻倍 = 突变泄漏进缓存）"
    );
}

/// DF-3：命中帧 `RenderResult.timings` 必须覆写为命中计时——与外层 session 计时
/// 同值（RenderResult.timings 语义 = 产生该帧的实际耗时；不得携带冷航全量幻影
/// 毫秒，消费方 apps/renderer resize 诊断读该值）。
#[test]
fn s1_fingerprint_hit_result_timings_matches_hit_not_cold() {
    let mut warm = RenderPipeline::new(320.0, 400.0);
    let cold = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    assert!(cold.timings.parse_ms > 0.0, "冷航前置：全量路径 parse 有耗时");
    let mut session = crate::BudgetedRenderSession::new(NAV_A_HTML, NAV_CSS);
    loop {
        if matches!(
            warm.advance_budgeted_render(&mut session, 5_000.0),
            crate::BudgetAdvance::Complete
        ) {
            break;
        }
    }
    assert_eq!(session.timings().parse_ms, 0.0, "热航命中前提");
    let result = session.take_result().expect("complete session has result");
    assert_eq!(
        result.timings.total_ms,
        session.timings().total_ms,
        "命中帧 result.timings 必须与 session.timings（命中计时）同值"
    );
    assert_eq!(result.timings.parse_ms, 0.0, "命中帧不得携带冷航 parse 毫秒");
}

// ── S1 中断续推回填键返修回归钉（DF2-1，红→绿证据见运行目录 evidence/t21-s2-red/green）──

/// DF2-1 场景文档：单个无尺寸属性 img（固有尺寸驱动布局，图片表落库前后
/// 布局/绘制面均可区分）。
const DF2_IMG_HTML: &str = "<html><body><img src=\"https://a.example/pic.png\"></body></html>";

/// TV2-R1 场景文档：img + 后随块级 div——div 的 Y 位置由 img 布局盒高度决定
/// （固有注入缺失时 img 盒塌缩、div 上移；注入 200×40 后 div 下移），使混合帧
/// （E0 布局 + E1 绘制）与两个纯参考帧像素可区分。
const TV2R1_FLOW_HTML: &str = "<html><body><img src=\"https://a.example/pic.png\"><div style=\"height:20px;background-color:#0000ff\"></div></body></html>";

/// DF2-1 → TV2-R1 演进钉：会话中段环境变更（图片分批落表）的回填语义。
///
/// DF2-1（S2 返修）曾要求回填键以 Complete 当刻环境重算（本测试旧名
/// `..._backfills_under_completion_env` 断言 E1 命中）；TV2-R1（S3 返修）
/// 收紧为保守跳过：重算键 ≠ 起点键一律不回填。本构造中全部步在落表后
/// 推进、帧实为纯净 E1，跳过属保守成本（该时序损失一次缓存机会）而非
/// 缺陷——因 [Layout,Paint) 中断带的混合帧无法与纯净帧从外部区分（键等
/// 是唯一安全门），见
/// `s1_fingerprint_layout_paint_boundary_env_change_skips_backfill` 的像素
/// 偏离证据。修后语义：两个环境的热导航都 MISS 全量且与关缓存参考逐位
/// 一致（红→绿证据：运行目录 evidence/t21-s3-red-interleave2 与后续绿跑）。
#[test]
fn s1_fingerprint_mid_session_env_change_skips_backfill_misses_both_envs() {
    let mut ctx = RasterCtx::new();
    let mut warm = RenderPipeline::new(320.0, 400.0);
    let img_key = crate::paint::image_resource_key("https://a.example/pic.png", warm.document_url());
    let mut img_sizes = std::collections::HashMap::new();
    img_sizes.insert(img_key, (200.0f32, 40.0f32));
    let mut img_colors = std::collections::HashMap::new();
    img_colors.insert(img_key, [255u8, 0, 0, 255]);

    // 零预算首 advance：起点键 K0 定格（E0 = 图片未到），确定性零步推进
    //（deadline = now，步循环不进入，模拟 16ms 生产预算下的中断续推形态）。
    let mut session = crate::BudgetedRenderSession::new(DF2_IMG_HTML, NAV_CSS);
    assert!(
        matches!(
            warm.advance_budgeted_render(&mut session, 0.0),
            crate::BudgetAdvance::InProgress
        ),
        "零预算首 advance 必须定格键后中断（场景前提）"
    );

    // 图片分批到达（E0→E1，会话中段落表），续推至 Complete：帧由 E1 布局/
    // 绘制产出，回填发生在此处（被测点）。
    warm.set_image_sizes(img_sizes.clone());
    warm.set_image_solid_colors(img_colors.clone());
    loop {
        if matches!(
            warm.advance_budgeted_render(&mut session, 5_000.0),
            crate::BudgetAdvance::Complete
        ) {
            break;
        }
    }
    let _ = session.take_result();

    // 旧环境热导航（表复位 = 重导航后图片未重新到达）：环境变更后的条目
    // 不得被旧环境会话命中——必须 MISS 全量，且输出与关缓存全量逐位一致。
    warm.set_image_sizes(std::collections::HashMap::new());
    warm.set_image_solid_colors(std::collections::HashMap::new());
    let hot_e0 = fingerprint_session(&mut warm, DF2_IMG_HTML, NAV_CSS);
    assert!(
        hot_e0.timings.parse_ms > 0.0,
        "旧环境查找必须 MISS 全量（parse_ms=0 即旧键装载了新环境帧）"
    );
    let ref_e0 = cold_frame(DF2_IMG_HTML, NAV_CSS);
    assert_frames_bitwise_equal(
        &ctx.rasterize(&hot_e0),
        &ctx.rasterize(&ref_e0),
        "DF2-1 旧环境热导航 vs 关缓存全量",
    );

    // 新环境热导航（E1 恢复）：TV2-R1 保守跳过——起点键 ≠ 重算键不回填，
    // 即便本构造的完成帧实为纯净 E1（保守成本，见函数级注释）；必须 MISS
    // 全量，输出与关缓存全量逐位一致。
    warm.set_image_sizes(img_sizes.clone());
    warm.set_image_solid_colors(img_colors.clone());
    let hot_e1 = fingerprint_session(&mut warm, DF2_IMG_HTML, NAV_CSS);
    assert!(
        hot_e1.timings.parse_ms > 0.0,
        "新环境热导航必须 MISS 全量（parse_ms=0 = 环境变更会话仍被回填，TV2-R1 保守门失效）"
    );
    let mut ref_e1_pipeline = RenderPipeline::new(320.0, 400.0);
    ref_e1_pipeline.set_image_sizes(img_sizes);
    ref_e1_pipeline.set_image_solid_colors(img_colors);
    let ref_e1 = ref_e1_pipeline.render_html(DF2_IMG_HTML, NAV_CSS);
    assert_frames_bitwise_equal(
        &ctx.rasterize(&hot_e1),
        &ctx.rasterize(&ref_e1),
        "DF2-1 新环境热导航 vs 关缓存全量",
    );
}

/// TV2-R1（medium 根修钉）：中断落在 [Layout 完成, Paint 开始) 步边界 + 同 tick
/// 图片批落表 → 续推产出的帧是混合帧（布局几何 = 旧环境、绘制 = 新环境），不得
/// 入缓存。
///
/// 缺陷时序（返修前红）：会话起点键 K0 定格（E0 = 图片未到）→ 微预算逐 advance
/// 推进至 Paint 步前中断（布局已按 E0 定形）→ 同 tick 图片批落表（E0→E1）→ 续
/// 推 Paint 完成：绘制消费 E1 表、布局几何仍是 E0 产物，混合帧以 Complete 当刻
/// 键 K1 回填——键值不同刻。E1 热导航命中装载混合帧，输出 ≠ 关缓存全量，同键
/// 覆盖还冲掉干净条目。修后语义（回填保守跳过）：Complete 重算键 ≠ 会话起点键
/// = 环境在会话中段变更过 → 不插入；E0/E1 热导航均 MISS 全量且逐位等关缓存参
/// 考。生产可达性：webview async_load tick 序 advance → poll_images 落表 → 同
/// tick 二次 advance 续推同一 session。
#[test]
fn s1_fingerprint_layout_paint_boundary_env_change_skips_backfill() {
    let mut ctx = RasterCtx::new();
    let mut warm = RenderPipeline::new(320.0, 400.0);
    let img_key = crate::paint::image_resource_key("https://a.example/pic.png", warm.document_url());
    let mut img_sizes = std::collections::HashMap::new();
    img_sizes.insert(img_key, (200.0f32, 40.0f32));
    let mut img_colors = std::collections::HashMap::new();
    img_colors.insert(img_key, [255u8, 0, 0, 255]);

    // 零预算首 advance：起点键 K0 定格（E0 = 图片未到），零步推进（场景前提）。
    let mut session = crate::BudgetedRenderSession::new(TV2R1_FLOW_HTML, NAV_CSS);
    assert!(
        matches!(
            warm.advance_budgeted_render(&mut session, 0.0),
            crate::BudgetAdvance::InProgress
        ),
        "零预算首 advance 必须定格键后中断（场景前提）"
    );

    // 微预算逐步推进到 [Layout 完成, Paint 开始) 步边界：微预算（1μs）下每步
    // 实际耗时超过预算 → 每次 advance 恰推进一个步；停在 Paint 步 = 布局已按
    // E0 定形、绘制未开始（混合帧的布局半边）。
    let mut steps = 0;
    while session.step() != crate::BudgetStep::Paint {
        assert!(steps < 16, "微预算推进必须停在 Paint 步边界（场景前提）");
        steps += 1;
        let _ = warm.advance_budgeted_render(&mut session, 0.001);
    }

    // 同 tick 图片批落表（E0→E1）+ 续推 Paint 完成（生产 tick 序）：混合帧在
    // Paint 臂末到达回填点（被测点）。
    warm.set_image_sizes(img_sizes.clone());
    warm.set_image_solid_colors(img_colors.clone());
    loop {
        if matches!(
            warm.advance_budgeted_render(&mut session, 5_000.0),
            crate::BudgetAdvance::Complete
        ) {
            break;
        }
    }
    let _ = session.take_result();

    // E1 热导航（表保持 = 图片已到的新环境）：缺陷形态 = 命中混合帧回填条目
    //（parse_ms=0）且输出与关缓存参考逐位不等（后随 div 的 Y 位置由 img 布局
    // 盒高度决定——E0 布局下 img 固有注入缺失、div 上移，E1 参考帧 div 在
    // img 200×40 之下）；修后形态 = MISS 全量且逐位等。
    warm.set_image_sizes(img_sizes.clone());
    warm.set_image_solid_colors(img_colors.clone());
    let hot_e1 = fingerprint_session(&mut warm, TV2R1_FLOW_HTML, NAV_CSS);
    let mut ref_e1_pipeline = RenderPipeline::new(320.0, 400.0);
    ref_e1_pipeline.set_image_sizes(img_sizes.clone());
    ref_e1_pipeline.set_image_solid_colors(img_colors.clone());
    let ref_e1 = ref_e1_pipeline.render_html(TV2R1_FLOW_HTML, NAV_CSS);
    let px_hot_e1 = ctx.rasterize(&hot_e1);
    let px_ref_e1 = ctx.rasterize(&ref_e1);
    let e1_differs = px_hot_e1.data != px_ref_e1.data;
    assert!(
        hot_e1.timings.parse_ms > 0.0,
        "E1 热导航必须 MISS 全量（实测 parse_ms={}，命中回填条目={}，输出与关缓存参考逐位不等={} —— TV2-R1 混合帧缺陷特征）",
        hot_e1.timings.parse_ms,
        hot_e1.timings.parse_ms == 0.0,
        e1_differs
    );
    assert_frames_bitwise_equal(&px_hot_e1, &px_ref_e1, "TV2-R1 E1 热导航 vs 关缓存全量");

    // E0 热导航（表复位 = 重导航后图片未重新到达）：同样 MISS 全量逐位等——
    // 不得命中任何残留/污染条目。
    warm.set_image_sizes(std::collections::HashMap::new());
    warm.set_image_solid_colors(std::collections::HashMap::new());
    let hot_e0 = fingerprint_session(&mut warm, TV2R1_FLOW_HTML, NAV_CSS);
    assert!(
        hot_e0.timings.parse_ms > 0.0,
        "E0 热导航必须 MISS 全量（parse_ms=0 = 命中污染条目）"
    );
    let ref_e0 = cold_frame(TV2R1_FLOW_HTML, NAV_CSS);
    assert_frames_bitwise_equal(
        &ctx.rasterize(&hot_e0),
        &ctx.rasterize(&ref_e0),
        "TV2-R1 E0 热导航 vs 关缓存全量",
    );

    // 守卫：保守跳过不误伤正常回填路径——环境稳定会话（起点键 == 当刻键）正常
    // 回填，同环境后续会话可命中（四段零耗）。
    let _ = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    let stable_hit = fingerprint_session(&mut warm, NAV_A_HTML, NAV_CSS);
    assert_eq!(
        stable_hit.timings.parse_ms, 0.0,
        "环境稳定会话必须正常回填并可被同环境命中（保守跳过不得禁用正常路径）"
    );
}

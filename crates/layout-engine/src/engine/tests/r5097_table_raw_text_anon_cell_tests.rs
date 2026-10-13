//! R5097（CSS Tables §17.2.1 匿名 cell）：display:table/inline-table 根的直接散文本子
//! （无 cell/row 元素包装）不再塌 0×0——build_grid 零行空表臂按匿名 cell 语义把表
//! 收缩适应到文本 max-content 宽 + IFC 高。旧路径 shrink_empty_table_to_padding_border
//! 清零、文本绘于表 y 处与后继块重叠。
//! driving: css/css-display/run-in run-in-table-between-001/002 +
//! run-in-inline-table-between-001/002（R5096 合并决策归正后余差转本渲染域）。
//! kill-switch: `ZW_TABLE_RAW_TEXT_ANON_CELL=0` 回退旧行为。
//!
//! 注：样式走 inline style 属性——直构管线的 compute_styles(&doc, &[]) 不含文档
//! `<style>` 块（作者样式表由 runner 外层收集传入），R5081「空转绿」教训同源。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

fn layout(html: &str) -> (Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// driving 形态（run-in-table-between 族去 run-in 化最小形）：块 + 纯文本表 + 块。
const BETWEEN_FORM: &str = r#"<html><body style="margin:0">
  <div>header</div>
  <span id="tbl" style="display:table">Some text.</span>
  <div id="target">Start of block.</div>
</body></html>"#;

/// 正例：`#tbl` 直接散文本子 → 匿名 cell 收缩适应（content 高 > 0、宽收缩适应），
/// 后继块排在表底边之下（不重叠）。
#[test]
fn test_r5097_raw_text_table_sized_to_anon_cell() {
    // kill-switch 测试会进程级改 env——本组测试统一持锁串行，防并行读到 env=0。
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let (doc, result) = layout(BETWEEN_FORM);
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    let target = find("target", &doc, &result.root).expect("target box");
    assert!(
        tbl.height > 1.0,
        "R5097: 纯文本表应按匿名 cell 语义取得内容高（旧路径塌 0×0），got {}",
        tbl.height
    );
    assert!(
        tbl.width > 1.0 && tbl.width < 400.0,
        "R5097: 纯文本表应收缩适应到文本 max-content 宽（非 0 非满容器），got {}",
        tbl.width
    );
    assert!(
        target.y >= tbl.y + tbl.height - 0.5,
        "R5097: 后继块应排在表底边之下（旧路径文本绘于表 y 处与后继块重叠），\
         table y={} h={} target y={}",
        tbl.y,
        tbl.height,
        target.y
    );
}

/// kill-switch：`ZW_TABLE_RAW_TEXT_ANON_CELL=0` 回退旧行为（纯文本表塌 0×0、
/// 后继块顶到表 y 重叠——run-in-between 族的 3.17% 签名）。
#[test]
fn test_r5097_kill_switch_restores_collapse() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // SAFETY: 锁内同步设置/读取；无其他测试并发读 ZW_TABLE_RAW_TEXT_ANON_CELL。
    unsafe {
        std::env::set_var("ZW_TABLE_RAW_TEXT_ANON_CELL", "0");
    }
    let (doc, result) = layout(BETWEEN_FORM);
    unsafe {
        std::env::remove_var("ZW_TABLE_RAW_TEXT_ANON_CELL");
    }
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    let target = find("target", &doc, &result.root).expect("target box");
    assert!(
        tbl.height < 1.0,
        "R5097 kill-switch: 应回退旧路径（塌 0×0 边框壳），got {}",
        tbl.height
    );
    assert!(
        (target.y - tbl.y).abs() < 0.5,
        "R5097 kill-switch: 旧路径后继块顶到表 y（重叠签名），got target y={} vs table y={}",
        target.y,
        tbl.y
    );
}

/// gate 负例：带 inline 元素子的表不触达本臂（混合内容匿名 cell 归后续切片）——
/// 维持旧 shrink_table_to_block_content no-op 臂的 taffy IFC 几何（block 级表满容器宽）。
#[test]
fn test_r5097_element_child_table_keeps_legacy_path() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0">
      <div>header</div>
      <div id="tbl" style="display:table"><span>inline child</span></div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    assert!(
        tbl.width > 700.0,
        "R5097: 元素子表不走散文本匿名 cell 臂（gate 收窄，混合内容归后续切片），\
         应维持旧路径满容器宽，got {}",
        tbl.width
    );
}

/// gate 负例：definite height 的纯文本表不触达本臂——显式高度语义（R4436 孤儿
/// caption 的行内 extent 等）归既有路径，测量值不得覆盖 definite height。
#[test]
fn test_r5097_definite_height_table_keeps_legacy_path() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0">
      <div>header</div>
      <div id="tbl" style="display:table;height:140px">Some text.</div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    assert!(
        (tbl.height - 140.0).abs() < 0.5,
        "R5097: definite height 表不走散文本匿名 cell 臂，高度应保留显式 140，got {}",
        tbl.height
    );
}

// wptfyi-css css-gaps 刀 D inline 站钉测（kd-inline-station PINS，2026-11-17）：
// shim 双面（T50 IDL / M face）gap-rule/inset 简写家族 inline 写读删往返。写面
// 展开落名单 longhand（简写键不占声明位，cssText 记账形；pair 值 `overlap-join
// 10px` 逐槽拆分投影）；读面从 raw 声明串重组 specified 串——名单 longhand 缺
// 任一即空串（Chrome 简写 serialize 同口径）、8 longhand 全值形走双轴四槽跨轴
// 等值模型；removeProperty = 名单 longhand 一并清。单点单源
// dom_bindings::css_style_declaration（expand/reconstruct 纯函数）+ part03 接线。
// driving: css-gaps 语料 rule-inset-valid.html 10 REG（8 longhand 读重组 len
// match 缺 8 臂恒空串）+ rule-inset-{shorthand,cap-junction-*,start-end-*} 轮
// 次页增益面；gCS 同模型钉 computed_style.rs::rule_inset_full_shorthand_cross_
// axis_equality。诊断链：V8 探针 direct 回调三分（expand 8-pair JSON ✓ /
// reconstruct col ✓ / reconstruct 8 → '' ✗）定谳 shim 接线无虞、单点缺臂。
#[test]
fn test_kd_rule_inset_inline_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__kdPins = [];
// expReads: [名, 期望读值] 对列表（逐 pin 自选读面名——家族读要求名单 longhand
// 全在，缺任一即空串）。
function kdPin(id, setupFn, expReads, expCssText) {
  var d = document.createElement('div');
  document.body.appendChild(d);
  var err = null;
  try { setupFn(d); } catch (e) { err = String(e); }
  var gotReads = [];
  for (var ri = 0; ri < expReads.length; ri++) {
    gotReads.push(d.style.getPropertyValue(expReads[ri][0]));
  }
  var got = { reads: gotReads, cssText: d.style.cssText };
  document.body.removeChild(d);
  globalThis.__kdPins.push({ id: id, err: err, got: got,
    expNames: expReads, expCssText: expCssText });
}

// 全值简写 rule-inset：单值全等塌缩；展开 8 longhand、简写键不占声明位。
kdPin("kd-ri-single-collapse", function (d) { d.style.setProperty('rule-inset', '10px'); },
  [['rule-inset', '10px'], ['column-rule-inset', '10px'], ['row-rule-inset', '10px']],
  'column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 10px; column-rule-inset-junction-start: 10px; column-rule-inset-junction-end: 10px; row-rule-inset-cap-start: 10px; row-rule-inset-cap-end: 10px; row-rule-inset-junction-start: 10px; row-rule-inset-junction-end: 10px');
// 全形四槽（`/` 双对）：侧内等值不折叠恒全形，双轴等值读原形。
kdPin("kd-ri-full-form", function (d) { d.style.setProperty('rule-inset', '10px 20px / 30px -10px'); },
  [['rule-inset', '10px 20px / 30px -10px'], ['column-rule-inset', '10px 20px / 30px -10px']],
  'column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 20px; column-rule-inset-junction-start: 30px; column-rule-inset-junction-end: -10px; row-rule-inset-cap-start: 10px; row-rule-inset-cap-end: 20px; row-rule-inset-junction-start: 30px; row-rule-inset-junction-end: -10px');
// 单轴对形（`/` 单侧）：`10px / 20px` 落 `10px 10px / 20px 20px` canonical（侧内
// 复制不折叠——bidirectional-shorthands 页 oracle 同源）。
kdPin("kd-ri-axis-pair", function (d) { d.style.setProperty('rule-inset', '10px / 20px'); },
  [['rule-inset', '10px 10px / 20px 20px']],
  'column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 10px; column-rule-inset-junction-start: 20px; column-rule-inset-junction-end: 20px; row-rule-inset-cap-start: 10px; row-rule-inset-cap-end: 10px; row-rule-inset-junction-start: 20px; row-rule-inset-junction-end: 20px');
// 负长/overlap-join 组合形合法透传往返（rule-inset-valid 页 REG 形族）——pair
// 值 `overlap-join 10px` 展开逐槽拆分（槽语法 `overlap-join | <length-percentage>`
// 单槽单分量，css-gaps-1 §inset）。
kdPin("kd-ri-negative", function (d) { d.style.setProperty('rule-inset', '-20px'); },
  [['rule-inset', '-20px']],
  'column-rule-inset-cap-start: -20px; column-rule-inset-cap-end: -20px; column-rule-inset-junction-start: -20px; column-rule-inset-junction-end: -20px; row-rule-inset-cap-start: -20px; row-rule-inset-cap-end: -20px; row-rule-inset-junction-start: -20px; row-rule-inset-junction-end: -20px');
kdPin("kd-ri-overlap-join-pair", function (d) { d.style.setProperty('rule-inset', 'overlap-join 10px / overlap-join 10px'); },
  [['rule-inset', 'overlap-join 10px / overlap-join 10px'], ['column-rule-inset-cap-start', 'overlap-join']],
  'column-rule-inset-cap-start: overlap-join; column-rule-inset-cap-end: 10px; column-rule-inset-junction-start: overlap-join; column-rule-inset-junction-end: 10px; row-rule-inset-cap-start: overlap-join; row-rule-inset-cap-end: 10px; row-rule-inset-junction-start: overlap-join; row-rule-inset-junction-end: 10px');
// 双轴失配（独立 longhand 写）→ 全值读空串（跨轴等值模型 None 臂——gCS
// rule-inset 臂 target2 oracle 同口径）；列轴自读照常。
kdPin("kd-ri-cross-axis-mismatch", function (d) {
  d.style.setProperty('column-rule-inset', '10px');
  d.style.setProperty('row-rule-inset', '20px');
},
  [['rule-inset', ''], ['column-rule-inset', '10px'], ['row-rule-inset', '20px']],
  'column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 10px; column-rule-inset-junction-start: 10px; column-rule-inset-junction-end: 10px; row-rule-inset-cap-start: 20px; row-rule-inset-cap-end: 20px; row-rule-inset-junction-start: 20px; row-rule-inset-junction-end: 20px');
// 槽对简写 rule-inset-cap：轴内对等值塌缩双轴全等读该值；槽读 `10px 20px`
// （cap 对不折叠）；junction 缺位 → 全值/列轴读空串。
kdPin("kd-ri-cap-pair", function (d) { d.style.setProperty('rule-inset-cap', '10px 20px'); },
  [['rule-inset-cap', '10px 20px'], ['column-rule-inset-cap', '10px 20px'],
   ['rule-inset', ''], ['column-rule-inset', '']],
  'column-rule-inset-cap-start: 10px; column-rule-inset-cap-end: 20px; row-rule-inset-cap-start: 10px; row-rule-inset-cap-end: 20px');
// removeProperty = 名单 longhand 一并清（R3211 简写 remove 语义）。
kdPin("kd-ri-remove-family", function (d) {
  d.style.setProperty('rule-inset', '10px');
  d.style.removeProperty('rule-inset');
},
  [['rule-inset', ''], ['column-rule-inset', ''], ['row-rule-inset', '']],
  '');
// removeProperty 家族键（清列轴四槽，行轴保形；行轴全名单在 → 行读 `10px`，
// 全值读列缺位空串）。
kdPin("kd-ri-remove-column-family", function (d) {
  d.style.setProperty('rule-inset', '10px');
  d.style.removeProperty('column-rule-inset');
},
  [['rule-inset', ''], ['column-rule-inset', ''], ['row-rule-inset', '10px']],
  'row-rule-inset-cap-start: 10px; row-rule-inset-cap-end: 10px; row-rule-inset-junction-start: 10px; row-rule-inset-junction-end: 10px');

var bad = [];
for (var pi = 0; pi < globalThis.__kdPins.length; pi++) {
  var c = globalThis.__kdPins[pi];
  var ok = !c.err && c.got.cssText === c.expCssText
    && c.got.reads.length === c.expNames.length;
  var detail = '';
  if (ok) {
    for (var ri = 0; ri < c.got.reads.length; ri++) {
      if (c.got.reads[ri] !== c.expNames[ri][1]) {
        ok = false;
        detail += " " + c.expNames[ri][0] + ": got=" + JSON.stringify(c.got.reads[ri])
          + " exp=" + JSON.stringify(c.expNames[ri][1]);
      }
    }
  }
  if (!ok) {
    bad.push(c.id + ": err=" + c.err + detail
      + " cssText=" + JSON.stringify(c.got.cssText) + "/" + JSON.stringify(c.expCssText));
  }
}
if (bad.length) throw "KDRI-PIN MISMATCH x" + bad.length + " | " + bad.join(" || ");
"##,
        )
        .expect("kd rule-inset inline pins: all forms must hold");
}

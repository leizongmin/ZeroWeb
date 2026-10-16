//! css/cssom 本地自建 inline reftest（siteopt qq-20261016-r1 切片 A）——
//! constructable stylesheets（CSSOM）视觉行为。

use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── adoptedStyleSheets 视觉生效（constructable CSSStyleSheet）──
    // 主臂：页面脚本 `new CSSStyleSheet()` + `replaceSync` + `document.adoptedStyleSheets`
    // 赋值——adopted 规则进入 cascade，div 绿底。缺陷态（无 CSSStyleSheet 构造器 /
    // adopted 表不进合成源）：脚本抛错或规则缺席 → div 无背景，与 ref 像素不等。
    // ref 臂：同规则 `<style>` 内联（author 源视觉等价）。
    // spec：https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssstylesheet 、
    // https://drafts.csswg.org/cssom/#dom-documentorshadowroot-adoptedstylesheets 。
    // 文件孪生（make reftest-upstream 域）：
    // tests/wpt-runner/local-reftests/css/cssom/adopted-stylesheet-background-zw-001*.html。
    InlineReftestDef {
        id: "css-cssom/adopted-stylesheet-background-zw-001",
        category: ReftestCategory::Layout,
        test_html: "<html><head><script>try { var s = new CSSStyleSheet(); s.replaceSync('div { background: rgb(0, 128, 0); width: 100px; height: 50px; }'); document.adoptedStyleSheets = [s]; } catch (e) { }</script></head><body><div></div></body></html>",
        ref_html: "<html><head><style>div { background: rgb(0, 128, 0); width: 100px; height: 50px; }</style></head><body><div></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

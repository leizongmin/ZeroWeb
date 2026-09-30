//! CSS Values（数学函数 calc/min/max/clamp）inline reftest。

use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

static REFTESTS: &[InlineReftestDef] = &[
    // css-values-4 §8.3 min() 混合百分比形态在 max-width 上的 used 值：
    // `max-width: min(284px, 100% - 24px)` 于 380px float 包含块内 = min(284, 356) =
    // 284px。converter 此前对 Min 形态无 taffy Dimension 表示即回退 length(0.0)，
    // float（CSS2 §10.3.5 shrink-to-fit 受 max-width 钳制）随之收缩 0 宽——行标题
    // 不可见（热榜行仅徽标墨迹缺陷的最小触发形状：float li > float a）。
    // 判别信号与字体无关：a.tag 主体 lime 背景 284×36、徽标 silver 24×36 位置
    // （x=284 vs 修复前 x=0）；4 行累计 diff ≈ 9.2% 对 Layout 1% 容差 ≈9× 裕度
    // （修复前实测 red 9.2%量级 / 修复后 0.00%，负控制记录 run 目录 diag/evidence/slice11/）。
    InlineReftestDef {
        id: "css-values/min-mixed-percentage-float-shrink-to-fit",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}.ul{list-style:none;margin:0;padding:0;width:760px}.li{float:left;clear:both;width:380px;height:36px;line-height:36px;font-size:14px}.tag{float:left;max-width:min(284px, 100% - 24px);height:36px;line-height:36px;background:lime;white-space:nowrap;overflow:hidden}.mk{display:inline-block;width:24px;height:36px;background:silver}</style></head><body><ul class=\"ul\"><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li></ul></body></html>",
        ref_html: "<html><head><style>body{margin:0}.ul{list-style:none;margin:0;padding:0;width:760px}.li{float:left;clear:both;width:380px;height:36px;line-height:36px;font-size:14px}.tag{float:left;max-width:284px;height:36px;line-height:36px;background:lime;white-space:nowrap;overflow:hidden}.mk{display:inline-block;width:24px;height:36px;background:silver}</style></head><body><ul class=\"ul\"><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"tag\">热榜行标题文本样例热榜行标题文本样例</a><span class=\"mk\"></span></li></ul></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

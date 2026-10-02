use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 96-105: 盒模型进阶 (original 11) ──
    InlineReftestDef {
        id: "css-box/margin-collapse-siblings",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:20px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:20px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:10px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/padding-box-sizing",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:border-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:border-box;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-solid-colors",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:10px solid green;border-bottom:10px solid blue;border-left:10px solid yellow;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:10px solid green;border-bottom:10px solid blue;border-left:10px solid yellow;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-hidden-clips",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-visible-no-clip",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/max-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:500px;max-width:200px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:500px;max-width:200px;height:50px;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/min-height-expands",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;min-height:200px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;min-height:200px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/percentage-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50%;height:100%;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50%;height:100%;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/auto-margin-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:auto;margin-right:auto;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:auto;margin-right:auto;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/negative-margin-overlap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-20px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/box-sizing-border-box",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;border:5px solid black;box-sizing:border-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;border:5px solid black;box-sizing:border-box;background:red;\"></div></body></html>",
        is_match: true,
    },
    // ── 106-144: 盒模型扩展 (39 new entries) ──

    // --- Margin collapse: parent-child, siblings, multiple, zero ---
    InlineReftestDef {
        id: "css-box/margin-collapse-parent-child",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"margin-top:30px;background:red;\"><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"margin-top:30px;background:red;\"><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-three-siblings",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:30px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:30px;background:green;margin-top:20px;margin-bottom:40px;\"></div><div style=\"width:100px;height:30px;background:blue;margin-top:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:30px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:30px;background:green;margin-top:20px;margin-bottom:40px;\"></div><div style=\"width:100px;height:30px;background:blue;margin-top:10px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-zero-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:0px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:0px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:0px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:0px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-negative-positive",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-10px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-nested-prevented-by-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"padding-top:1px;background:red;\"><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"padding-top:1px;background:red;\"><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-siblings-equal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:25px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:25px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:25px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:25px;\"></div></body></html>",
        is_match: true,
    },
    // --- Box-sizing: content-box vs border-box ---
    InlineReftestDef {
        id: "css-box/content-box-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:content-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:content-box;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-box-with-padding-border",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:120px;height:70px;padding:10px;border:5px solid black;box-sizing:border-box;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:120px;height:70px;padding:10px;border:5px solid black;box-sizing:border-box;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/content-box-with-border",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;border:10px solid red;box-sizing:content-box;background:yellow;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;border:10px solid red;box-sizing:content-box;background:yellow;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-box-vs-content-box-same-visual",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:120px;height:70px;padding:10px;border:5px solid black;box-sizing:border-box;background:orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;border:5px solid black;box-sizing:content-box;background:orange;\"></div></body></html>",
        is_match: true,
    },
    // --- Padding: individual sides, percentage ---
    InlineReftestDef {
        id: "css-box/padding-individual-sides",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding-top:10px;padding-right:20px;padding-bottom:30px;padding-left:40px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding-top:10px;padding-right:20px;padding-bottom:30px;padding-left:40px;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/padding-percentage-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:red;\"><div style=\"width:100px;height:50px;padding:10%;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:red;\"><div style=\"width:100px;height:50px;padding:10%;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/padding-shorthand-vertical-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding:10px 20px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding:10px 20px;background:green;\"></div></body></html>",
        is_match: true,
    },
    // --- Border: different widths per side, border with background ---
    InlineReftestDef {
        id: "css-box/border-different-widths",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:5px solid red;border-right:10px solid green;border-bottom:15px solid blue;border-left:20px solid yellow;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:5px solid red;border-right:10px solid green;border-bottom:15px solid blue;border-left:20px solid yellow;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-with-background",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:10px solid black;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:10px solid black;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-uniform-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:10px solid purple;background:yellow;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:10px solid purple;background:yellow;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-no-shrink-content-box",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;border:5px solid red;box-sizing:content-box;background:lime;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;border:5px solid red;box-sizing:content-box;background:lime;\"></div></body></html>",
        is_match: true,
    },
    // --- Width/Height: auto, percentage, max/min constraints ---
    InlineReftestDef {
        id: "css-box/width-auto-fills-parent",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:red;\"><div style=\"height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:red;\"><div style=\"height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/percentage-height-with-parent",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;background:red;\"><div style=\"width:100%;height:50%;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;background:red;\"><div style=\"width:100%;height:50%;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/max-height-shrinks",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;max-height:100px;background:orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;max-height:100px;background:orange;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/min-width-expands",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;min-width:150px;height:50px;background:teal;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;min-width:150px;height:50px;background:teal;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/max-width-larger-than-width-no-effect",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;max-width:300px;height:50px;background:navy;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;max-width:300px;height:50px;background:navy;\"></div></body></html>",
        is_match: true,
    },
    // --- Overflow: hidden, visible, scroll, auto with clipping ---
    InlineReftestDef {
        id: "css-box/overflow-scroll-establishes-scrollbar",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:scroll;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:scroll;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-auto-with-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:auto;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:auto;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-auto-no-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;overflow:auto;background:gray;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;overflow:auto;background:gray;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-hidden-text-clip",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:20px;overflow:hidden;background:white;\">ABCDEFGHIJKLMNOPQRSTUVWXYZ</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:20px;overflow:hidden;background:white;\">ABCDEFGHIJKLMNOPQRSTUVWXYZ</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-visible-default",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:rgba(255,0,0,0.5);\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:rgba(255,0,0,0.5);\"></div></div></body></html>",
        is_match: true,
    },
    // --- Auto margin for centering ---
    InlineReftestDef {
        id: "css-box/auto-margin-center-fixed-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:red;margin:0 auto;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:red;margin:0 auto;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/auto-margin-center-in-container",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;background:gray;\"><div style=\"width:100px;height:50px;margin:0 auto;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;background:gray;\"><div style=\"width:100px;height:50px;margin:0 auto;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/auto-margin-right-only",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:0;margin-right:auto;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:0;margin-right:auto;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/auto-margin-both-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:40px;margin-left:auto;margin-right:auto;background:purple;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:40px;margin-left:auto;margin-right:auto;background:purple;\"></div></body></html>",
        is_match: true,
    },
    // --- Negative margins: overlap, pull-up ---
    InlineReftestDef {
        id: "css-box/negative-margin-pull-up",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:60px;background:red;\"></div><div style=\"width:100px;height:60px;background:blue;margin-top:-30px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:60px;background:red;\"></div><div style=\"width:100px;height:60px;background:blue;margin-top:-30px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/negative-margin-left-overlap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-left:-30px;margin-top:0;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-left:-30px;margin-top:0;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/negative-margin-sibling-complete-overlap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-50px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-50px;\"></div></body></html>",
        is_match: true,
    },
    // --- Nested box model: child in parent with padding/border ---
    InlineReftestDef {
        id: "css-box/nested-padding-child-offset",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:20px;background:red;\"><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:20px;background:red;\"><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/nested-border-child-position",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;border:10px solid black;background:red;\"><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;border:10px solid black;background:red;\"><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/nested-padding-border-combined",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:15px;border:5px solid black;background:gray;\"><div style=\"width:100px;height:100px;background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:15px;border:5px solid black;background:gray;\"><div style=\"width:100px;height:100px;background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/nested-border-box-child-fill",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:20px;border:10px solid black;box-sizing:border-box;background:red;\"><div style=\"width:100%;height:100%;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;padding:20px;border:10px solid black;box-sizing:border-box;background:red;\"><div style=\"width:100%;height:100%;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // --- Mismatch cases: different box model values produce different visuals ---
    InlineReftestDef {
        id: "css-box/mismatch-content-box-vs-border-box",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:20px;box-sizing:content-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:20px;box-sizing:border-box;background:red;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-box/mismatch-overflow-hidden-vs-visible",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-box/margin-collapse-siblings-30-20",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/mismatch-negative-vs-positive-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-box/mismatch-different-border-widths",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:5px solid black;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:20px solid black;background:red;\"></div></body></html>",
        is_match: false,
    },
    // ── slice12（R4919）：inline-block 收缩到 fit-content 时后代文本按自身
    // font-size 量测（baidu 顶部导航「更多」竖排簇）──
    // 回归形态：容器 12px ⊃ 锚 13px 文本「更多」——intrinsic walk 把后代文本
    // 误按容器字号量测（24 vs 真值 26，css-sizing-3 intrinsic size）→ 收缩过窄
    // → 文本折行竖排。形态：.nav-item inline-block（继承 36px）> a 39px 拉丁
    // 双词组（即 12⊃13 的等比放大——runner 字体环境无 CJK 字形，信号由可见
    // 字形/背景承载；bar + 大 line-height 使折行信号 ≥5× Layout 1% 阈值）。
    // 邻臂（.big 同字号 inline-block，无嵌套字体分裂）与主臂同排——其自身量测免疫
    //（R4919 关断下仍按 39px 量测）恒单行；但主臂折行会经基线对齐**纵向推移**邻臂
    //（88854px 差值含该位移贡献）——邻近变体负控制判别口径 = 两状态恒单行，非
    // 「位置不受影响」。ref 页 .nav-item 加 white-space:nowrap 钳单行；修复态两页
    // 逐像素相等。
    // 负控制（800×600=480000px，Layout 阈值 1%，本套件实测口径）：default 0 diff；
    // ZW_INTRINSIC_PERFONT=0 → 88854px = 18.51%（18.5×）。
    // 文件孪生（make reftest-upstream 域 + 几何 dump 用）：
    // tests/wpt-runner/local-reftests/css/css-sizing/intrinsic-nested-font-inline-block-zw-001*.html。
    InlineReftestDef {
        id: "css-box/intrinsic-nested-font-inline-block-zw-001",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0;font:36px/46px Arial,sans-serif}.row{margin:8px}.nav-item{display:inline-block;background:#3fbf3f}.nav-item a{font-size:39px;line-height:138px;color:#222}.big{font-size:39px;line-height:138px;background:#e6a23c}</style></head><body><div class=\"row\"><span class=\"nav-item\"><a>abcdef ghijkl</a></span><span class=\"nav-item big\" style=\"margin-left:40px\">abcdef ghijkl</span></div></body></html>",
        ref_html: "<html><head><style>body{margin:0;font:36px/46px Arial,sans-serif}.row{margin:8px}.nav-item{display:inline-block;background:#3fbf3f;white-space:nowrap}.nav-item a{font-size:39px;line-height:138px;color:#222}.big{font-size:39px;line-height:138px;background:#e6a23c}</style></head><body><div class=\"row\"><span class=\"nav-item\"><a>abcdef ghijkl</a></span><span class=\"nav-item big\" style=\"margin-left:40px\">abcdef ghijkl</span></div></body></html>",
        is_match: true,
    },
    // ── slice19（R4938）：inline 元素内原子行内级后代保布局盒（R2156 skip 曾整棵丢弃）──
    // 回归形态：div > span > input(inline-block)——span 被 inline_box_model_coherence
    // skip 后原子行内级后代既无 taffy 子树也无 LayoutBox → 不绘制（IFC 行高仍由
    // collect_items 收集项撑起，信号 = input 矩形本体像素：360×40 + 240×60 = 28800px
    // ≈ 6.0% ≫ Layout 1% 阈值）。ref 页去 span 包装（原子 input 直接为块容器子，
    // R109 路径）——修复态两页逐像素相等。
    // CSS2 §9.2.1.1 inline formatting；§10.3.1 replaced inline。
    // 文件孪生（make reftest-upstream 域 + 几何 dump 用）：
    // tests/wpt-runner/local-reftests/css/CSS2/box-display/atomic-inline-in-inline-zw-001*.html。
    InlineReftestDef {
        id: "css-box/atomic-inline-in-inline-zw-001",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div><span><input style=\"width:360px;height:40px\"><input style=\"width:240px;height:60px\"></span></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div><input style=\"width:360px;height:40px\"><input style=\"width:240px;height:60px\"></div></body></html>",
        is_match: true,
    },
    // ── slice19 收尾（S-TE2）：跨行位钉——span 内双 200px input 在 300px 容器
    // 强制换行（pr61 testeff te3/te4 探针证两臂逐值一致：a=[0,0,208,48]
    // b=[0,48,208,48]，b.y=48 即第二行行位）。钉「提升臂产物跨行定位 ≡ 直接子
    // 形态」；base 双 input 矩形缺席 = 16000/480000 = 3.33%。
    // 文件孪生：local-reftests/css/CSS2/box-display/atomic-inline-in-inline-wrap-zw-002*.html。
    InlineReftestDef {
        id: "css-box/atomic-inline-in-inline-wrap-zw-002",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div style=\"width:300px\"><span><input style=\"width:200px;height:40px\"><input style=\"width:200px;height:40px\"></span></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div style=\"width:300px\"><input style=\"width:200px;height:40px\"><input style=\"width:200px;height:40px\"></div></body></html>",
        is_match: true,
    },
    // ── slice19 收尾（S-TE3）：svg 等价单行钉——svg 为替换元素（collect 判据
    // is_replaced_element 八类；sync 判据不含 svg 型），pr61 testeff te5/te6 探针
    // 证两臂逐值一致 [108,18,60,30]。钉「被 skip span 内 input+svg 并排 ≡ 直接子
    // 形态」（单变量 = 包装层有无）；base input+svg 矩形缺席 = 12000/480000 = 2.5%。
    // 文件孪生：local-reftests/css/CSS2/box-display/atomic-inline-in-inline-svg-zw-003*.html。
    InlineReftestDef {
        id: "css-box/atomic-inline-in-inline-svg-zw-003",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div><span><input style=\"width:200px;height:40px\"><svg width=\"100\" height=\"40\" style=\"vertical-align:top\"><rect width=\"100\" height=\"40\" fill=\"#c30\"/></svg></span></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div><input style=\"width:200px;height:40px\"><svg width=\"100\" height=\"40\" style=\"vertical-align:top\"><rect width=\"100\" height=\"40\" fill=\"#c30\"/></svg></div></body></html>",
        is_match: true,
    },
    // ── slice19 收尾（S-TE3 多行探针案，先探后钉）：svg 换行到第二行时行位是否
    // 随 IFC 片段同步（sync 判据不含 svg 型，te 探针未覆盖多行）。两臂一致才保留
    // 本钉；不一致则撤钉记挂账。base 矩形缺席 = 8000+8000 = 16000/480000 = 3.33%。
    // 文件孪生：local-reftests/css/CSS2/box-display/atomic-inline-in-inline-svgwrap-zw-004*.html。
    InlineReftestDef {
        id: "css-box/atomic-inline-in-inline-svgwrap-zw-004",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div style=\"width:300px\"><span><input style=\"width:200px;height:40px\"><svg width=\"200\" height=\"40\" style=\"vertical-align:top\"><rect width=\"200\" height=\"40\" fill=\"#c30\"/></svg></span></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}input{display:inline-block;margin:0;padding:0;border:0;background:#03c;vertical-align:top}</style></head><body><div style=\"width:300px\"><input style=\"width:200px;height:40px\"><svg width=\"200\" height=\"40\" style=\"vertical-align:top\"><rect width=\"200\" height=\"40\" fill=\"#c30\"/></svg></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

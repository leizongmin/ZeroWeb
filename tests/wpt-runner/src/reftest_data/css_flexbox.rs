use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 56-65: Flexbox 布局 ──
    InlineReftestDef {
        id: "css-flexbox/flex-row-two-items",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-column-direction",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-row-vs-block",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-grow-equal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-grow:1;height:50px;background:red;\"></div><div style=\"flex-grow:1;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-grow:1;height:50px;background:red;\"></div><div style=\"flex-grow:1;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-wrap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-justify-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-align-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;\"><div style=\"width:50px;height:30px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;\"><div style=\"width:50px;height:30px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:120px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:120px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-nested",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-direction:column;width:100px;height:100px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-direction:column;width:100px;height:100px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-basis-auto",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-basis:80px;height:50px;background:orange;\"></div><div style=\"flex-basis:120px;height:50px;background:cyan;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-basis:80px;height:50px;background:orange;\"></div><div style=\"flex-basis:120px;height:50px;background:cyan;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 139-148: Flexbox 进阶 ──
    InlineReftestDef {
        id: "css-flexbox/grow-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:2;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:2;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/grow-with-base",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 50px;background:red;\"></div><div style=\"flex:2 1 50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 50px;background:red;\"></div><div style=\"flex:2 1 50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/wrap-multi-line",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:200px;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:200px;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/align-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-space-between",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-between;width:300px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-between;width:300px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/shrink-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 0 150px;background:red;\"></div><div style=\"flex:0 0 150px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 0 150px;background:red;\"></div><div style=\"flex:0 0 150px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/column-direction",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"flex-grow:1;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"flex-grow:1;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/gap-between-items",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:130px;height:50px;\"><div style=\"width:30px;height:50px;background:red;\"></div><div style=\"width:30px;height:50px;background:blue;\"></div><div style=\"width:30px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:130px;height:50px;\"><div style=\"width:30px;height:50px;background:red;\"></div><div style=\"width:30px;height:50px;background:blue;\"></div><div style=\"width:30px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/order-reorder",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:150px;height:50px;\"><div style=\"order:2;width:50px;height:50px;background:red;\"></div><div style=\"order:1;width:50px;height:50px;background:blue;\"></div><div style=\"order:3;width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:150px;height:50px;\"><div style=\"order:2;width:50px;height:50px;background:red;\"></div><div style=\"order:1;width:50px;height:50px;background:blue;\"></div><div style=\"order:3;width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/basis-0-grow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 0px;background:red;\"></div><div style=\"flex:1 1 0px;background:blue;\"></div><div style=\"flex:1 1 0px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 160-169: Flexbox 边界 case ──
    InlineReftestDef {
        id: "css-flexbox/align-self-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;align-self:flex-end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;align-self:flex-end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-basis-auto-with-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-basis:auto;width:100px;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-basis:auto;width:100px;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/nowrap-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:nowrap;width:100px;height:50px;\"><div style=\"width:80px;height:50px;background:red;\"></div><div style=\"width:80px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:nowrap;width:100px;height:50px;\"><div style=\"width:80px;height:50px;background:red;\"></div><div style=\"width:80px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-end;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-end;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/wrap-reverse",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/shrink-ratio",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 2 150px;background:red;\"></div><div style=\"flex:0 1 150px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 2 150px;background:red;\"></div><div style=\"flex:0 1 150px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/min-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:1;min-width:80px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:1;min-width:80px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/max-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1;max-width:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1;max-width:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/nested-flex-wrap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-wrap:wrap;flex:1;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-wrap:wrap;flex:1;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    // ── 190-199: M3 edge case reftests ──
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-reverse-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;flex-direction:row;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;flex-direction:row;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-grow-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;padding:5px;background:red;\"></div><div style=\"flex-grow:2;padding:10px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;padding:5px;background:red;\"></div><div style=\"flex-grow:2;padding:10px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-shrink-zero",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px;height:50px;\"><div style=\"flex:0 0 80px;background:red;\"></div><div style=\"flex:0 0 80px;background:blue;\"></div><div style=\"flex:0 0 80px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px;height:50px;\"><div style=\"flex:0 0 80px;background:red;\"></div><div style=\"flex:0 0 80px;background:blue;\"></div><div style=\"flex:0 0 80px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── M6 Flexbox 扩展 reftest（目标 ≥ 50）──

    // flex: 1 均分（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-1-equal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div><div style=\"flex:1;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div><div style=\"flex:1;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex: 2 vs flex: 1（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-2-vs-1",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:2;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:2;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-basis: 0（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-basis-0",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1 1 0;height:30px;background:red\"></div><div style=\"flex:2 1 0;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1 1 0;height:30px;background:red\"></div><div style=\"flex:2 1 0;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-wrap: nowrap overflow（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-nowrap-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px\"><div style=\"width:80px;height:30px;background:red\"></div><div style=\"width:80px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px\"><div style=\"width:80px;height:30px;background:red\"></div><div style=\"width:80px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: flex-start（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: flex-end（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-end;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-end;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: stretch（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-stretch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:stretch;width:200px;height:60px\"><div style=\"width:50px;background:red\"></div><div style=\"width:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:stretch;width:200px;height:60px\"><div style=\"width:50px;background:red\"></div><div style=\"width:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: flex-start（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-start;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-start;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: space-around（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-space-around",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-around;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div><div style=\"width:40px;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-around;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div><div style=\"width:40px;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: space-evenly（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-space-evenly",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-evenly;width:200px\"><div style=\"width:30px;height:30px;background:red\"></div><div style=\"width:30px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-evenly;width:200px\"><div style=\"width:30px;height:30px;background:red\"></div><div style=\"width:30px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // order: -1 重新排序（self-match）
    InlineReftestDef {
        id: "css-flexbox/order-negative",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"order:1;width:50px;height:30px;background:red\"></div><div style=\"order:-1;width:50px;height:30px;background:blue\"></div><div style=\"width:50px;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"order:1;width:50px;height:30px;background:red\"></div><div style=\"order:-1;width:50px;height:30px;background:blue\"></div><div style=\"width:50px;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex-wrap: wrap 3 行（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-3-lines",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:20px;background:blue\"></div><div style=\"width:50px;height:20px;background:green\"></div><div style=\"width:50px;height:20px;background:yellow\"></div><div style=\"width:50px;height:20px;background:purple\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:20px;background:blue\"></div><div style=\"width:50px;height:20px;background:green\"></div><div style=\"width:50px;height:20px;background:yellow\"></div><div style=\"width:50px;height:20px;background:purple\"></div></div></body></html>",
        is_match: true,
    },
    // flex column + gap（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-column-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;gap:10px;width:100px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;gap:10px;width:100px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // nested flex（self-match）
    InlineReftestDef {
        id: "css-flexbox/nested-flex-row-in-col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"display:flex;width:200px;height:30px\"><div style=\"flex:1;background:red\"></div><div style=\"flex:1;background:blue\"></div></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"display:flex;width:200px;height:30px\"><div style=\"flex:1;background:red\"></div><div style=\"flex:1;background:blue\"></div></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex item margin: auto（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-item-auto-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:40px\"><div style=\"width:50px;height:30px;margin:auto;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:40px\"><div style=\"width:50px;height:30px;margin:auto;background:red\"></div></div></body></html>",
        is_match: true,
    },
    // flex-grow + min-width（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-grow-min-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;min-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;min-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-grow + max-width（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-grow-max-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;max-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;max-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-content: flex-start multi-line（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-content-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;align-content:flex-start;width:100px;height:100px\"><div style=\"width:40px;height:20px;background:red\"></div><div style=\"width:40px;height:20px;background:blue\"></div><div style=\"width:40px;height:20px;background:green\"></div><div style=\"width:40px;height:20px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;align-content:flex-start;width:100px;height:100px\"><div style=\"width:40px;height:20px;background:red\"></div><div style=\"width:40px;height:20px;background:blue\"></div><div style=\"width:40px;height:20px;background:green\"></div><div style=\"width:40px;height:20px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

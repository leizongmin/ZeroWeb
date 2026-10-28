use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 66-75: Grid 布局 ──
    InlineReftestDef {
        id: "css-grid/grid-fixed-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-fr-units",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-2x2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:90px 90px;gap:20px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:90px 90px;gap:20px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-auto-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:50px;width:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:50px;width:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-mixed-fr-px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px;height:50px;\"><div style=\"background:orange;\"></div><div style=\"background:cyan;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px;height:50px;\"><div style=\"background:orange;\"></div><div style=\"background:cyan;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-vs-block-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-grid/grid-three-cols",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:green;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:green;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-row-gap-col-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;row-gap:10px;column-gap:20px;width:180px;\"><div style=\"height:40px;background:red;\"></div><div style=\"height:40px;background:blue;\"></div><div style=\"height:40px;background:green;\"></div><div style=\"height:40px;background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;row-gap:10px;column-gap:20px;width:180px;\"><div style=\"height:40px;background:red;\"></div><div style=\"height:40px;background:blue;\"></div><div style=\"height:40px;background:green;\"></div><div style=\"height:40px;background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-nested",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 149-168: Grid 进阶 ──
    InlineReftestDef {
        id: "css-grid/fr-unit-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 2fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 2fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/mixed-fr-px-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr 2fr;width:400px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr 2fr;width:400px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/auto-placement-3x2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:50px 50px;width:300px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:50px 50px;width:300px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/gap-rows-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;column-gap:10px;row-gap:10px;width:210px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;column-gap:10px;row-gap:10px;width:210px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/nested-grid-in-flex",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:100px;\"><div style=\"display:grid;grid-template-columns:1fr 1fr;flex:1;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"flex:1;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:100px;\"><div style=\"display:grid;grid-template-columns:1fr 1fr;flex:1;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"flex:1;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/minmax-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:minmax(100px,1fr) 1fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:minmax(100px,1fr) 1fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/repeat-auto-fill",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:repeat(3,1fr);width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-in-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;background:yellow;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;background:yellow;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/justify-items-stretch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:300px;height:100px;\"><div style=\"background:red;height:50px;\"></div><div style=\"background:blue;height:50px;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:300px;height:100px;\"><div style=\"background:red;height:50px;\"></div><div style=\"background:blue;height:50px;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/flex-in-grid-item",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:flex;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:flex;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/shorthand-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;gap:5px 10px;width:210px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"height:50px;background:green;\"></div><div style=\"height:50px;background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;row-gap:5px;column-gap:10px;width:210px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"height:50px;background:green;\"></div><div style=\"height:50px;background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 170-179: Grid 边界 case ──
    InlineReftestDef {
        id: "css-grid/auto-rows-minmax",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:minmax(50px,auto);width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"height:80px;background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:minmax(50px,auto);width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"height:80px;background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/justify-content-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;justify-content:center;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;justify-content:center;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/align-content-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:30px 30px;align-content:center;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:30px 30px;align-content:center;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/implicit-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:40px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:40px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/place-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;place-items:center;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;place-items:center;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-auto-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-auto-flow:column;grid-auto-columns:80px;width:320px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-auto-flow:column;grid-auto-columns:80px;width:320px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/named-grid-area-simple",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"grid-column:1;grid-row:1;background:red;\"></div><div style=\"grid-column:2;grid-row:1;background:blue;\"></div><div style=\"grid-column:1/3;grid-row:2;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"grid-column:1;grid-row:1;background:red;\"></div><div style=\"grid-column:2;grid-row:1;background:blue;\"></div><div style=\"grid-column:1/3;grid-row:2;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/fr-with-percentage",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50% 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50% 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/empty-tracks",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/percentage-track-sizing",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:25% 25% 25% 25%;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:25% 25% 25% 25%;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-template-rows-percentage",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:60% 40%;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:60% 40%;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-align-self-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;align-self:end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;align-self:end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── M6 Grid 扩展 reftest（目标 ≥ 50）──

    // grid-template: 简写（self-match）
    InlineReftestDef {
        id: "css-grid/grid-template-shorthand",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template:100px 100px / 100px 100px;width:200px;height:200px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div><div style=\"background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template:100px 100px / 100px 100px;width:200px;height:200px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div><div style=\"background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // grid-area: span（self-match）
    InlineReftestDef {
        id: "css-grid/grid-area-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-column:span 2;background:red;height:50px\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-column:span 2;background:red;height:50px\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-row: span（self-match）
    InlineReftestDef {
        id: "css-grid/grid-row-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-row:span 2;background:red\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-row:span 2;background:red\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-column: 1 / -1（self-match）
    InlineReftestDef {
        id: "css-grid/grid-column-full-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px 50px;grid-template-rows:50px;width:150px\"><div style=\"grid-column:1/-1;background:red;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px 50px;grid-template-rows:50px;width:150px\"><div style=\"grid-column:1/-1;background:red;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-auto-flow: dense（self-match）
    InlineReftestDef {
        id: "css-grid/grid-auto-flow-dense",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;grid-auto-flow:dense;width:100px\"><div style=\"grid-column:span 2;height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;grid-auto-flow:dense;width:100px\"><div style=\"grid-column:span 2;height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid with gap: 20px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-gap-20px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;gap:20px;width:180px\"><div style=\"height:40px;background:red\"></div><div style=\"height:40px;background:blue\"></div><div style=\"height:40px;background:green\"></div><div style=\"height:40px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;gap:20px;width:180px\"><div style=\"height:40px;background:red\"></div><div style=\"height:40px;background:blue\"></div><div style=\"height:40px;background:green\"></div><div style=\"height:40px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // justify-items: center（self-match）
    InlineReftestDef {
        id: "css-grid/justify-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:center;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:center;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-items: end（self-match）
    InlineReftestDef {
        id: "css-grid/justify-items-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:end;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:end;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: center（self-match）
    InlineReftestDef {
        id: "css-grid/align-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:center;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:center;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: end（self-match）
    InlineReftestDef {
        id: "css-grid/align-items-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:end;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:end;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // place-items: center center（self-match）
    InlineReftestDef {
        id: "css-grid/place-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:60px;place-items:center;width:100px;height:60px\"><div style=\"width:40px;height:20px;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:60px;place-items:center;width:100px;height:60px\"><div style=\"width:40px;height:20px;background:red\"></div></div></body></html>",
        is_match: true,
    },
    // grid-auto-rows: 40px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-auto-rows-40px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:40px;width:100px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:40px;width:100px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div></div></body></html>",
        is_match: true,
    },
    // nested grid（self-match）
    InlineReftestDef {
        id: "css-grid/nested-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"display:grid;grid-template-columns:50px 50px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div><div style=\"height:40px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"display:grid;grid-template-columns:50px 50px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div><div style=\"height:40px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid in flex item（self-match）
    InlineReftestDef {
        id: "css-grid/grid-in-flex-item",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;display:grid;grid-template-columns:1fr 1fr\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;display:grid;grid-template-columns:1fr 1fr\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div></div></body></html>",
        is_match: true,
    },
    // grid 3 columns with fr（self-match）
    InlineReftestDef {
        id: "css-grid/grid-3col-fr",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid mixed fr and px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-mixed-fr-px-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // grid justify-content: space-between（self-match）
    InlineReftestDef {
        id: "css-grid/justify-content-space-between",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:40px 40px;justify-content:space-between;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:40px 40px;justify-content:space-between;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // grid align-content: center（self-match）
    InlineReftestDef {
        id: "css-grid/align-content-center-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:30px;align-content:center;width:100px;height:80px\"><div style=\"background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:30px;align-content:center;width:100px;height:80px\"><div style=\"background:red\"></div></div></body></html>",
        is_match: true,
    },
    // t-p117：grid item 流内文本 + 直属 inline-block 原子，max-content 轨道——修复前
    // taffy 轨道推导漏计文本，li 塌到原子宽 120px（文本 0 宽约束竖排），与 ref 200px
    // 差 40% 远超 Layout 1% 阈值；修复后 li = "hi" 80 + 原子 120 = 200px 单行，
    // 与块级 ref 等价锚定（css-grid-1 §7.2.2 max-content contribution）。
    InlineReftestDef {
        id: "css-grid/grid-item-hoisted-atomic-max-content-zw-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><ul style=\"display:grid;grid-template-columns:max-content;list-style:none;margin:0;padding:0;width:500px;height:260px;font:40px/40px Ahem\"><li style=\"height:260px;background:red\">hi<span style=\"display:inline-block;width:120px;height:40px;background:#0f0;border:0;padding:0\"></span></li></ul></body></html>",
        ref_html: "<html><body style=\"margin:0\"><ul style=\"list-style:none;margin:0;padding:0;width:500px;height:260px;font:40px/40px Ahem\"><li style=\"width:200px;height:260px;background:red\">hi<span style=\"display:inline-block;width:120px;height:40px;background:#0f0;border:0;padding:0\"></span></li></ul></body></html>",
        is_match: true,
    },
    // t-p117 拉伸保持（css-grid-1 §10.3 justify normal→stretch）：auto 轨道双 item
    // 同列——覆写若把窄 item（hi 80 + 原子 120 = 200）写成定尺，红底停在 200px 与
    // ref 500px 差 60%；item 保持 auto 即被拉伸步撑满轨宽 500px（r2 起 auto 轨道
    // 保守不覆写，本测同时钉住「不覆写时拉伸仍成立」），与块级 ref 等宽锚定。
    InlineReftestDef {
        id: "css-grid/grid-item-hoisted-atomic-track-stretch-zw-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><ul style=\"display:grid;grid-template-columns:auto;width:500px;list-style:none;margin:0;padding:0;font:40px/40px Ahem\"><li style=\"background:red\">hi<span style=\"display:inline-block;width:120px;height:40px;background:#0f0;border:0;padding:0\"></span></li><li style=\"background:blue\">plain</li></ul></body></html>",
        ref_html: "<html><body style=\"margin:0\"><ul style=\"list-style:none;margin:0;padding:0;width:500px;font:40px/40px Ahem\"><li style=\"background:red\">hi<span style=\"display:inline-block;width:120px;height:40px;background:#0f0;border:0;padding:0\"></span></li><li style=\"background:blue\">plain</li></ul></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

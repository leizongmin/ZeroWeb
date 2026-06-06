use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── Table layout reftests (M4) ──────────────────────────────
    // 基本 2 列表格（self-match）
    InlineReftestDef {
        id: "css-table/basic-2col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 基本 3 列表格（self-match）
    InlineReftestDef {
        id: "css-table/basic-3col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:30px\"><div style=\"display:inline-block;width:100px;height:30px;background:red\"></div><div style=\"display:inline-block;width:100px;height:30px;background:green\"></div><div style=\"display:inline-block;width:100px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 多行表格（self-match，验证多行不崩溃且渲染一致）
    InlineReftestDef {
        id: "css-table/multi-row",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 带 tbody 的表格（self-match）
    InlineReftestDef {
        id: "css-table/with-tbody",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tbody><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></tbody></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 表格自动宽度（self-match）
    InlineReftestDef {
        id: "css-table/auto-width-equal-cols",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"height:30px;background:red\"></td><td style=\"height:30px;background:green\"></td><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"height:30px;background:red\"></td><td style=\"height:30px;background:green\"></td><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 表格单元格不同高度（行高取最大值，self-match）
    InlineReftestDef {
        id: "css-table/row-tallest-cell",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // thead/tbody/tfoot 结构（self-match）
    InlineReftestDef {
        id: "css-table/thead-tbody-tfoot",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:red\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:green\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:blue\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr></tfoot></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:red\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:green\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:blue\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr></tfoot></table></body></html>",
        is_match: true,
    },
    // th 和 td 混合使用（self-match）
    InlineReftestDef {
        id: "css-table/th-td-mixed",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:red\"></th><th style=\"width:100px;height:30px;background:red\"></th></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:red\"></th><th style=\"width:100px;height:30px;background:red\"></th></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 单列表格（self-match）
    InlineReftestDef {
        id: "css-table/single-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:100px\"><tr><td style=\"height:30px;background:red\"></td></tr><tr><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px\"><div style=\"width:100px;height:30px;background:red\"></div><div style=\"width:100px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // ── 1. Basic tables: various column counts and widths ────────

    // 4 列表格（self-match）
    InlineReftestDef {
        id: "css-table/basic-4col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:400px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:blue\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:400px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:blue\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 1 列窄表格（self-match）
    InlineReftestDef {
        id: "css-table/one-col-narrow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:50px\"><tr><td style=\"height:20px;background:orange\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:50px\"><tr><td style=\"height:20px;background:orange\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 2 列不等宽表格（self-match）
    InlineReftestDef {
        id: "css-table/two-col-unequal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:200px;height:30px;background:teal\"></td><td style=\"width:100px;height:30px;background:purple\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:200px;height:30px;background:teal\"></td><td style=\"width:100px;height:30px;background:purple\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 3 列不等宽表格（self-match）
    InlineReftestDef {
        id: "css-table/three-col-unequal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:400px\"><tr><td style=\"width:150px;height:25px;background:navy\"></td><td style=\"width:100px;height:25px;background:maroon\"></td><td style=\"width:150px;height:25px;background:olive\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:400px\"><tr><td style=\"width:150px;height:25px;background:navy\"></td><td style=\"width:100px;height:25px;background:maroon\"></td><td style=\"width:150px;height:25px;background:olive\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 宽表格（self-match）
    InlineReftestDef {
        id: "css-table/wide-table",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:600px\"><tr><td style=\"width:200px;height:30px;background:silver\"></td><td style=\"width:200px;height:30px;background:gray\"></td><td style=\"width:200px;height:30px;background:dimgray\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:600px\"><tr><td style=\"width:200px;height:30px;background:silver\"></td><td style=\"width:200px;height:30px;background:gray\"></td><td style=\"width:200px;height:30px;background:dimgray\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 2. Multi-row tables: same/different heights ──────────────

    // 3 行相同高度表格（self-match）
    InlineReftestDef {
        id: "css-table/three-rows-same-height",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:20px;background:orange\"></td><td style=\"width:100px;height:20px;background:purple\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:20px;background:orange\"></td><td style=\"width:100px;height:20px;background:purple\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 4 行表格（self-match）
    InlineReftestDef {
        id: "css-table/four-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:15px;background:red\"></td><td style=\"width:100px;height:15px;background:blue\"></td></tr><tr><td style=\"width:100px;height:15px;background:green\"></td><td style=\"width:100px;height:15px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:15px;background:orange\"></td><td style=\"width:100px;height:15px;background:purple\"></td></tr><tr><td style=\"width:100px;height:15px;background:cyan\"></td><td style=\"width:100px;height:15px;background:magenta\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:15px;background:red\"></td><td style=\"width:100px;height:15px;background:blue\"></td></tr><tr><td style=\"width:100px;height:15px;background:green\"></td><td style=\"width:100px;height:15px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:15px;background:orange\"></td><td style=\"width:100px;height:15px;background:purple\"></td></tr><tr><td style=\"width:100px;height:15px;background:cyan\"></td><td style=\"width:100px;height:15px;background:magenta\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 5 行表格（self-match）
    InlineReftestDef {
        id: "css-table/five-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:12px;background:red\"></td><td style=\"width:100px;height:12px;background:blue\"></td></tr><tr><td style=\"width:100px;height:12px;background:green\"></td><td style=\"width:100px;height:12px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:12px;background:orange\"></td><td style=\"width:100px;height:12px;background:purple\"></td></tr><tr><td style=\"width:100px;height:12px;background:cyan\"></td><td style=\"width:100px;height:12px;background:magenta\"></td></tr><tr><td style=\"width:100px;height:12px;background:lime\"></td><td style=\"width:100px;height:12px;background:pink\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:12px;background:red\"></td><td style=\"width:100px;height:12px;background:blue\"></td></tr><tr><td style=\"width:100px;height:12px;background:green\"></td><td style=\"width:100px;height:12px;background:yellow\"></td></tr><tr><td style=\"width:100px;height:12px;background:orange\"></td><td style=\"width:100px;height:12px;background:purple\"></td></tr><tr><td style=\"width:100px;height:12px;background:cyan\"></td><td style=\"width:100px;height:12px;background:magenta\"></td></tr><tr><td style=\"width:100px;height:12px;background:lime\"></td><td style=\"width:100px;height:12px;background:pink\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 多行不同高度（self-match）
    InlineReftestDef {
        id: "css-table/multi-row-diff-height",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr><tr><td style=\"width:100px;height:40px;background:green\"></td><td style=\"width:100px;height:40px;background:yellow\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr><tr><td style=\"width:100px;height:40px;background:green\"></td><td style=\"width:100px;height:40px;background:yellow\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 3. Table with thead/tbody/tfoot structure ────────────────

    // thead + tbody 结构（self-match）
    InlineReftestDef {
        id: "css-table/thead-tbody",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:darkred\"></td><td style=\"width:100px;height:20px;background:darkred\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:30px;background:lightgreen\"></td><td style=\"width:100px;height:30px;background:lightgreen\"></td></tr></tbody></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:darkred\"></td><td style=\"width:100px;height:20px;background:darkred\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:30px;background:lightgreen\"></td><td style=\"width:100px;height:30px;background:lightgreen\"></td></tr></tbody></table></body></html>",
        is_match: true,
    },
    // tbody + tfoot 结构（self-match）
    InlineReftestDef {
        id: "css-table/tbody-tfoot",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tbody><tr><td style=\"width:100px;height:25px;background:lightblue\"></td><td style=\"width:100px;height:25px;background:lightblue\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:darkblue\"></td><td style=\"width:100px;height:20px;background:darkblue\"></td></tr></tfoot></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tbody><tr><td style=\"width:100px;height:25px;background:lightblue\"></td><td style=\"width:100px;height:25px;background:lightblue\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:darkblue\"></td><td style=\"width:100px;height:20px;background:darkblue\"></td></tr></tfoot></table></body></html>",
        is_match: true,
    },
    // 多行 thead/tbody/tfoot（self-match）
    InlineReftestDef {
        id: "css-table/thead-tbody-tfoot-multi-row",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:15px;background:darkred\"></td><td style=\"width:100px;height:15px;background:darkred\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:15px;background:lightgreen\"></td><td style=\"width:100px;height:15px;background:lightgreen\"></td></tr><tr><td style=\"width:100px;height:15px;background:lightyellow\"></td><td style=\"width:100px;height:15px;background:lightyellow\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:15px;background:darkblue\"></td><td style=\"width:100px;height:15px;background:darkblue\"></td></tr></tfoot></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:15px;background:darkred\"></td><td style=\"width:100px;height:15px;background:darkred\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:15px;background:lightgreen\"></td><td style=\"width:100px;height:15px;background:lightgreen\"></td></tr><tr><td style=\"width:100px;height:15px;background:lightyellow\"></td><td style=\"width:100px;height:15px;background:lightyellow\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:15px;background:darkblue\"></td><td style=\"width:100px;height:15px;background:darkblue\"></td></tr></tfoot></table></body></html>",
        is_match: true,
    },
    // ── 4. Table with th headers ─────────────────────────────────

    // th 表头行（self-match）
    InlineReftestDef {
        id: "css-table/th-header-row",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><th style=\"width:100px;height:25px;background:darkgray\"></th><th style=\"width:100px;height:25px;background:darkgray\"></th><th style=\"width:100px;height:25px;background:darkgray\"></th></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><th style=\"width:100px;height:25px;background:darkgray\"></th><th style=\"width:100px;height:25px;background:darkgray\"></th><th style=\"width:100px;height:25px;background:darkgray\"></th></tr></table></body></html>",
        is_match: true,
    },
    // th 表头 + td 数据行（self-match）
    InlineReftestDef {
        id: "css-table/th-header-with-data",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><th style=\"width:100px;height:20px;background:darkgray\"></th><th style=\"width:100px;height:20px;background:darkgray\"></th><th style=\"width:100px;height:20px;background:darkgray\"></th></tr><tr><td style=\"width:100px;height:20px;background:lightgray\"></td><td style=\"width:100px;height:20px;background:lightgray\"></td><td style=\"width:100px;height:20px;background:lightgray\"></td></tr><tr><td style=\"width:100px;height:20px;background:white\"></td><td style=\"width:100px;height:20px;background:white\"></td><td style=\"width:100px;height:20px;background:white\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><th style=\"width:100px;height:20px;background:darkgray\"></th><th style=\"width:100px;height:20px;background:darkgray\"></th><th style=\"width:100px;height:20px;background:darkgray\"></th></tr><tr><td style=\"width:100px;height:20px;background:lightgray\"></td><td style=\"width:100px;height:20px;background:lightgray\"></td><td style=\"width:100px;height:20px;background:lightgray\"></td></tr><tr><td style=\"width:100px;height:20px;background:white\"></td><td style=\"width:100px;height:20px;background:white\"></td><td style=\"width:100px;height:20px;background:white\"></td></tr></table></body></html>",
        is_match: true,
    },
    // th 在 thead 中（self-match）
    InlineReftestDef {
        id: "css-table/th-in-thead",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><th style=\"width:100px;height:25px;background:darkslategray\"></th><th style=\"width:100px;height:25px;background:darkslategray\"></th></tr></thead><tbody><tr><td style=\"width:100px;height:25px;background:lightcyan\"></td><td style=\"width:100px;height:25px;background:lightcyan\"></td></tr></tbody></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><th style=\"width:100px;height:25px;background:darkslategray\"></th><th style=\"width:100px;height:25px;background:darkslategray\"></th></tr></thead><tbody><tr><td style=\"width:100px;height:25px;background:lightcyan\"></td><td style=\"width:100px;height:25px;background:lightcyan\"></td></tr></tbody></table></body></html>",
        is_match: true,
    },
    // 全 th 表格（self-match）
    InlineReftestDef {
        id: "css-table/all-th-cells",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:goldenrod\"></th><th style=\"width:100px;height:30px;background:goldenrod\"></th></tr><tr><th style=\"width:100px;height:30px;background:gold\"></th><th style=\"width:100px;height:30px;background:gold\"></th></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:goldenrod\"></th><th style=\"width:100px;height:30px;background:goldenrod\"></th></tr><tr><th style=\"width:100px;height:30px;background:gold\"></th><th style=\"width:100px;height:30px;background:gold\"></th></tr></table></body></html>",
        is_match: true,
    },
    // ── 5. Table width: fixed, percentage, auto ──────────────────

    // 固定宽度表格（self-match）
    InlineReftestDef {
        id: "css-table/fixed-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:250px\"><tr><td style=\"height:30px;background:coral\"></td><td style=\"height:30px;background:coral\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:250px\"><tr><td style=\"height:30px;background:coral\"></td><td style=\"height:30px;background:coral\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 百分比宽度表格（self-match）
    InlineReftestDef {
        id: "css-table/percent-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:50%\"><tr><td style=\"height:30px;background:salmon\"></td><td style=\"height:30px;background:salmon\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:50%\"><tr><td style=\"height:30px;background:salmon\"></td><td style=\"height:30px;background:salmon\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 100% 宽度表格（self-match）
    InlineReftestDef {
        id: "css-table/full-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:100%\"><tr><td style=\"height:30px;background:tomato\"></td><td style=\"height:30px;background:tomato\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:100%\"><tr><td style=\"height:30px;background:tomato\"></td><td style=\"height:30px;background:tomato\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 无宽度设定的表格（auto，self-match）
    InlineReftestDef {
        id: "css-table/auto-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table><tr><td style=\"width:60px;height:25px;background:khaki\"></td><td style=\"width:80px;height:25px;background:khaki\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table><tr><td style=\"width:60px;height:25px;background:khaki\"></td><td style=\"width:80px;height:25px;background:khaki\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 表格 table-layout fixed（self-match）
    InlineReftestDef {
        id: "css-table/table-layout-fixed",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px;table-layout:fixed\"><tr><td style=\"height:30px;background:peru\"></td><td style=\"height:30px;background:peru\"></td><td style=\"height:30px;background:peru\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px;table-layout:fixed\"><tr><td style=\"height:30px;background:peru\"></td><td style=\"height:30px;background:peru\"></td><td style=\"height:30px;background:peru\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 6. Cell sizing: different widths/heights in same row ──────

    // 同行不同宽度单元格（self-match）
    InlineReftestDef {
        id: "css-table/cell-diff-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:200px;height:30px;background:sienna\"></td><td style=\"width:100px;height:30px;background:tan\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:200px;height:30px;background:sienna\"></td><td style=\"width:100px;height:30px;background:tan\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 同行不同高度单元格（self-match）
    InlineReftestDef {
        id: "css-table/cell-diff-height",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:plum\"></td><td style=\"width:100px;height:60px;background:orchid\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:plum\"></td><td style=\"width:100px;height:60px;background:orchid\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 3 列同行不同尺寸（self-match）
    InlineReftestDef {
        id: "css-table/cell-varied-sizes",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:50px;height:20px;background:violet\"></td><td style=\"width:150px;height:40px;background:mediumpurple\"></td><td style=\"width:100px;height:30px;background:mediumorchid\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:50px;height:20px;background:violet\"></td><td style=\"width:150px;height:40px;background:mediumpurple\"></td><td style=\"width:100px;height:30px;background:mediumorchid\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 7. Table in a container (nested in div) ──────────────────

    // 表格嵌套在 div 中（self-match）
    InlineReftestDef {
        id: "css-table/in-div-container",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px\"><table style=\"width:100%\"><tr><td style=\"height:30px;background:steelblue\"></td><td style=\"height:30px;background:lightsteelblue\"></td></tr></table></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px\"><table style=\"width:100%\"><tr><td style=\"height:30px;background:steelblue\"></td><td style=\"height:30px;background:lightsteelblue\"></td></tr></table></div></body></html>",
        is_match: true,
    },
    // 表格嵌套在有 padding 的 div 中（self-match）
    InlineReftestDef {
        id: "css-table/in-padded-div",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;padding:10px;background:whitesmoke\"><table style=\"width:100%\"><tr><td style=\"height:25px;background:royalblue\"></td><td style=\"height:25px;background:cornflowerblue\"></td></tr></table></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;padding:10px;background:whitesmoke\"><table style=\"width:100%\"><tr><td style=\"height:25px;background:royalblue\"></td><td style=\"height:25px;background:cornflowerblue\"></td></tr></table></div></body></html>",
        is_match: true,
    },
    // 表格嵌套在有边框的 div 中（self-match）
    InlineReftestDef {
        id: "css-table/in-bordered-div",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:220px;border:2px solid black;padding:5px\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:crimson\"></td><td style=\"width:100px;height:30px;background:firebrick\"></td></tr></table></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:220px;border:2px solid black;padding:5px\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:crimson\"></td><td style=\"width:100px;height:30px;background:firebrick\"></td></tr></table></div></body></html>",
        is_match: true,
    },
    // ── 8. Border on table/cells ─────────────────────────────────

    // td 有边框的表格（self-match）
    InlineReftestDef {
        id: "css-table/td-border",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:lightsalmon;border:1px solid red\"></td><td style=\"width:100px;height:30px;background:lightcoral;border:1px solid red\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:lightsalmon;border:1px solid red\"></td><td style=\"width:100px;height:30px;background:lightcoral;border:1px solid red\"></td></tr></table></body></html>",
        is_match: true,
    },
    // table 有边框（self-match）
    InlineReftestDef {
        id: "css-table/table-border",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px;border:2px solid black\"><tr><td style=\"width:100px;height:30px;background:wheat\"></td><td style=\"width:100px;height:30px;background:burlywood\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px;border:2px solid black\"><tr><td style=\"width:100px;height:30px;background:wheat\"></td><td style=\"width:100px;height:30px;background:burlywood\"></td></tr></table></body></html>",
        is_match: true,
    },
    // table 和 td 都有边框（self-match）
    InlineReftestDef {
        id: "css-table/table-and-td-border",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px;border:1px solid gray\"><tr><td style=\"width:100px;height:30px;background:thistle;border:1px solid purple\"></td><td style=\"width:100px;height:30px;background:lavender;border:1px solid purple\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px;border:1px solid gray\"><tr><td style=\"width:100px;height:30px;background:thistle;border:1px solid purple\"></td><td style=\"width:100px;height:30px;background:lavender;border:1px solid purple\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 9. Background colors on rows/cells ───────────────────────

    // 行背景色（self-match）
    InlineReftestDef {
        id: "css-table/row-bg-color",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr style=\"background:lightblue\"><td style=\"width:100px;height:25px\"></td><td style=\"width:100px;height:25px\"></td></tr><tr style=\"background:lightyellow\"><td style=\"width:100px;height:25px\"></td><td style=\"width:100px;height:25px\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr style=\"background:lightblue\"><td style=\"width:100px;height:25px\"></td><td style=\"width:100px;height:25px\"></td></tr><tr style=\"background:lightyellow\"><td style=\"width:100px;height:25px\"></td><td style=\"width:100px;height:25px\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 单元格不同背景色（self-match）
    InlineReftestDef {
        id: "css-table/cell-bg-colors",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:100px;height:25px;background:red\"></td><td style=\"width:100px;height:25px;background:green\"></td><td style=\"width:100px;height:25px;background:blue\"></td></tr><tr><td style=\"width:100px;height:25px;background:cyan\"></td><td style=\"width:100px;height:25px;background:magenta\"></td><td style=\"width:100px;height:25px;background:yellow\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:100px;height:25px;background:red\"></td><td style=\"width:100px;height:25px;background:green\"></td><td style=\"width:100px;height:25px;background:blue\"></td></tr><tr><td style=\"width:100px;height:25px;background:cyan\"></td><td style=\"width:100px;height:25px;background:magenta\"></td><td style=\"width:100px;height:25px;background:yellow\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 表格背景色（self-match）
    InlineReftestDef {
        id: "css-table/table-bg-color",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px;background:ivory\"><tr><td style=\"width:100px;height:30px;background:seashell\"></td><td style=\"width:100px;height:30px;background:linen\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px;background:ivory\"><tr><td style=\"width:100px;height:30px;background:seashell\"></td><td style=\"width:100px;height:30px;background:linen\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 单元格有 padding（self-match）
    InlineReftestDef {
        id: "css-table/cell-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:sandybrown;padding:5px\"></td><td style=\"width:100px;height:30px;background:chocolate;padding:5px\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:sandybrown;padding:5px\"></td><td style=\"width:100px;height:30px;background:chocolate;padding:5px\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 简单 colspan=2（self-match）
    InlineReftestDef {
        id: "css-table/colspan-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td colspan=\"2\" style=\"height:20px;background:indianred\"></td></tr><tr><td style=\"width:100px;height:20px;background:lightpink\"></td><td style=\"width:100px;height:20px;background:lightpink\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td colspan=\"2\" style=\"height:20px;background:indianred\"></td></tr><tr><td style=\"width:100px;height:20px;background:lightpink\"></td><td style=\"width:100px;height:20px;background:lightpink\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 表格 div 等价对比（match，table vs div 等价布局）
    InlineReftestDef {
        id: "css-table/table-vs-div-equiv",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:dodgerblue\"></td><td style=\"width:100px;height:40px;background:deepskyblue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:dodgerblue\"></div><div style=\"display:inline-block;width:100px;height:40px;background:deepskyblue\"></div></div></body></html>",
        is_match: true,
    },
    // ── 10. Mismatch cases: different visuals ────────────────────

    // 1 列 vs 2 列（mismatch）
    InlineReftestDef {
        id: "css-table/col-count-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:200px;height:40px;background:red\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        is_match: false,
    },
    // 不同颜色（mismatch）
    InlineReftestDef {
        id: "css-table/color-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:red\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:blue\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        is_match: false,
    },
    // 不同行数（mismatch）
    InlineReftestDef {
        id: "css-table/row-count-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        is_match: false,
    },
    // 不同宽度（mismatch）
    InlineReftestDef {
        id: "css-table/width-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"height:30px;background:teal\"></td><td style=\"height:30px;background:teal\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:400px\"><tr><td style=\"height:30px;background:teal\"></td><td style=\"height:30px;background:teal\"></td></tr></table></body></html>",
        is_match: false,
    },
    // 不同高度（mismatch）
    InlineReftestDef {
        id: "css-table/height-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:purple\"></td><td style=\"width:100px;height:20px;background:purple\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:50px;background:purple\"></td><td style=\"width:100px;height:50px;background:purple\"></td></tr></table></body></html>",
        is_match: false,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

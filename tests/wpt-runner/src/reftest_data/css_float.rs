use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── 180-189: Float 布局 (original 10) ──
    InlineReftestDef {
        id: "css-float/left-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/right-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/two-left-stacked",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-and-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-vs-right-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-float/nested-float",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:100px;height:100px;background:red;\"><div style=\"float:left;width:30px;height:30px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:100px;height:100px;background:red;\"><div style=\"float:left;width:30px;height:30px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-with-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-none-no-float",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:none;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-in-flex",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-in-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:50px;\"><div style=\"float:left;width:30px;height:30px;background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:50px;\"><div style=\"float:left;width:30px;height:30px;background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Multiple left floats stacking horizontally (5 cases) ──
    InlineReftestDef {
        id: "css-float/three-left-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:left;width:60px;height:60px;background:blue;\"></div><div style=\"float:left;width:60px;height:60px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:left;width:60px;height:60px;background:blue;\"></div><div style=\"float:left;width:60px;height:60px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/four-left-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:left;width:80px;height:50px;background:blue;\"></div><div style=\"float:left;width:80px;height:50px;background:green;\"></div><div style=\"float:left;width:80px;height:50px;background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:left;width:80px;height:50px;background:blue;\"></div><div style=\"float:left;width:80px;height:50px;background:green;\"></div><div style=\"float:left;width:80px;height:50px;background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/five-left-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:500px;height:100px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div><div style=\"float:left;width:80px;height:40px;background:green;\"></div><div style=\"float:left;width:80px;height:40px;background:yellow;\"></div><div style=\"float:left;width:80px;height:40px;background:purple;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:500px;height:100px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div><div style=\"float:left;width:80px;height:40px;background:green;\"></div><div style=\"float:left;width:80px;height:40px;background:yellow;\"></div><div style=\"float:left;width:80px;height:40px;background:purple;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-floats-exact-fit",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:100px;height:50px;background:red;\"></div><div style=\"float:left;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:100px;height:50px;background:red;\"></div><div style=\"float:left;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Multiple right floats (4 cases) ──
    InlineReftestDef {
        id: "css-float/two-right-stacked",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/three-right-horizontal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:right;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div><div style=\"float:right;width:80px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:right;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div><div style=\"float:right;width:80px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/right-float-single-wide",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:200px;height:80px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:200px;height:80px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Mixed left and right floats (4 cases) ──
    InlineReftestDef {
        id: "css-float/mixed-left-right-two",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:80px;height:60px;background:red;\"></div><div style=\"float:right;width:80px;height:60px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:80px;height:60px;background:red;\"></div><div style=\"float:right;width:80px;height:60px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/mixed-left-right-three",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:60px;height:50px;background:red;\"></div><div style=\"float:left;width:60px;height:50px;background:green;\"></div><div style=\"float:right;width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:60px;height:50px;background:red;\"></div><div style=\"float:left;width:60px;height:50px;background:green;\"></div><div style=\"float:right;width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/mixed-both-sides-symmetrical",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"float:left;width:50px;height:50px;background:green;\"></div><div style=\"float:right;width:50px;height:50px;background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"float:left;width:50px;height:50px;background:green;\"></div><div style=\"float:right;width:50px;height:50px;background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/mixed-left-right-with-block",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:right;width:60px;height:60px;background:blue;\"></div><div style=\"width:60px;height:30px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:right;width:60px;height:60px;background:blue;\"></div><div style=\"width:60px;height:30px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Float with clear (5 cases) ──
    InlineReftestDef {
        id: "css-float/clear-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/clear-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:60px;height:30px;background:red;\"></div><div style=\"float:right;clear:right;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:60px;height:30px;background:red;\"></div><div style=\"float:right;clear:right;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/clear-both-left-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:right;width:60px;height:30px;background:green;\"></div><div style=\"float:left;clear:both;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:right;width:60px;height:30px;background:green;\"></div><div style=\"float:left;clear:both;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/clear-left-after-two-lefts",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:40px;background:red;\"></div><div style=\"float:left;width:60px;height:40px;background:green;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:40px;background:red;\"></div><div style=\"float:left;width:60px;height:40px;background:green;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/clear-both-after-stacked",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:30px;background:green;\"></div><div style=\"float:left;clear:both;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:30px;background:green;\"></div><div style=\"float:left;clear:both;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Float with different sizes (4 cases) ──
    InlineReftestDef {
        id: "css-float/left-small-large",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:40px;height:40px;background:red;\"></div><div style=\"float:left;width:100px;height:80px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:40px;height:40px;background:red;\"></div><div style=\"float:left;width:100px;height:80px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-large-small",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:100px;height:80px;background:red;\"></div><div style=\"float:left;width:40px;height:40px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:100px;height:80px;background:red;\"></div><div style=\"float:left;width:40px;height:40px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/right-different-sizes",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:50px;height:30px;background:red;\"></div><div style=\"float:right;width:80px;height:60px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:50px;height:30px;background:red;\"></div><div style=\"float:right;width:80px;height:60px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/mixed-sizes-left-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:200px;\"><div style=\"float:left;width:60px;height:90px;background:red;\"></div><div style=\"float:right;width:100px;height:50px;background:blue;\"></div><div style=\"float:left;width:40px;height:40px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:200px;\"><div style=\"float:left;width:60px;height:90px;background:red;\"></div><div style=\"float:right;width:100px;height:50px;background:blue;\"></div><div style=\"float:left;width:40px;height:40px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Float inside container with overflow:hidden (BFC) (4 cases) ──
    InlineReftestDef {
        id: "css-float/bfc-overflow-hidden-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:left;width:80px;height:80px;background:red;\"></div><div style=\"width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:left;width:80px;height:80px;background:red;\"></div><div style=\"width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/bfc-overflow-hidden-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:right;width:80px;height:80px;background:red;\"></div><div style=\"width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:right;width:80px;height:80px;background:red;\"></div><div style=\"width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/bfc-overflow-hidden-both",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;overflow:hidden;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div><div style=\"width:60px;height:20px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;overflow:hidden;\"><div style=\"float:left;width:80px;height:50px;background:red;\"></div><div style=\"float:right;width:80px;height:50px;background:blue;\"></div><div style=\"width:60px;height:20px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/bfc-overflow-hidden-stacked",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:left;width:60px;height:60px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;overflow:hidden;\"><div style=\"float:left;width:60px;height:60px;background:red;\"></div><div style=\"float:left;width:60px;height:60px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Float with margin and padding interactions (4 cases) ──
    InlineReftestDef {
        id: "css-float/left-margin-20",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin-left:20px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin-left:20px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/right-margin-20",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:50px;height:50px;margin-right:20px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:50px;height:50px;margin-right:20px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;padding:10px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;padding:10px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-margin-padding-combined",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;padding:5px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;padding:5px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Float wrapping behavior when not enough space (4 cases) ──
    InlineReftestDef {
        id: "css-float/wrap-to-next-line",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:200px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:200px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/wrap-right-under-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:200px;\"><div style=\"float:left;width:100px;height:60px;background:red;\"></div><div style=\"float:right;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:200px;\"><div style=\"float:left;width:100px;height:60px;background:red;\"></div><div style=\"float:right;width:80px;height:40px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/wrap-three-in-narrow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:300px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div><div style=\"float:left;width:80px;height:40px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:300px;\"><div style=\"float:left;width:80px;height:40px;background:red;\"></div><div style=\"float:left;width:80px;height:40px;background:blue;\"></div><div style=\"float:left;width:80px;height:40px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── Nested floats (float inside float) (4 cases) ──
    InlineReftestDef {
        id: "css-float/nested-left-in-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:150px;height:100px;background:red;\"><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"float:left;width:50px;height:50px;background:green;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:150px;height:100px;background:red;\"><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"float:left;width:50px;height:50px;background:green;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/nested-right-in-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:150px;height:100px;background:red;\"><div style=\"float:right;width:50px;height:50px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:right;width:150px;height:100px;background:red;\"><div style=\"float:right;width:50px;height:50px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/nested-mixed-in-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:200px;height:100px;background:red;\"><div style=\"float:left;width:40px;height:40px;background:blue;\"></div><div style=\"float:right;width:40px;height:40px;background:green;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:200px;height:100px;background:red;\"><div style=\"float:left;width:40px;height:40px;background:blue;\"></div><div style=\"float:right;width:40px;height:40px;background:green;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/nested-two-levels",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:300px;\"><div style=\"float:left;width:200px;height:150px;background:red;\"><div style=\"float:left;width:100px;height:80px;background:blue;\"><div style=\"float:left;width:40px;height:40px;background:green;\"></div></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:400px;height:300px;\"><div style=\"float:left;width:200px;height:150px;background:red;\"><div style=\"float:left;width:100px;height:80px;background:blue;\"><div style=\"float:left;width:40px;height:40px;background:green;\"></div></div></div></div></body></html>",
        is_match: true,
    },
    // ── Mismatch cases: float vs no-float, left vs right (5 cases) ──
    InlineReftestDef {
        id: "css-float/float-left-with-block-beside",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-vs-right-two-floats-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:60px;height:50px;background:red;\"></div><div style=\"float:left;width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:right;width:60px;height:50px;background:red;\"></div><div style=\"float:right;width:60px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-float/float-none-block-behavior",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:none;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:none;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/clear-left-stacking",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:200px;\"><div style=\"float:left;width:60px;height:30px;background:red;\"></div><div style=\"float:left;clear:left;width:60px;height:30px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-with-margin-20px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:20px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:20px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── ::after clearfix + 仅含浮动的 UL（baidu 热榜叠字回归，R1323/R1392）──
    // test 页为 baidu 同款机制形状：wrapper（定宽居中 + padding-top）> 标题 +
    // UL（高 0，float li 全溢出，odd 行 clear:both）+ ::after clear:both 收尾。
    // 机制失效时（float 子的 clear 误置 clearance_active → R1319 把 ::after 的
    // 合法 clearance 拉回 UL 底）wrapper 塌缩（135→84），后续 lime 块叠压银色
    // 行（信号 ~8%，远超 Layout 1% 容差）。ref 页同视觉但容器显式内容高 114px
    //（content-box，border-box=135）绕过机制。无文本，排除字体噪声。
    InlineReftestDef {
        id: "css-float/clearfix-after-nested-float-clear",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}.w{position:relative;margin:93px auto 0;padding-top:21px;width:760px}.w::after{content:\"\";display:block;clear:both}.t{height:24px;margin-bottom:18px;background:yellow}.ul{list-style:none;padding:0;margin:0}.li{float:left;width:369px;height:36px;background:silver}.li.odd{clear:both;margin-right:20px}.after{height:40px;background:lime}</style></head><body><div class=\"w\"><div class=\"t\"></div><ul class=\"ul\"><li class=\"li odd\"></li><li class=\"li\"></li><li class=\"li odd\"></li><li class=\"li\"></li></ul></div><div class=\"after\"></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}.w{position:relative;margin:93px auto 0;padding-top:21px;width:760px;height:114px}.t{height:24px;margin-bottom:18px;background:yellow}.ul{list-style:none;padding:0;margin:0}.li{float:left;width:369px;height:36px;background:silver}.li.odd{clear:both;margin-right:20px}.after{height:40px;background:lime}</style></head><body><div class=\"w\"><div class=\"t\"></div><ul class=\"ul\"><li class=\"li odd\"></li><li class=\"li\"></li><li class=\"li odd\"></li><li class=\"li\"></li></ul></div><div class=\"after\"></div></body></html>",
        is_match: true,
    },
    // ── R1392 余项：中间非 BFC 容器自身 border/padding 分量（嵌套浮动底边虚减）──
    // test 页为机制形状：outer > wrapper（border-top:5 + padding-top:16，非 BFC）
    // > float(369×36)，outer > cleared(clear:both)。按 CSS2 §9.5.2 clear 须让位嵌套
    // 浮动底边 21+36=57；余项在时 cleared 落 36，lime（36..116）与银色浮动带
    //（21..57）叠压 21px（信号 ~6.6%，远超 Layout 1% 容差）。ref 页把 wrapper 改
    // overflow:hidden（BFC 容纳浮动，wrapper 高 57）且 cleared 去掉 clear——同视觉
    // 绕过机制。无文本，排除字体噪声。
    InlineReftestDef {
        id: "css-float/nested-float-clear-middle-frame",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}.outer{width:760px}.w{border-top:5px solid #cc0000;padding-top:16px}.f{float:left;width:369px;height:36px;background:silver}.after{clear:both;height:80px;background:lime}</style></head><body><div class=\"outer\"><div class=\"w\"><div class=\"f\"></div></div><div class=\"after\"></div></div></body></html>",
        ref_html: "<html><head><style>body{margin:0}.outer{width:760px}.w{overflow:hidden;border-top:5px solid #cc0000;padding-top:16px}.f{float:left;width:369px;height:36px;background:silver}.after{height:80px;background:lime}</style></head><body><div class=\"outer\"><div class=\"w\"><div class=\"f\"></div></div><div class=\"after\"></div></div></body></html>",
        is_match: true,
    },
    // ── slice10：float 子 + inline 兄弟共存容器不吸收 float 子树文本（百度热榜
    // 残余叠字回归，paint Path B）──
    // test 页为触发形状：li(float) ×3 行 > [a(float, overflow:hidden,
    // [i(inline-block), span.t(inline CJK 标题文本)]), mark(inline-block)]。
    // 机制失效时 paint Path B 空 styles 无法识别 float 子 → collector 把 a 子树
    // 的标题文本吸收进 li 的 IFC，重排到 float 下方第二行（strut 模型推算：
    // 首行基线 ≈57，缺陷位 ≈147 = 57+90，近似口径），且 painted_inline_nodes
    // 去重抑制 a 盒自身正确绘制（CSS2 §9.5 float 脱离常规流）。ref 页去掉
    // mark 兄弟——活体单因子实验（repro4 r3a vs r3f）已证该兄弟是触发器：
    // 无 mark 时同一形状文本恒在正确基线。
    // 判别信号（TE-1 返修）：吸收缺陷只错位 Path B glyph run（非文本盒经盒树
    // 正常绘制、两页同位零 diff，实测结构色块方案假绿 0.80%/0.20%）——故信号
    // 用 CJK 系统回退字（Noto Sans CJK，依赖面与既有 css-text/cjk-line-break
    // 相同）：fs40 × 9 字/行 × 3 行，密集中文墨块在 ref（行 1）与 test（错位
    // 行 2）两处互斥出现，diff 由墨迹面积主导且与 Ahem 无关（撤 Ahem 实测
    // 同值红）；3 行累计裕度 ≥3× 于 Layout 1% 容差。双变异负控制实测见
    // diag/evidence/slice10/reftest-negative-control-{a,b,c,r}-rework.log。
    InlineReftestDef {
        id: "css-float/float-child-inline-sibling-no-text-absorption",
        category: ReftestCategory::Layout,
        test_html: "<html><head><style>body{margin:0}.ul{list-style:none;margin:0;padding:0;width:760px}.li{float:left;clear:both;width:400px;height:90px;line-height:90px;font-size:12px;white-space:nowrap}.a{float:left;display:block;width:400px;height:90px;line-height:90px;font-size:14px;overflow:hidden;white-space:nowrap}.i{display:inline-block;width:30px;height:30px;line-height:30px;font-size:30px}.t{display:inline;line-height:90px;font-size:40px;color:black}.mk{display:inline-block;width:10px;height:40px;margin-left:4px}</style></head><body><ul class=\"ul\"><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a><span class=\"mk\"></span></li><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a><span class=\"mk\"></span></li></ul></body></html>",
        ref_html: "<html><head><style>body{margin:0}.ul{list-style:none;margin:0;padding:0;width:760px}.li{float:left;clear:both;width:400px;height:90px;line-height:90px;font-size:12px;white-space:nowrap}.a{float:left;display:block;width:400px;height:90px;line-height:90px;font-size:14px;overflow:hidden;white-space:nowrap}.i{display:inline-block;width:30px;height:30px;line-height:30px;font-size:30px}.t{display:inline;line-height:90px;font-size:40px;color:black}.mk{display:inline-block;width:10px;height:40px;margin-left:4px}</style></head><body><ul class=\"ul\"><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a></li><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a></li><li class=\"li\"><a class=\"a\"><i class=\"i\"></i><span class=\"t\">热榜行文本吸收缺陷</span></a></li></ul></body></html>",
        is_match: true,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

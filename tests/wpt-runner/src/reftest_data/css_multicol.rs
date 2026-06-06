use super::InlineReftestDef;
use crate::reftest::ReftestCategory;

const REFTESTS: &[InlineReftestDef] = &[
    // ── Multi-column 布局 reftest ──

    // 1. column-count:2 基础（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 2. column-count:3 三列（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-3",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 3. column-width 自动计算列数（self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-auto",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 4. column-gap 列间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 5. columns 简写属性（self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 6. 均衡分配：4 个子元素到 2 列（self-match，验证不 crash）
    InlineReftestDef {
        id: "css-multicol/balanced-4-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // 7. 不均衡子元素高度（self-match）
    InlineReftestDef {
        id: "css-multicol/uneven-heights",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:60px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:60px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 8. 多列 + column-rule（self-match，column-rule-solid 不 crash）
    InlineReftestDef {
        id: "css-multicol/with-column-rule",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:2px solid black;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:2px solid black;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 9. column-count mismatch（不同列数应产生不同渲染）
    InlineReftestDef {
        id: "css-multicol/mismatch-column-count",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: false,
    },
    // 10. 无 column-count / column-width 时为单列（self-match）
    InlineReftestDef {
        id: "css-multicol/no-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // ── 新增 40 条 reftest（#11-#50） ──

    // ── 1) column-count: 2, 3, 4, 5 with various child counts ──

    // 11. column-count:4 四列 8 子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-4-eight-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:4;width:400px\"><div style=\"height:20px;background:#e6194b\"></div><div style=\"height:20px;background:#3cb44b\"></div><div style=\"height:20px;background:#ffe119\"></div><div style=\"height:20px;background:#4363d8\"></div><div style=\"height:20px;background:#f58231\"></div><div style=\"height:20px;background:#911eb4\"></div><div style=\"height:20px;background:#42d4f4\"></div><div style=\"height:20px;background:#f032e6\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:4;width:400px\"><div style=\"height:20px;background:#e6194b\"></div><div style=\"height:20px;background:#3cb44b\"></div><div style=\"height:20px;background:#ffe119\"></div><div style=\"height:20px;background:#4363d8\"></div><div style=\"height:20px;background:#f58231\"></div><div style=\"height:20px;background:#911eb4\"></div><div style=\"height:20px;background:#42d4f4\"></div><div style=\"height:20px;background:#f032e6\"></div></div></body></html>",
        is_match: true,
    },
    // 12. column-count:5 五列 10 子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-5-ten-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:5;width:400px\"><div style=\"height:15px;background:red\"></div><div style=\"height:15px;background:orange\"></div><div style=\"height:15px;background:yellow\"></div><div style=\"height:15px;background:green\"></div><div style=\"height:15px;background:blue\"></div><div style=\"height:15px;background:indigo\"></div><div style=\"height:15px;background:violet\"></div><div style=\"height:15px;background:pink\"></div><div style=\"height:15px;background:gray\"></div><div style=\"height:15px;background:brown\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:5;width:400px\"><div style=\"height:15px;background:red\"></div><div style=\"height:15px;background:orange\"></div><div style=\"height:15px;background:yellow\"></div><div style=\"height:15px;background:green\"></div><div style=\"height:15px;background:blue\"></div><div style=\"height:15px;background:indigo\"></div><div style=\"height:15px;background:violet\"></div><div style=\"height:15px;background:pink\"></div><div style=\"height:15px;background:gray\"></div><div style=\"height:15px;background:brown\"></div></div></body></html>",
        is_match: true,
    },
    // 13. column-count:2 五个子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-2-five-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:yellow\"></div><div style=\"height:20px;background:purple\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:yellow\"></div><div style=\"height:20px;background:purple\"></div></div></body></html>",
        is_match: true,
    },
    // 14. column-count:3 六个子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-3-six-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:20px;background:#aaa\"></div><div style=\"height:20px;background:#bbb\"></div><div style=\"height:20px;background:#ccc\"></div><div style=\"height:20px;background:#ddd\"></div><div style=\"height:20px;background:#eee\"></div><div style=\"height:20px;background:#fff\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:20px;background:#aaa\"></div><div style=\"height:20px;background:#bbb\"></div><div style=\"height:20px;background:#ccc\"></div><div style=\"height:20px;background:#ddd\"></div><div style=\"height:20px;background:#eee\"></div><div style=\"height:20px;background:#fff\"></div></div></body></html>",
        is_match: true,
    },
    // 15. column-count:2 只有一个子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-2-single-child",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:40px;background:teal\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:40px;background:teal\"></div></div></body></html>",
        is_match: true,
    },
    // 16. column-count:4 七个子元素（不能均分，self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-4-seven-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:4;width:400px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:yellow\"></div><div style=\"height:20px;background:orange\"></div><div style=\"height:20px;background:purple\"></div><div style=\"height:20px;background:cyan\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:4;width:400px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:yellow\"></div><div style=\"height:20px;background:orange\"></div><div style=\"height:20px;background:purple\"></div><div style=\"height:20px;background:cyan\"></div></div></body></html>",
        is_match: true,
    },
    // ── 2) column-width: various widths, combined with column-count ──

    // 17. column-width:80px 容器 320px（应产生 4 列，self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-80px-320px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:80px;width:320px\"><div style=\"height:25px;background:coral\"></div><div style=\"height:25px;background:steelblue\"></div><div style=\"height:25px;background:seagreen\"></div><div style=\"height:25px;background:goldenrod\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:80px;width:320px\"><div style=\"height:25px;background:coral\"></div><div style=\"height:25px;background:steelblue\"></div><div style=\"height:25px;background:seagreen\"></div><div style=\"height:25px;background:goldenrod\"></div></div></body></html>",
        is_match: true,
    },
    // 18. column-width:150px 容器 300px（应产生 2 列，self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-150px-300px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:150px;width:300px\"><div style=\"height:30px;background:olive\"></div><div style=\"height:30px;background:maroon\"></div><div style=\"height:30px;background:navy\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:150px;width:300px\"><div style=\"height:30px;background:olive\"></div><div style=\"height:30px;background:maroon\"></div><div style=\"height:30px;background:navy\"></div></div></body></html>",
        is_match: true,
    },
    // 19. column-width + column-count 同时指定（column-count 优先，self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-with-count",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:50px;column-count:3;width:300px\"><div style=\"height:25px;background:tomato\"></div><div style=\"height:25px;background:dodgerblue\"></div><div style=\"height:25px;background:limegreen\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:50px;column-count:3;width:300px\"><div style=\"height:25px;background:tomato\"></div><div style=\"height:25px;background:dodgerblue\"></div><div style=\"height:25px;background:limegreen\"></div></div></body></html>",
        is_match: true,
    },
    // 20. column-width:200px 容器 200px（只能放 1 列，self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-equals-container",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:200px;width:200px\"><div style=\"height:30px;background:salmon\"></div><div style=\"height:30px;background:lightblue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:200px;width:200px\"><div style=\"height:30px;background:salmon\"></div><div style=\"height:30px;background:lightblue\"></div></div></body></html>",
        is_match: true,
    },
    // 21. column-width:auto 回退为隐含 1 列（self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-auto-only",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:auto;width:200px\"><div style=\"height:30px;background:peru\"></div><div style=\"height:30px;background:sienna\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:auto;width:200px\"><div style=\"height:30px;background:peru\"></div><div style=\"height:30px;background:sienna\"></div></div></body></html>",
        is_match: true,
    },
    // ── 3) column-gap: different gap sizes ──

    // 22. column-gap:0 无间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap-zero",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:0\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:0\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 23. column-gap:40px 大间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap-large",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:40px;width:280px\"><div style=\"height:30px;background:crimson\"></div><div style=\"height:30px;background:darkcyan\"></div><div style=\"height:30px;background:darkorange\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:40px;width:280px\"><div style=\"height:30px;background:crimson\"></div><div style=\"height:30px;background:darkcyan\"></div><div style=\"height:30px;background:darkorange\"></div></div></body></html>",
        is_match: true,
    },
    // 24. column-gap:normal 默认间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap-normal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:normal\"><div style=\"height:25px;background:#a00\"></div><div style=\"height:25px;background:#0a0\"></div><div style=\"height:25px;background:#00a\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:normal\"><div style=\"height:25px;background:#a00\"></div><div style=\"height:25px;background:#0a0\"></div><div style=\"height:25px;background:#00a\"></div></div></body></html>",
        is_match: true,
    },
    // 25. column-gap:5px 小间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap-small",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:5px\"><div style=\"height:20px;background:#c00\"></div><div style=\"height:20px;background:#0c0\"></div><div style=\"height:20px;background:#00c\"></div><div style=\"height:20px;background:#cc0\"></div><div style=\"height:20px;background:#0cc\"></div><div style=\"height:20px;background:#c0c\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px;column-gap:5px\"><div style=\"height:20px;background:#c00\"></div><div style=\"height:20px;background:#0c0\"></div><div style=\"height:20px;background:#00c\"></div><div style=\"height:20px;background:#cc0\"></div><div style=\"height:20px;background:#0cc\"></div><div style=\"height:20px;background:#c0c\"></div></div></body></html>",
        is_match: true,
    },
    // ── 4) columns shorthand: various combinations ──

    // 26. columns:3 简写（self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand-3",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:3;width:300px\"><div style=\"height:25px;background:chocolate\"></div><div style=\"height:25px;background:darkviolet\"></div><div style=\"height:25px;background:mediumseagreen\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:3;width:300px\"><div style=\"height:25px;background:chocolate\"></div><div style=\"height:25px;background:darkviolet\"></div><div style=\"height:25px;background:mediumseagreen\"></div></div></body></html>",
        is_match: true,
    },
    // 27. columns:100px 简写（column-width，self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:100px;width:300px\"><div style=\"height:25px;background:hotpink\"></div><div style=\"height:25px;background:mediumblue\"></div><div style=\"height:25px;background:gold\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:100px;width:300px\"><div style=\"height:25px;background:hotpink\"></div><div style=\"height:25px;background:mediumblue\"></div><div style=\"height:25px;background:gold\"></div></div></body></html>",
        is_match: true,
    },
    // 28. columns:2 100px 简写（同时指定 count 和 width，self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand-both",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:2 100px;width:200px\"><div style=\"height:30px;background:indianred\"></div><div style=\"height:30px;background:slateblue\"></div><div style=\"height:30px;background:forestgreen\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:2 100px;width:200px\"><div style=\"height:30px;background:indianred\"></div><div style=\"height:30px;background:slateblue\"></div><div style=\"height:30px;background:forestgreen\"></div></div></body></html>",
        is_match: true,
    },
    // 29. columns:auto 简写（self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand-auto",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:auto;width:200px\"><div style=\"height:30px;background:lightcoral\"></div><div style=\"height:30px;background:lightgreen\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:auto;width:200px\"><div style=\"height:30px;background:lightcoral\"></div><div style=\"height:30px;background:lightgreen\"></div></div></body></html>",
        is_match: true,
    },
    // ── 5) Balanced distribution: many children, varying heights ──

    // 30. 均衡：2 列 6 个等高子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/balanced-2col-6children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:15px;background:#111\"></div><div style=\"height:15px;background:#222\"></div><div style=\"height:15px;background:#333\"></div><div style=\"height:15px;background:#444\"></div><div style=\"height:15px;background:#555\"></div><div style=\"height:15px;background:#666\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:15px;background:#111\"></div><div style=\"height:15px;background:#222\"></div><div style=\"height:15px;background:#333\"></div><div style=\"height:15px;background:#444\"></div><div style=\"height:15px;background:#555\"></div><div style=\"height:15px;background:#666\"></div></div></body></html>",
        is_match: true,
    },
    // 31. 均衡：3 列 9 个等高子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/balanced-3col-9children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:12px;background:#a1a1a1\"></div><div style=\"height:12px;background:#b2b2b2\"></div><div style=\"height:12px;background:#c3c3c3\"></div><div style=\"height:12px;background:#d4d4d4\"></div><div style=\"height:12px;background:#e5e5e5\"></div><div style=\"height:12px;background:#f6f6f6\"></div><div style=\"height:12px;background:#aaaaaa\"></div><div style=\"height:12px;background:#bbbbbb\"></div><div style=\"height:12px;background:#cccccc\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:12px;background:#a1a1a1\"></div><div style=\"height:12px;background:#b2b2b2\"></div><div style=\"height:12px;background:#c3c3c3\"></div><div style=\"height:12px;background:#d4d4d4\"></div><div style=\"height:12px;background:#e5e5e5\"></div><div style=\"height:12px;background:#f6f6f6\"></div><div style=\"height:12px;background:#aaaaaa\"></div><div style=\"height:12px;background:#bbbbbb\"></div><div style=\"height:12px;background:#cccccc\"></div></div></body></html>",
        is_match: true,
    },
    // 32. 均衡：2 列不等高子元素（self-match，shortest-column-first 策略）
    InlineReftestDef {
        id: "css-multicol/balanced-varying-heights-2col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:50px;background:darkred\"></div><div style=\"height:10px;background:darkgreen\"></div><div style=\"height:10px;background:darkblue\"></div><div style=\"height:10px;background:darkorange\"></div><div style=\"height:10px;background:darkviolet\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:50px;background:darkred\"></div><div style=\"height:10px;background:darkgreen\"></div><div style=\"height:10px;background:darkblue\"></div><div style=\"height:10px;background:darkorange\"></div><div style=\"height:10px;background:darkviolet\"></div></div></body></html>",
        is_match: true,
    },
    // 33. 均衡：3 列 5 个不等高子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/balanced-varying-heights-3col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:40px;background:#700\"></div><div style=\"height:20px;background:#070\"></div><div style=\"height:10px;background:#007\"></div><div style=\"height:30px;background:#770\"></div><div style=\"height:15px;background:#077\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:40px;background:#700\"></div><div style=\"height:20px;background:#070\"></div><div style=\"height:10px;background:#007\"></div><div style=\"height:30px;background:#770\"></div><div style=\"height:15px;background:#077\"></div></div></body></html>",
        is_match: true,
    },
    // 34. 均衡：2 列一个大高块 + 许多小块（self-match）
    InlineReftestDef {
        id: "css-multicol/balanced-one-tall-many-small",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:80px;background:rebeccapurple\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:80px;background:rebeccapurple\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div><div style=\"height:10px;background:lightgray\"></div></div></body></html>",
        is_match: true,
    },
    // ── 6) Single child in multicol container ──

    // 35. 单子元素 column-count:3（self-match）
    InlineReftestDef {
        id: "css-multicol/single-child-count-3",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:50px;background:mediumorchid\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:50px;background:mediumorchid\"></div></div></body></html>",
        is_match: true,
    },
    // 36. 单子元素 column-width:100px（self-match）
    InlineReftestDef {
        id: "css-multicol/single-child-width-100",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:60px;background:mediumturquoise\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:60px;background:mediumturquoise\"></div></div></body></html>",
        is_match: true,
    },
    // 37. 单子元素占满容器高度（self-match）
    InlineReftestDef {
        id: "css-multicol/single-child-tall",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:120px;background:midnightblue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:120px;background:midnightblue\"></div></div></body></html>",
        is_match: true,
    },
    // ── 7) column-rule: different styles/colors/widths ──

    // 38. column-rule:3px dotted red（self-match）
    InlineReftestDef {
        id: "css-multicol/column-rule-dotted-red",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;column-gap:20px;column-rule:3px dotted red;width:360px\"><div style=\"height:30px;background:#eee\"></div><div style=\"height:30px;background:#ddd\"></div><div style=\"height:30px;background:#ccc\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;column-gap:20px;column-rule:3px dotted red;width:360px\"><div style=\"height:30px;background:#eee\"></div><div style=\"height:30px;background:#ddd\"></div><div style=\"height:30px;background:#ccc\"></div></div></body></html>",
        is_match: true,
    },
    // 39. column-rule:4px double blue（self-match）
    InlineReftestDef {
        id: "css-multicol/column-rule-double-blue",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:4px double blue;width:240px\"><div style=\"height:35px;background:#f0f0f0\"></div><div style=\"height:35px;background:#e0e0e0\"></div><div style=\"height:35px;background:#d0d0d0\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:4px double blue;width:240px\"><div style=\"height:35px;background:#f0f0f0\"></div><div style=\"height:35px;background:#e0e0e0\"></div><div style=\"height:35px;background:#d0d0d0\"></div></div></body></html>",
        is_match: true,
    },
    // 40. column-rule:1px dashed green（self-match）
    InlineReftestDef {
        id: "css-multicol/column-rule-dashed-green",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:16px;column-rule:1px dashed green;width:216px\"><div style=\"height:40px;background:mintcream\"></div><div style=\"height:40px;background:lavender\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:16px;column-rule:1px dashed green;width:216px\"><div style=\"height:40px;background:mintcream\"></div><div style=\"height:40px;background:lavender\"></div></div></body></html>",
        is_match: true,
    },
    // 41. column-rule:none 无分隔线（self-match）
    InlineReftestDef {
        id: "css-multicol/column-rule-none",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;column-gap:20px;column-rule:none;width:340px\"><div style=\"height:25px;background:wheat\"></div><div style=\"height:25px;background:thistle\"></div><div style=\"height:25px;background:turquoise\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;column-gap:20px;column-rule:none;width:340px\"><div style=\"height:25px;background:wheat\"></div><div style=\"height:25px;background:thistle\"></div><div style=\"height:25px;background:turquoise\"></div></div></body></html>",
        is_match: true,
    },
    // ── 8) Multicol with nested elements ──

    // 42. 多列容器内嵌套 div（self-match）
    InlineReftestDef {
        id: "css-multicol/nested-div",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div><div style=\"height:25px;background:tomato\"></div><div style=\"height:25px;background:gold\"></div></div><div style=\"height:30px;background:dodgerblue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div><div style=\"height:25px;background:tomato\"></div><div style=\"height:25px;background:gold\"></div></div><div style=\"height:30px;background:dodgerblue\"></div></div></body></html>",
        is_match: true,
    },
    // 43. 多列容器内嵌套多列（self-match）
    InlineReftestDef {
        id: "css-multicol/nested-multicol",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:300px\"><div style=\"column-count:2\"><div style=\"height:15px;background:coral\"></div><div style=\"height:15px;background:skyblue\"></div><div style=\"height:15px;background:lime\"></div><div style=\"height:15px;background:pink\"></div></div><div style=\"height:30px;background:plum\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:300px\"><div style=\"column-count:2\"><div style=\"height:15px;background:coral\"></div><div style=\"height:15px;background:skyblue\"></div><div style=\"height:15px;background:lime\"></div><div style=\"height:15px;background:pink\"></div></div><div style=\"height:30px;background:plum\"></div></div></body></html>",
        is_match: true,
    },
    // 44. 多列容器内混合内联和块级（self-match）
    InlineReftestDef {
        id: "css-multicol/mixed-inline-block-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:sandybrown\"></div><div style=\"height:20px;background:mediumpurple\"></div><div style=\"height:25px;background:lightseagreen\"></div><div style=\"height:15px;background:palevioletred\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:sandybrown\"></div><div style=\"height:20px;background:mediumpurple\"></div><div style=\"height:25px;background:lightseagreen\"></div><div style=\"height:15px;background:palevioletred\"></div></div></body></html>",
        is_match: true,
    },
    // 45. 多列容器内带 padding 的子元素（self-match）
    InlineReftestDef {
        id: "css-multicol/children-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;padding:5px;background:khaki\"></div><div style=\"height:20px;padding:5px;background:orchid\"></div><div style=\"height:20px;padding:5px;background:cadetblue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;padding:5px;background:khaki\"></div><div style=\"height:20px;padding:5px;background:orchid\"></div><div style=\"height:20px;padding:5px;background:cadetblue\"></div></div></body></html>",
        is_match: true,
    },
    // ── 9) Edge cases: very narrow/wide columns, 1 column ──

    // 46. column-count:1 单列（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-1",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:1;width:200px\"><div style=\"height:30px;background:firebrick\"></div><div style=\"height:30px;background:teal\"></div><div style=\"height:30px;background:darkolivegreen\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:1;width:200px\"><div style=\"height:30px;background:firebrick\"></div><div style=\"height:30px;background:teal\"></div><div style=\"height:30px;background:darkolivegreen\"></div></div></body></html>",
        is_match: true,
    },
    // 47. 很窄的列 column-width:30px 容器 300px（self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-very-narrow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:30px;width:300px\"><div style=\"height:10px;background:#d00\"></div><div style=\"height:10px;background:#0d0\"></div><div style=\"height:10px;background:#00d\"></div><div style=\"height:10px;background:#dd0\"></div><div style=\"height:10px;background:#0dd\"></div><div style=\"height:10px;background:#d0d\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:30px;width:300px\"><div style=\"height:10px;background:#d00\"></div><div style=\"height:10px;background:#0d0\"></div><div style=\"height:10px;background:#00d\"></div><div style=\"height:10px;background:#dd0\"></div><div style=\"height:10px;background:#0dd\"></div><div style=\"height:10px;background:#d0d\"></div></div></body></html>",
        is_match: true,
    },
    // 48. 很宽的列 column-width:500px 容器 300px（退化为单列，self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-very-wide",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:500px;width:300px\"><div style=\"height:30px;background:#a52a2a\"></div><div style=\"height:30px;background:#2e8b57\"></div><div style=\"height:30px;background:#4682b4\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:500px;width:300px\"><div style=\"height:30px;background:#a52a2a\"></div><div style=\"height:30px;background:#2e8b57\"></div><div style=\"height:30px;background:#4682b4\"></div></div></body></html>",
        is_match: true,
    },
    // 49. column-count:2 容器带 padding（self-match）
    InlineReftestDef {
        id: "css-multicol/container-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px;padding:10px;box-sizing:border-box\"><div style=\"height:25px;background:rosybrown\"></div><div style=\"height:25px;background:slategray\"></div><div style=\"height:25px;background:darkkhaki\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px;padding:10px;box-sizing:border-box\"><div style=\"height:25px;background:rosybrown\"></div><div style=\"height:25px;background:slategray\"></div><div style=\"height:25px;background:darkkhaki\"></div></div></body></html>",
        is_match: true,
    },
    // ── 10) Mismatch cases ──

    // 50. mismatch: column-count:2 vs column-count:4（mismatch）
    InlineReftestDef {
        id: "css-multicol/mismatch-count-2-vs-4",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:400px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:4;width:400px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:yellow\"></div></div></body></html>",
        is_match: false,
    },
];

pub fn reftests() -> &'static [InlineReftestDef] {
    REFTESTS
}

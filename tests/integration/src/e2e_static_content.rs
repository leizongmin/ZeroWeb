// t8i（site-compat bilibili-20261002-r1）：**script 路径 fragment 视图插入**端到端钉。
//
// 根因背景（设计卡 .acceptance/site-optimizer/bilibili-20261002-r1/designs/
// t8i-template-content-clonenode.md 第二轮定谳）：页面 `<script>` 路径下
// `template.innerHTML=` 的 content 子为 M 域解析代理（`_zwFragmentAdded` → `_zwMEl`，
// 无 `__zwHandle`、缺 cloneNode）——与 CDP evaluate 路径 fabric 分叉。Vue legacy
// `insertStaticContent`（index-legacy chunk 行级实证）新鲜路径
// `t.insertBefore(zl.content, n)` 直插 content 视图；缓存路径克隆**上次挂载捕获的
// parent.firstChild 链边界对象** `i.cloneNode(!0)`——边界即 M 域 kid 时抛
// `TypeError: i.cloneNode is not a function`（站点 18 错误簇）。
//
// 本钉以纯页面 script（无 evaluate 参与）复刻双容器形态 + 缓存重挂载全链：
// - 形态 A（站点组件内容器，createElement 产物 handle 域）：R97 分支解析子收编
//   （registry push + 重挂 parentNode + cloneNode 补齐）→ fresh 可见 + 边界可克隆；
// - 形态 B（root 容器，querySelector 产物 sel-only 域）：此前静默 no-op 死区 →
//   sel 域 overlay 并入（`_zwSelPendingParent` 槽 parentSel=sel）；
// - 缓存路径：cloneNode(true) + `i=i.nextSibling` 兄弟游走 + 逐个克隆回插
//   （Vue 第二次挂载原样复刻）→ 无 TypeError、宿主/视图子数按 3 递增。
//
// 探针必须走真实页面加载：evaluate 路径子带 handle、脚本路径为 M 域代理——
// evaluate 形态探针对本根因不敏感（fabric 分叉教训，见同目录 e2e_lit_library 头注）。

#[cfg(test)]
mod static_content_e2e {
    use zero_webview::{WebView, WebViewConfig};

    fn run_page(page_script: &str) -> String {
        let html = format!(
            r#"<html><head><title>StaticContent E2E</title></head><body>
<div id="app"></div>
<script>
{page_script}
</script>
</body></html>"#
        );
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(&html, None);
        let _ = wv.run_page_scripts_strict();
        wv.execute_script_with_dom(
            "(typeof globalThis.__t8iReport === 'string') ? globalThis.__t8iReport : 'NO-REPORT'",
        )
        .unwrap_or_else(|_| "EXEC-ERR".to_string())
    }

    /// 形态 A + 缓存重挂载：handle 容器（createElement 产物）——Vue 组件内静态内容
    /// 挂载原样复刻（fresh insertBefore(content, anchor) → 边界捕获 → 二次挂载走
    /// cloneNode 缓存路径）。修复前：边界对象 clone=undefined → TypeError（站点
    /// 18 簇同款）。
    #[test]
    fn static_content_handle_container_cached_remount() {
        let report = run_page(
            r#"
var log = [];
var box = document.createElement('div');
document.body.appendChild(box);
var anchor = document.createComment('[');
box.appendChild(anchor);
var zl = document.createElement('template');
zl.innerHTML = '<b>1</b><i>2</i><em>3</em>';
// 新鲜路径：content 视图直插（Vue `t.insertBefore(zl.content, n)`）。
box.insertBefore(zl.content, anchor);
log.push('fresh:' + box.childNodes.length);
log.push('prev:' + (anchor.previousSibling ? anchor.previousSibling.tagName : 'null'));
// 缓存路径：Vue 第二次挂载原样复刻——`for(;t.insertBefore(i.cloneNode(!0),n),
// i!==r&&(i=i.nextSibling);)`：克隆并回插的是**原始链**上 start→end 的每个节点
//（start/end 为上次挂载返回的静态内容首尾边界；克隆体插入后走原始链推进）。
var startB = box.firstChild;
var endE = anchor.previousSibling;
var n = 0;
var i2 = startB;
for (;;) {
  box.insertBefore(i2.cloneNode(true), anchor);
  n++;
  if (i2 === endE || !i2.nextSibling) break;
  i2 = i2.nextSibling;
}
log.push('cached:' + n + ':' + box.childNodes.length);
globalThis.__t8iReport = log.join('|');
"#,
        );
        assert_eq!(
            report, "fresh:4|prev:EM|cached:3:7",
            "handle 容器：fresh 4 子 + EM 边界可克隆 + 缓存路径 3 克隆（3+1+3=7）"
        );
    }

    /// 形态 B：sel-only 容器（querySelector 产物，Vue root mount 形态）——修复前
    /// R97 handle 门之外静默 no-op（子不可见）；修复后 sel 域 overlay 并入 + 解析子
    /// 收编（边界可克隆）。
    #[test]
    fn static_content_sel_container_visible() {
        let report = run_page(
            r#"
var log = [];
var app = document.querySelector('#app');
var anchor = document.createComment('<');
app.appendChild(anchor);
var zl = document.createElement('template');
zl.innerHTML = '<b>x</b><i>y</i>';
app.insertBefore(zl.content, anchor);
log.push('fresh:' + app.childNodes.length);
log.push('prev:' + (anchor.previousSibling ? anchor.previousSibling.tagName : 'null'));
log.push('clone:' + typeof (anchor.previousSibling && anchor.previousSibling.cloneNode));
globalThis.__t8iReport = log.join('|');
"#,
        );
        assert_eq!(
            report, "fresh:3|prev:I|clone:function",
            "sel-only 容器：2 子并入 + I 边界 + cloneNode 可用（修复前 1/null/undefined）"
        );
    }
}

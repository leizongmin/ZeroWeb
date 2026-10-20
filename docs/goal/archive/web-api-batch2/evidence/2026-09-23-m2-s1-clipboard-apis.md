# M2 切片 1 — ClipboardItem/Clipboard 富 MIME 面（clipboard-apis 修齐轮 1）

**日期**: 2026-09-23
**套件**: `make testharness-clipboard-apis`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-23-m2-s1-clipboard-apis.json](2026-09-23-m2-s1-clipboard-apis.json)
**基线**: [2026-09-23-m1-clipboard-apis-baseline.md](2026-09-23-m1-clipboard-apis-baseline.md)（21.1%）

## 结果

| 指标 | M1 基线 | M2-s1 | Δ |
|---|---|---|---|
| subtests Pass | 15/71 = 21.1% | **44/70 = 62.9%** | +41.8pp |
| 全绿用例 | 5/33 | **15/33** | +10 |

## 落地面（engine js_dom_shim/part02.js clipboard 块重写）

- **globalThis.ClipboardItem**：record 值存原始引用（Blob | DOMString | Promise 均收）；
  `types` getter / `presentationStyle`（'unspecified'）/ `getType`（missing → NotFoundError；
  read 项代际失配 → InvalidStateError——cached/stale-getType-reject 两案语义）
- **globalThis.Clipboard 接口**（`navigator.clipboard instanceof Clipboard` + SameObject 面）
- **write**：sequence<ClipboardItem> 转换校验（非序列/缺 length → TypeError，promise-returning
  走 rejected Promise 不同步抛）、元素 brand 校验、>1 项 NotAllowedError（上游 "not
  implemented" 同款现状）、image/* 须 Blob（DOMString → TypeError）、Promise 值 settle 后
  归一 DOMString→Blob(type)、写后 store 代际 +1
- **read**：空 store 返 []；非空返当代 ClipboardItem（绑定 `_epoch`）
- **readText**：从 store text/plain Blob 解码（R2817/R2964 往返语义保持）
- **writeText**：缺参 TypeError（required DOMString）

单测：`test_clipboard_item_rich_mime_wab2m2`（part07.rs，接口面/往返/八簇校验/代际失效/
NotFoundError）；R2964 测试断言同步新语义（read() 返单项）。

## 全绿案 15（新增 10）

cached/stale-getType-reject、custom-formats write-read ±web-prefix、html-script-removal、
write-domstring、unsanitized-plaintext、clipboard-events-synthetic、data-transfer-file-list、
permissions granted ×2、text-write-read ×4。

## 剩余 18 Fail 聚类（下一轮修齐方向）

| 簇 | 案数 | 根因 | 下一动作 |
|---|---|---|---|
| execCommand('copy') → 异步 store 桥未接 | 2（read-sanitize、read-resource-load）+ write-html-read-html 连带 | document.oncopy clipboardData.setData 内容不落 navigator store | M2 切片 2（part06 execCommand + store 桥） |
| 本地文件 fetch→Blob 无 type（`expected "image/png" but got ""`） | 3（write-blobs、promise-write-blobs、write-image） | runner fetch 本地文件无扩展名→MIME 映射 | runner fetch infra（engine fetch_bridge，本流域） |
| 权限 denied 不拒绝 | 2 | P3 权限语义（query 'denied' → readText/writeText NotAllowedError） | P3（security-hardening DC-4 对齐点） |
| copy 事件 isTrusted false | 1 | 信任链事件语义 | 记账（testdriver/信任面） |
| DataTransfer clearData 计数 | 1 | DataTransfer items.add 后 clearData 语义 | M2 顺带 |
| tentative custom formats 校验（>100 上限/web 前缀规则/unsanitized 多格式拒绝） | 4 | spec tentative 面 | 记账（spec 定稿重入） |
| `/common/subset-tests.js` fetch | 1 | runner 不服务跨 corpus `/common/` 绝对路径 | 记账（runner infra） |
| basics 1 subtest Timeout | 1 | 待甄别（fetch image 子测） | 下一轮甄别 |

# M2 切片 2 — execCommand copy 桥 + fetch MIME + 时限预算（clipboard-apis 修齐轮 2）

**日期**: 2026-09-23
**套件**: `make testharness-clipboard-apis`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-23-m2-s2-clipboard-apis.json](2026-09-23-m2-s2-clipboard-apis.json)
**前值**: [2026-09-23-m2-s1-clipboard-apis.md](2026-09-23-m2-s1-clipboard-apis.md)（62.9%）

## 结果

| 指标 | M2-s1 | M2-s2 | Δ |
|---|---|---|---|
| subtests Pass | 44/70 = 62.9% | **51/72 = 70.8%** | +7.9pp |
| 全绿用例 | 15/33 | **19/33** | +4 |

新增全绿：async-write-blobs-read-blobs、async-promise-write-blobs-read-blobs、
async-navigator-clipboard-read-sanitize、async-navigator-clipboard-read-resource-load。

## 落地面（三处）

1. **execCommand('copy'/'cut') 桥**（engine part06.js + part02.js 钩子）：copy/cut 派发携带真
   DataTransfer 作 clipboardData（此前 R2936 恒 null → handler `clipboardData.setData`
   TypeError）；defaultPrevented → handler 载荷按 spec "update the clipboard content" 经
   `__zwClipboardStoreWrite` 内部钩子（`__zw` 前缀同 `__zw_opfs` 约定）落 navigator.clipboard
   store；未 preventDefault / paste 不落 store（permissive 语义保持）。接通 read-sanitize /
   read-resource-load。单测 `test_clipboard_execcommand_copy_bridge_wab2m2s2`（preventDefault
   落 store + 未 preventDefault 不落 store 双断言）。
2. **静态资源 MIME 映射**（runner `wpt_static_resource_headers`）：png/jpg/gif/webp/svg/ico/
   css/json 扩展名 → content-type（此前仅 html/txt/js）。`fetch('resources/greenbox.png')` →
   `Response.blob()` 读 content-type 头 → Blob.type 'image/png'，接通 write-blobs /
   promise-write-blobs 的 type 断言。
3. **单案时限预算**（runner `CORPUS_CASE_TIMEOUT` 30s，两 corpus 共用）：clipboard basics 族
   单案串行 ~19 个 test_driver.click 激活周期 × 探针循环 ~0.8s/周期 → 10s 默认时限 ~9 周期处
   伪超时（b16/b6 探针实测幂等复现）。basics 9→11 真子测绿；余下停滞点见下。

## 已甄别未解（记档，不硬解）

- **basics 停滞点非纯时限**：30s 下仍卡在 ~11-12 激活周期（0.8s/周期推算 ~15s 应完成）——
  周期 ~10-12 后 testdriver click 管线停摆，触发器未定位（apply_testdriver_command 无状态
  per-command、__zw_td_queue splice 出队、stub pending/queuedElements 映射均排除；嫌疑 =
  探针循环长程状态或 Activate 派发泵长程退化）。11/19 真子测绿。记账：runner 探针循环
  长程行为，重入 = 下一轮 runner infra 专项。
- write-image「malformed image data 应拒绝」：write() 无图片数据校验（上游校验真 PNG 魔数）——
  记账（语义面，M2 后续）。
- write-html-read-html `headElement.remove is not a function`：DOMParser 产物 head 元素缺
  remove()——js-dom 面，非 clipboard 域，记账。
- read-unsanitized-null：read(options) 字典校验面（unsanitized 非序列应 TypeError）——小面，
  M2 后续。
- permissions denied ×2：P3（security-hardening DC-4 对齐）。
- tentative custom formats/unsanitized ×5 + svg Unsupported ×1 + /common fetch ×1：同 M2-s1
  记账。
- fullscreen @30s 持平 63.0%（8 个 Timeout 为真挂起非时限预算，M3 甄别对象）。

# Web API 第二批 — 运行时控制面板（master.md）

**入口文档**: [../web-api-batch2.md](../web-api-batch2.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-23（M2-s1：ClipboardItem/Clipboard 富 MIME 面，21.1%→62.9%）

---

## 当前状态

**专项定位**：M12 余面收编——Clipboard API + Fullscreen API 语义落地，WPT 两 corpus
为验收标尺。DnD 排除挂账（宿主拖拽输入管线深依赖）。

**与兄弟 goal 的边界**：
- security-hardening — 权限语义供数关系（本 goal 最小权限查询面，其 DC-4 落地时对齐）
- rendering-compat — `:fullscreen` 样式面跨域记账，`crates/style-system` 不碰
- cdp-protocol / devtools — 无协议面耦合
- android-browser / desktop-browser — 全屏窗口语义触 host-runtime 平台面时 git log 核对

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | clipboard-apis / fullscreen 两 corpus fetch 脚本 + 导入 + 基线 | ✅ M1（2026-09-23） |
| P2 | navigator.clipboard 四方法 + ClipboardItem/Clipboard 面 + ClipboardEvent + 内存后端 | 🔄 M2-s1（2026-09-23）21.1%→**62.9%**（富 MIME 面落地）；余簇 = execCommand copy 桥 ×2-3、fetch→Blob MIME ×3、denied 拒绝 ×2（P3）、杂项 |
| P3 | 最小权限查询面（security-hardening DC-4 对齐点） | ⏳（denied 拒绝 ×2 案 + query 状态联动） |
| P4 | Fullscreen 事件/状态面 + viewport 联动 | ⏳ M3（基线 63.0%，主簇 = 事件 target/栈时序） |
| P5 | 平台剪贴板后端（host-runtime 能力评估）或差异记账 | ⏳ M4 挂账定稿 |

## 已完成切片

- **M1（2026-09-23）**：`fetch-clipboard-apis-subset.sh` + `fetch-fullscreen-subset.sh`
  （pin 315976933870）+ runner 双子命令（`testharness-clipboard-apis` /
  `testharness-fullscreen`，observers 式内容级 skip）+ Makefile 四目标（test-guard 包裹）
  + 基线 evidence ×2（clipboard 21.1% / fullscreen 63.0%，账本 88 行）。
- **M1 基建修复（WAB2-M1）**：engine `script_gen.rs` strict-eval 全局发布扫描补
  `async function`/`function*`/`async function*` 形态（clipboard-apis helper 全 async
  形态 → 24 案 ReferenceError 的根因；修复前口径 13.0% → 修复后 21.1%）。单测 +
  R147/R198/R201 邻接回归全绿。
- **立项基线事实修正**：引擎已有部分 fullscreen 语义（viewport 桥遗产）——fullscreen
  基线起点 63.0% 显著高于立项预期（立项记「无 requestFullscreen 面」）。
- **M2 切片 1（2026-09-23）**：ClipboardItem/Clipboard 富 MIME 面——part02.js clipboard
  块重写（ClipboardItem 全局类 + types/presentationStyle/getType 代际失效 + Clipboard
  接口 instanceof + write 输入校验簇 + read/write 存取）。clipboard-apis 21.1%→62.9%
  （44/70，全绿案 5→15）。evidence/2026-09-23-m2-s1-clipboard-apis.{md,json}。

## 下一步计划

1. **M2-s2**：execCommand('copy') → 异步 store 桥（part06 oncopy clipboardData.setData
   内容落 navigator store；read-sanitize/read-resource-load/write-html 三案）+ 本地文件
   fetch→Blob 扩展名→MIME 映射（engine fetch_bridge；write-blobs/write-image 三案）+
   basics Timeout 子测甄别
2. **M2-s3/P3**：权限 denied 拒绝语义（query 状态联动 readText/writeText
   NotAllowedError；security-hardening DC-4 对齐）+ DataTransfer clearData 顺带
3. **M3**：fullscreenchange/error target 与栈时序簇 → promises 拒绝形态 → Timeout 8 案
   甄别 → viewport 联动验证（消费 ④ viewport 桥）
4. **M4**：DC 逐项判定 + 平台后端差异挂账定稿

**跨域记账（回流不越界）**：
- `:fullscreen` 伪类/UA 渲染样式面 → rendering-compat 流域（rendering/ 9 案不导入 +
  api 内伪类断言簇）
- iframe 依赖面（fullscreen 25 案 + clipboard detached-iframe 6 案）→ 重入 = iframe
  文档管道
- runner infra：testdriver `bless`/`set_context` 越白名单（11 案 Unsupported）、
  `/common/` 绝对路径 fetch（1 案）

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 导入与基线 | ✅ 2026-09-23 |
| M2 — Clipboard 语义 + 后端 | ⏳ |
| M3 — Fullscreen 语义 | ⏳ |
| M4 — 收口 | ⏳ |

## 验证基线

- 基线 evidence：`evidence/2026-09-23-m1-clipboard-apis-baseline.{md,json}`
  （33 案 / 15-71 = 21.1%）+ `evidence/2026-09-23-m1-fullscreen-baseline.{md,json}`
  （55 案 / 92-146 = 63.0%）；账本 `imported-testharness.txt` +88 行（WAB2-M1-baseline）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  engine 面 js_dom_bridge R147/R198/R201/WAB2-M1 单测绿；全量 `make test` 门禁见
  M1 提交说明

**碰撞管理**：碰 engine shim 面前与 security-hardening / cdp-protocol `git log` 互核。

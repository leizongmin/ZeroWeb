# Web Components goal 收口记录（2026-09-11 DONE 判定）

**判定**：DC-1~5 全部满足 → **DONE**（Final Output Protocol）。
**最终成绩**：WPT 三目录（custom-elements / shadow-dom / the-template-element）
**3619 / 4764 subtests（76.0%）**——基线 437/4730（9%）起步，累计 **+3182 Pass**。
**证据链**：`docs/goal/web-components/evidence/`（54 个文件，每步 md+json 成对）。

## 里程碑全录（全部 ✅）

| 里程碑 | 内容 | 收口 |
|---|---|---|
| M1 切片 1 | WPT 三目录导入 265 案 + 基线 9%（fetch 脚本 + runner manifest + imported-testharness.txt 账本） | 2026-09-10 |
| M1 切片 2a/2b | customElements.upgrade 真语义（is-aware）+ whenDefined 真 Promise + CustomElementRegistry 接口对象 | 2026-09-10 |
| M1 切片 3/4 | adoptedCallback 派发路径 + v8 native/quickjs 双路径一致（native fail 集与 polyfill 重合） | 2026-09-10 |
| M2 | template DOM 层真实化（parser inert fragment contents + content 属性 + R145 查询规则收敛） | 2026-09-10 |
| M3 切片 1-7 | slot 全链路（IDL→分配→slotchange→assignedNodes）+ attachShadow 校验 + slottable 过滤 + 事件路径同构化 | 2026-09-10/11 |
| M3 切片 8 第一增量 | composed path / shadow root 站 / per-station retarget / relatedTarget / eventPhase / clearTargets / disabledFeatures | 2026-09-11 |
| M3 切片 8 第二增量 1-13 小步 | signalSet spec 直译 + MO 投递分轮 + 接收者 identity + reactions 反射面 + CE markup 构造管线 + 跨文档 adopted + 数值反射 | 2026-09-11 |

关键数据点：slotchange 尾 12 案全清（第九小步 net +13）；reactions 反射面（第十步
net +45）；CE markup 构造管线（第十一步 net +59）；reactions/Element.html 47/47 全清
（第十三步）。

## DC 逐项判定（对照 docs/goal/web-components.md）

| DC | 项 | 判定 | 证据 |
|---|---|---|---|
| DC-1 | 三目录 window 可执行面导入 + skip list 注明 | ✅ | fetch-web-components-subset.sh 头注释 + runner `wc_case_skipped` 同一规则集 |
| DC-1 | fetch 脚本 | ✅ | scripts/fetch-web-components-subset.sh（WPT_REV pin 同版） |
| DC-1 | 分类通过率报告 + 基线 | ✅ | evidence/2026-09-10-wc-baseline.{md,json} + 每步成对 evidence |
| DC-1 | driving 用例入常驻断言集账本 | ✅ | imported-testharness.txt（593 行，WC-M1-baseline 批次 + 后续） |
| DC-1 | 通过率报告持久化 evidence/ | ✅ | 54 文件（2026-09-10 → 2026-09-11 全程） |
| DC-2 | upgrade 真语义 | ✅ | M1 切片 2a/2b + R149 define 自动升级 + is-aware |
| DC-2 | whenDefined 真 Promise | ✅ | define 前 pending / define 后 microtask resolve |
| DC-2 | adoptedCallback 派发 | ✅ | M1 切片 3 + 本周跨文档 adopted（m3s8m 探针实证三段全序） |
| DC-2 | v8 native / quickjs 双路径一致 | ✅ | M1 切片 4；native fail 集与 polyfill 完全重合（R136-R138 起持续） |
| DC-3 | parser 层 inert DocumentFragment contents | ✅ | M2 切片 1（DC-3 三项全满足，2026-09-10） |
| DC-3 | content 属性真实 fragment | ✅ | 同上 |
| DC-3 | template 内资源不加载/脚本不执行 | ✅ | 同上 |
| DC-4 | HTMLSlotElement 接线（name / el.slot / assignedSlot） | ✅ | M3 切片 1 |
| DC-4 | assignedNodes({flatten}) 真语义 | ✅ | M3 切片 3 |
| DC-4 | slotchange 派发（分配变化，microtask 时序） | ✅ | M3 切片 5-8 第二增量第九小步（spec notify 步骤 4-7 直译） |
| DC-4 | shadow 树分配 + light DOM fallback 基础 flattened tree 查询 | ✅ | M3 切片 3-6 |
| DC-5 | make test 全绿 | ✅ | 第十三小步轮 67 套件全绿（2026-09-11） |
| DC-5 | clippy -D warnings 零警告 | ✅ | 同轮（后续增量无 Rust 变更） |
| DC-5 | 每修复有单测 + driving WPT 资产化 | ✅ | 各切片单测 + 271 案常驻断言集每轮全量复跑 |
| DC-5 | make reftest 无回归 | ✅ | 收口轮复跑 687/687（100%）（2026-09-11） |

## 遗留深项（goal 收口后记录，非 DC 范围）

均为**其他子系统深项域**，按 goal 执行协议「遇深结构记挂账跳过」处理，不阻 DC：

1. **reactions/ 剩余 fail（~207F）**：media 播放状态联动（audio/video 播放回调时序）、
   table insertRow/deleteRow 族（table-scoped 解析升级）、Range/cloneContents 交叉面——
   各自为独立深项域，非 CE 构造/反射面。
2. **builtin-coverage（~126F）**：customized built-ins 经 innerHTML 实例化的解析面
   （R380 查询融合域交界）——与 parser 域共享根因。
3. **XHTML 悬空 body 查询域（5 案）**：pre-existing，与 template 真实化无因果（M2 已判）。
4. **customized-builtins iframe/reparse 面**：深结构。

## 渲染级排除（等用户点名专项）

- **Shadow DOM 渲染级 composed tree**：shadow 树进样式/布局/绘制、`:host`/`::slotted`
  ——与 rendering-compat 交界，按 Support Envelope 排除条款记录，不算未满足 DC。
- **Rust `resolve_slots` 接线**：JS 面 slot 全链路已完成；渲染级消费属上项专项。

## 收口时点门禁

- `make test`：67 套件全绿（2026-09-11，第十三小步 HEAD ec6b8f165）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零警告。
- `make reftest`：687/687（100%）。
- `cargo build`：make test compile-first 覆盖。

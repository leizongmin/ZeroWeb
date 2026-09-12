# Web API 第二批 — Clipboard API + Fullscreen API（M12 面）

**版本**: v1.0
**日期**: 2026-09-12
**状态**: Active
**执行模式**: WPT 驱动（上游 clipboard-apis / fullscreen corpus 为验收标尺）+ 语义修齐；
照 event-loop-spec / web-components goal 通用打法
**父目标**: `docs/goal/zero-web.md`（M12 高级 Web 能力「更多 DOM API」余面）

> **说明**
> 本文档是 ZeroWeb「Web API 第二批」专项目标执行契约。父目标 M12 明示 Clipboard API /
> Fullscreen API / Drag & Drop 未立项——本目标收编前两者；**DnD 因依赖宿主拖拽输入
> 管线（深），排除挂账**（重入条件见下）。
>
> **▶ 拆分动机（2026-09-12 用户决策）**：① M12 余面收编，照 event-loop-spec /
> web-components 成熟打法（WPT 导入 → 基线 → 语义修齐）；② Clipboard 是 Web 应用
> 高频依赖（编辑器/复制分享），Fullscreen 是视频/演示场景高频依赖；③ 与渲染流域
> 零 crate 重叠（JS/事件语义面），可独立并行。
>
> **▶ 基线事实（2026-09-12 实测）**：
> - **Clipboard**：无 navigator.clipboard 面；系统剪贴板访问缺（headless 需内存后端，
>     平台剪贴板后端照 host-runtime 能力评估）；WPT `clipboard-apis/` corpus 存在。
> - **Fullscreen**：无 Element.requestFullscreen/exitFullscreen 面、无 fullscreenchange
>     事件、无 `:fullscreen` 伪类（样式面属 rendering-compat 流域，跨域记账）；WPT
>     `fullscreen/` corpus 存在。
> - **权限依赖**：Clipboard read/write 规范要求 permissions 语义——security-hardening
>     goal DC-4 供完整语义层；本目标先落 feature-detect + 本地后端 + 最小权限查询面
>     （边界声明见 Support Envelope）。
> - **验收标尺**：上游 `clipboard-apis/` + `fullscreen/` window 可执行面子集导入
>     （fetch 脚本照 observers/fs 先例）；本地等价用例如实标注。

---

## Mission

以 **WPT 真实用例为验收标准**，落地 Clipboard API（read/readText/write/writeText +
clipboardchange 事件评估）与 Fullscreen API（requestFullscreen/exitFullscreen +
fullscreenchange/error 事件）语义，使编辑器/媒体场景高频 API 不再缺位。

**关键约束**：
1. **WPT 标尺先行**：导入基线 → 逐簇修齐（纯资产切片零源码改动先行）
2. **headless 后端如实标注**：剪贴板内存后端为 headless/CI 语义；平台剪贴板后端
   以 host-runtime 能力为准，差异记账
3. **DnD 不在本目标**：依赖宿主拖拽输入管线（深），挂账；重入条件 = 输入管线
   成熟或 cdp-protocol Input.dispatchDragEvent 面就位后用户点名

覆盖范围：

1. **Clipboard API** — navigator.clipboard（read/readText/write/writeText）+
   ClipboardEvent + permissions 查询最小面 + 内存/平台后端
2. **Fullscreen API** — Element.requestFullscreen/exitFullscreen（含 options）+
   document.fullscreenElement/fullscreenEnabled + fullscreenchange/fullscreenerror
   事件 + 全屏状态下的 viewport 语义（消费 ④ viewport 桥遗产）

### 排除（明确不在范围内）

- Drag & Drop —— 宿主拖拽输入管线深依赖，挂账（重入条件见 Mission 约束 3）
- `:fullscreen` 伪类与全屏渲染样式面 —— rendering-compat 流域，跨域记账
- Web Share / Notification / Credential Management —— 未立项域

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | navigator.clipboard / Element fullscreen 语义面 | part 系 JS 语义 |
| webview/host-runtime | 剪贴板后端（内存 + 平台能力评估）、全屏窗口语义 | host 侧支撑 |
| WPT 资产 | clipboard-apis / fullscreen 子集导入 | fetch 脚本 + 账本 |
| 测试 | 单测 + 集成 + 上游 corpus 通道 | 照 observers 先例 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 security-hardening（同日新立）**：权限语义供数关系——本目标最小权限查询面
  不与其 DC-4 冲突（后者完整语义层落地时对齐/替换）；碰 engine shim 面 git log 互核。
- **与 rendering-compat**：`:fullscreen` 样式面不在本流；`crates/style-system` 不碰。
- **与 cdp-protocol**：全屏/剪贴板无 CDP 面耦合（headless 验收走其自动化通道仅作
  消费）。
- **与 android-browser / desktop-browser**：无共享面（全屏窗口语义若触 host-runtime
  平台面，git log 核对）。

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 导入与基线

- [ ] 上游 `clipboard-apis/` + `fullscreen/` window 可执行面子集导入（fetch 脚本 +
      常驻通道 + imported 账本）
- [ ] 分类通过率基线（文本 + JSON）落 evidence/

### DC-2: Clipboard 语义收敛

- [ ] navigator.clipboard 四方法语义（read/readText/write/writeText）+ ClipboardEvent；
      WPT 驱动逐簇修齐，通过率有可追踪提升
- [ ] 剪贴板后端：headless 内存后端可用；平台剪贴板后端落地或差异如实记账

### DC-3: Fullscreen 语义收敛

- [ ] requestFullscreen/exitFullscreen（含 options）/fullscreenElement/fullscreenEnabled
      + fullscreenchange/error 事件；WPT 驱动逐簇修齐
- [ ] 全屏状态 viewport 语义联动验证（消费 ④ viewport 桥）

### DC-4: 测试与质量不可退让

- [ ] `make test` 全绿零失败 + clippy `-D warnings` + fmt 干净
- [ ] 每语义切片带单测/集成测试；`make reftest` 零回归（API 面不触无关节点渲染）

---

## 活跃里程碑

### M1 — WPT 导入与基线

**目标**：两 corpus fetch 脚本 + 导入 + 通过率基线（纯资产切片零源码改动）。

### M2 — Clipboard 语义 + 后端

**目标**：navigator.clipboard 面 + 内存后端 + 最小权限查询；WPT 修齐。

### M3 — Fullscreen 语义

**目标**：fullscreen 事件/状态面 + viewport 联动；WPT 修齐。

### M4 — 收口

**目标**：DC 逐项判定 + 平台后端差异挂账定稿。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方「DONE 允许条件」 |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（本地等价用例如实标注）；
`cargo build` + `make test` + `cargo clippy` 全过；master.md 内部自洽，evidence 持久化。

---

## Execution Protocol

### 自主执行原则

1. **基线先行**：M1 纯资产切片零源码改动
2. **逐簇收敛**：WPT 失败聚类 → 逐簇修齐 → 账本更新
3. **跨域记账**：`:fullscreen` 样式面等渲染流域缺口记账回流，不越界硬改

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮
2. **平台剪贴板差异**：headless 内存后端口径先行，平台差异记账
3. **权限语义对齐**：security-hardening DC-4 落地时对齐替换本 goal 最小面

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。
  仅在目标本身实质变化时修改；每轮执行不重写。
- **运行时控制平面** `docs/goal/web-api-batch2/master.md`：当前真实状态唯一控制面板。
- **归档区域** `docs/goal/web-api-batch2/archive/`：只追加不修改。
- **证据区域** `docs/goal/web-api-batch2/evidence/`：WPT 基线/修齐账本，持续追加。

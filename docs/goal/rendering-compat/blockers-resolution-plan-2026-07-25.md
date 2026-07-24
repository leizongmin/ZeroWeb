# 阻塞问题解决方案 — Rendering / JS Runtime / 窗口

**日期**：2026-07-25
**性质**：方案层（根因 / 解锁路径 / 切片 / 依赖 / 推荐），非代码实施。供架构方向决策。
**关联**：[phase-a-IFC-unification-design.md](phase-a-IFC-unification-design.md)、[unified-font-stack-design.md](unified-font-stack-design.md)、[linebox-metric-unification-rfc.md](linebox-metric-unification-rfc.md)、[js-dom-bridge-design.md](js-dom-bridge-design.md)、[master.md](master.md)
**可执行首切片（实施入口）**：Phase A → [phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md](phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md)；P1b → [p1b-rfc-2026-07-25.md](p1b-rfc-2026-07-25.md)（v0.3 选型）+ [p1b-slice1](p1b-slice1-event-loop-microtask-mechanism-2026-07-25.md)（已撤销 microtask，保留历史）

---

## 0. 概览

ZW 当前「吭哧一月只 3pp」的深层根因：项目进入**架构投资期**，增量小修空间收窄。四大阻塞全部超出「单会话小修」范围，每条都被一个架构前置卡住。摸底（表单→Phase A、fetch→异步通道、MutationObserver→节点身份、vertical→同 Phase A）从多个独立角度撞到同一面墙。

| 阻塞 | 影响面 | 当前状态 | 解锁杠杆 |
|---|---|---|---|
| **Phase A** IFC 三路径统一 | reftest 大盘 57%、37-form-controls overlap、vertical-mode | Phase 1/2 部分 LANDED，Phase 3 未解 | **最高** |
| **font-stack** | 字体像素（css-text/css-fonts/welcome 描述文本） | plateau-accepted（M18 暂不优先） | 中（须推翻 M18） |
| **P1b** JS Bridge 原生化 | fetch/MutationObserver/事件循环/交互式网站 | selector-shim，未原生化 | 高（独立于 Phase A） |
| **P3** GPU/Display 窗口 | 真实窗口/GPU 验收 | WSL2 headless 受限 | 低（环境非代码） |

---

## 1. 依赖图

```
Phase A (line-box metric coherence) ──┬──> reftest 大盘 (57% → ?)
                                       ├──> 表单 overlap (37-form-controls)
                                       ├──> vertical-mode (R109)
                                       └──> font-stack 的 metric 半（advance/line-height coherence）

font-stack (raster/shape coherence) ──> 字体像素（metric 半依赖 Phase A，raster/shape 半独立）

P1b (V8 原生绑定 + event loop) ──┬──> fetch 真实化
                                  ├──> MutationObserver 真实化
                                  ├──> 事件循环 / setTimeout
                                  └──> 交互式网站（独立于 Phase A / font-stack）

P3 (GPU 环境验证) ──> 真实窗口/GPU 验收（独立，环境依赖）
```

**关键判断**：
- **Phase A 与 P1b 互相独立**（渲染 vs JS 运行时），**可并行两轨**。
- font-stack 的 metric 半 = Phase A 的子集（line-box metric coherence 共享）；raster/shape 半独立。
- P3 完全独立（环境），不阻塞任何代码推进。

---

## 2. Phase A — IFC 三路径统一（最高杠杆）

### 根因
layout↔paint 的 inline formatting context 度量在**三路径**间不一致：
- **Path A（stored）**：`compute_final` 预存的 line-box metric
- **Path B（paint re-run）**：stored 失效时 paint 重算 IFC（R101/R125 large-font 死锁根）
- **measure 路径**：intrinsic sizing / remeasure 时的 IFC 度量

三路径的 baseline / ascent / descent / line-height 各算各的 → line-box 高度与 baseline 分歧 → vertical-align、inline-block line-box 贡献、换行精度全受影响。这是 reftest 57%、37-form-controls label overlap、vertical-mode 的**共同结构性根因**。

### 当前状态
- Phase 1/2 部分 LANDED：R207 `PHASEA_STORE_EXT` + R355 多行放宽 + R817 linebox Phase 2（+45 case）
- **Phase 3（line-box metric 完全统一）未解** = 真硬阻塞
- 墙②（multicol + 换行精度）、墙③（混合 inline+block 内容存储排除）
- v1.4 addendum（R1985，2026-07-24）裁决：「勿再以 line-box metric / inline-block identity 为独立 lever，fix 须随 Phase A 整体 unification」

### 方案（高层）
三路径收敛到**单一权威 line-box metric 源**——一次计算，三路径共用：
1. 定义权威 line-box metric 结构：per-font ascent/descent + **inline-block margin-box 贡献**（CSS §10.8.1）+ half-leading + strut
2. compute 阶段算一次，存到 fragment
3. Path A 读 stored；**消灭 Path B**（paint re-run 改读同一 stored，而非重算）
4. measure 路径用同一 metric 计算

### 切片（pre-authorized ruling #4，多 session）
1. **inline-block line-box 贡献**（§10.8.1）—— 直接解 37-form-controls overlap，且是 line-box metric 基础规则
2. **per-font metric 注入**（U1b-wiring dormant infra 已 LANDED R1637-R1639，激活即可）
3. line-box height = max(text line-height, inline-block margin-box, strut)
4. 消灭 Path B（paint re-run → 读 stored）
5. vertical-align baseline 对齐（va-117a 簇）

### 风险
- deadlock 史（R125/R206/R213）：单点改三路径之一 net-negative。**必须三路径协同**，非切片单点。
- 每切片 env-gated + A/B **三态门禁**：welcome <20% + linebox/css-text/normal-flow oracle 零回归 + self-source 通过率不降。净负即回退。

### 依据
phase-a-IFC-unification-design.md v1.4、linebox-metric-unification-rfc.md A2、R109 vertical-native-layout-design.md。

---

## 3. font-stack — 字体栈对齐（plateau-accepted）

### 根因
ZW 字体栈（fontdue 光栅 + 启发式 advance + 常数 metric）≠ chromium（FreeType 光栅 + Skia shaping + per-font metric）。

### 当前状态
- FreeType 光栅已 default-on（R1068，+232 case）
- 残余 font-wall 在 **metric coherence**（advance / line-height / generic-vs-explicit）
- advance 轴 fully closed（R1946/R1950）；per-font line-height 三证 refute（R1185/R1636）；Skia net-24 ruled out（R1560）
- **孤立切片全 net-negative**，须 full coherence rebuild
- **2026-07-17 用户裁决接受 plateau**（reftest ~57%）；ROADMAP M18 暂不优先

### 方案（若推翻 M18）
full font-stack rebuild —— layout/paint/wrap metric coherence + chromium-matching raster/shape：
- C1/C2：per-font metric 区分 generic（~1.15 Blink 默认）vs explicit（fontdue hhea）
- C3：advance in layout（须 FontLoader Rc-share refactor，昂贵）
- raster/shape：rustybuzz production shaping + AA/subpixel 对齐

### 决策点
当前已接受 plateau。要做须同时：**(a) 推翻 M18 暂不优先** + **(b) 授权 multi-week** + **(c) 接受孤立切片不可蹭**（必须整体 coherence）。
**metric 半依赖 Phase A**（line-box metric coherence 共享）→ Phase A 完成后，font-stack 的 metric 半自动解锁一部分，再评估是否推翻 M18 做 raster/shape 半。

### 依据
unified-font-stack-design.md v0.2.3、font-wall-cdep-scoping.md、skia-cdep-rfc.md。

---

## 4. P1b — JS Bridge 原生化（解锁交互式网站）

### 根因
JS↔DOM 桥接是 **「selector 快照 + 批处理 mutations + 同步 execute」** 模型（dom_bridge.rs / js_dom_bridge.rs）：
- JS 操作 → `__zw_*` callback → DomMutation 列表（**selector-based**，非节点身份）
- 无持久 JS 节点身份（元素是 selector handle，非 node ref）
- `execute_script_direct` 同步执行，**无 microtask/task queue**

后果：fetch（Promise 异步）、MutationObserver（childList 需节点身份 + microtask 触发）、事件循环（setTimeout 真实延迟）**全都只能 stub 或近似**，无法 spec-真实化。attributes MutationObserver 近似可做但多文件 + 递归风险 + 偏离 spec，性价比低（详见 R2025 评估）。

### 方案（高层）
V8（已 feature gate，rusty_v8 / rquickjs）**原生绑定 DOM** —— JS 直接持有 Document 节点引用 + HTML spec event loop：
1. **节点身份**：JS element = Rust `NodeId` 的 wrapper（非 selector）→ MutationObserver childList 可行
2. **event loop**：microtask queue + task queue + rAF + requestIdleCallback → Promise.then / fetch / setTimeout 真实
3. 在此之上 fetch/MutationObserver/setTimeout 自然 spec-真实

### 切片（需独立 P1b RFC）
1. **event loop microtask queue** —— 让 Promise.then 真实（fetch/Observer 基础）
2. **节点身份 wrapper**（selector → NodeId）—— 让 MutationObserver childList 可行
3. fetch 走 net crate（异步，microtask resolve）
4. MutationObserver 真实触发
5. setTimeout/setInterval 真实延迟

### 风险
- 架构级（替换 dom_bridge shim 全部 `__zw_*` 机制）
- rusty_v8 原生绑定 API 复杂，跨进程（renderer/browser 双入口）一致性
- master.md line 2743 已列 P1b「架构级改造，需独立 RFC」

### 依据
js-dom-bridge-design.md、dom_bridge.rs、js_dom_bridge.rs、master.md P1b。

---

## 5. P3 — GPU/Display 真实窗口验证

### 根因
当前开发环境 WSL2 headless，无真实 GPU/display。

### 方案
三平台（macOS / Linux / Windows）真实窗口渲染验证 + GPU 加速合成 smoke：
- 各平台真实窗口启动 `zero-browser`
- wgpu GPU 加速合成验收
- 三平台渲染一致性 smoke

### 切片
环境搭建 + 验收 smoke（非代码逻辑，是环境 + 验收门禁）。

### 风险
环境依赖（需真实桌面 GPU），非代码阻塞。功能不依赖，优先级低。

---

## 6. 推荐执行顺序

**并行两轨**（Phase A 与 P1b 互相独立）：

### 轨 A：Phase A IFC 统一（最高杠杆）
- pre-authorized ruling #4，**可立即开**
- 解锁最多方向：reftest 大盘 + 表单 overlap + vertical + font-stack metric 半
- **首切片**：inline-block line-box 贡献（§10.8.1）—— 解 37-form-controls overlap，且是 line-box metric 的基础规则，独立可验证
- 多 session 推进，每切片三态门禁

### 轨 B：P1b JS 原生化（交互式网站 = M11 最后缺口）
- 需先出 **P1b 设计 RFC**（架构级，master.md 已预留）
- **首切片**：event loop microtask queue（让 Promise 真实，是 fetch/Observer 基础）
- multi-month

### 暂不做
- **font-stack**：保持 plateau-accepted（M18 暂不优先）。等 Phase A 完成后 metric 半自动解锁，再评估推翻 M18 做 raster/shape 半
- **P3**：低优先，环境就绪时验

---

## 7. 决策点（需用户拍）

1. **启动 Phase A 轨 A**（pre-authorized，可立即开，最高杠杆）—— **推荐：是**
2. **启动 P1b RFC**（轨 B，架构级，解锁交互式网站）—— **推荐：是**（M11 最后非环境缺口）
3. **font-stack 推翻 M18** —— **推荐：否**（保持 plateau，等 Phase A metric 半）
4. **P3 何时验** —— 低优先，环境就绪时

---

## 附：本轮（2026-07-25）已落地的底盘稳固（不依赖架构决策）

- **legacy smoke 门禁可见性**：run-all.sh struct FAIL 现打印 issue 详情（37 号 5 issue 可见，诊断入口打通）
- **基线校准**：README 从过时 R1690（49/51）更新到当前 50/51（27 R1743 PASS，37 Phase A 阻塞）
- **门禁记录**：run-rules.md 补 product-smoke-legacy 常态门禁
- **关键 RFC status 校准**：phase-a-IFC-unification-design.md 过时「草稿/未落地」→ 反映 v1.4 裁决 + Phase 1/2 部分 LANDED + Phase 3 阻塞
- **memory 沉淀**：reftest 57% = 已接受 plateau（勿再建议 font-stack 解锁）；ZW_ 开关 `!=Ok("0")` = default-on 语义（判 dormant 看写法）

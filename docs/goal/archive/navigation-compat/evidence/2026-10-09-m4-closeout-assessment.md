# M4 收口评估（2026-10-09）

**依据**: S4I 后全量语料 [2026-10-09-m2-s4i-full-corpus.txt](2026-10-09-m2-s4i-full-corpus.txt)
（442 子测试 308 Pass = 69.7%；navigation-api 169/230 = 73.5%）；门禁 make test 20,233P/0F、
reftest 704/704、clippy -D warnings、fmt 连续全绿。

## DC 逐项判定

| DC | 判定 | 依据 |
|---|---|---|
| DC-1 WPT 导入与基线 | ✅ **满足** | M1 runner 通道 + 四 corpus fetch + 基线 20.9%（[M1 证据](2026-10-07-m1-baseline.md)）；`docs/compat/trends/wpt-suites.csv` 逐切片回填至 308/442 |
| DC-2 会话历史与导航事件（轻面） | ✅ **主簇满足** | history-interface **91.8%**（S3）、the-location-interface **95.3%**（S1/S4F）、history-traversal **93.3%**（S2）、navigation-api **73.5%**（S4~S4I 十二段：read side/navigate/intercept/traverse/scroll-behavior/focus-reset/exotic/precommit/traverseTo/ongoing-abort/downloadRequest/host 激活）；残余逐项定性见下表（非 DC-2 主簇语义缺口） |
| DC-3 iframe 浏览上下文（用户门控） | ⏳ **pending（如实标注）** | 2026-10-04 已征询、2026-10-06 48h 跟催零回复 → 维持立项态挂起（不回复即默认挂起）。**用户未明确豁免 → goal 不判 DONE**。replace-before-load 38F 勘察重定性为本 DC 依赖（iframe 自有 session history 载体，2026-10-09） |
| DC-4 测试与质量不可退让 | ✅ **满足** | 每轮 `make test` 全绿（20,233P/0F）+ clippy -D warnings 零 warning + fmt 零 diff + `make reftest` 704/704（S4D~S4F shim 变更后统一复验）；每语义切片带 WPT 语料断言（十二切片证据链齐全）；每轮全量语料 per-subtest 精确 diff 零回归 |

## 残余定性（136 Fail/Timeout 全量盘点）

**A. 域回流（不属本 goal envelope，记账回流）**
- DOM 焦点域 ~9 案：Tab 键顺序焦点导航（2T）+ autofocus load 期处理（7 NotRun）——回流
  web-api-batch/dom goal。
- 渲染域 ~20 案：scroll anchoring + rect 快照刷新（scroll-behavior reload 族 4F）、
  scroll-to-fragid 几何/编码变体（~14F）、scroll-position writing-mode 面——回流
  rendering-compat。
- element IDL 域 2 案：静态主文档锚 `a.href = v` IDL 赋值不落 attr（expando 形态）——
  S4I 已绕行（expando-first 读），attr 持久化面回流 dom goal。

**B. runner 形态（单文档 runner 不可达，形态升级前挂账）**
- replace-before-load 38F：iframe 自有 session history（M3 依赖）。
- bfcache 族 ~10 案：entries-after-bfcache / dispose-after-bfcache / activation-after-bfcache /
  navigation-history-back-bfcache / pagereveal ×2——bfcache 未立项（见 P5 定稿）。
- 跨文档导航链 ~6 案：navigating-across-documents / same-url-replace-cross-document /
  cross-document-away-and-back——依赖真跨文档 load 流。
- user-activation 深面：navigation-activation 的 transient-activation 时序面。

**C. 可切片（shim 内，后续轮次余量 ~25 案）**
- form submit → navigate 事件族 5T（formData/sourceElement/navigationType——S4J，参照锚路径）。
- state classic/nav 分槽 2F（history-pushState/replaceState 的 entry.getState() undefined 面）。
- navigation-activation 对象暴露 4F（`navigation.activation` 同文档恒定性语义）。
- dispose/reload 深簇 ~6F（dispose-same-document 事件序变体）。
- 零散：create-script-set-location（跨文档 load 序）、004 xhr helper infra、optional 速率限制 ×2。

## P5 挂账定稿（bfcache / fission）

- **bfcache**：未立项、无实现面（back/forward 均为同文档内存会话，无 Document 快照持久化）。
  依赖序：frame tree（M3）→ Document 快照/恢复管线 → bfcache 语义。受影响语料 ~10 案
  （entries/dispose/activation-after-bfcache、pagereveal、history-back-bfcache）——挂账至
  M3 后续立项，本 goal 内不再推进。
- **fission**：charter 明确排除（多进程 frame 归属触 zero-protocol 契约，M3 触发即 BLOCK
  上报用户）。维持排除定稿，本 goal 不触碰。

## 结论

M1/M2/M4 判定完成，DC-1/2/4 满足；**DC-3 维持用户门控 pending（未获豁免）→ goal 不判
DONE，维持 Active 推进态**。可切片余量（C 类 ~25 案）由后续轮次按 S4J（form submit）→
state 分槽 → activation 暴露顺序推进；A/B 类挂账已定稿回流。

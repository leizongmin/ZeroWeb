# M3 追加轮：append sel 项通道补齐——slot-fallback-content-006 翻绿根因修复

**日期**: 2026-11-08
**切片**: 45 号 M3 追加轮（R3b 落账后 006 未翻绿的「另行调查」收口）
**提交**: 见本文档提交行（part05.js `_appendVariadic` sel 项臂一处）
**结论**: slot-fallback-content-006 从 4.90% 稳定红翻绿（solo 0.07%，DC-4
矩阵 33P/2F 达标）。根因不在 R3b manual 通道，而在 shim `Element.append()`
变参通道对 **sel-backed 静态解析元素**（仅有 `__zwSelector` 无 `__zwHandle`）
的分型缺失——`_appendVariadic` 节点判定只认 `item.__zwHandle`，此类项掉入
文本分支被 `String(item)` 化。同型洞已在 insertAdjacent 漏斗修过（R4200），
append/replaceChildren 通道漏网。R3b 通道本身工作正常，006 此前从未走到。

## §1 归因链（探针分型，.acceptance/r3b-probe*.html + PNG）

006 的脚本对五个 host 各做三步：`attachShadow({slotAssignment:'manual'})`
→ 把 light DOM 里的 `<slot>` `appendChild` 进 shadow root → `host.append(A,B)`
→ `slot.assign(B,A)`。R3b 前后像素差逐位相同（4.90% = 23543/480000）→ 怀疑
manual 通道从未被执行。分四步定位：

1. **probe1（createElement 形态）**：createElement slot + appendChild 移动 +
   assign——JS 侧全正常（manualList:2、handle 全有、bridge 在），渲染只见
   回退内容、light 子不漏。→ 坏相必然来自 006 特有路径（parse 产元素 +
   `host.append()`）。
2. **probe2（006 host1 形态复刻）**：`host1.append(A,B)` 后 hostKids 出现
   `nodeType:undefined` 坏壳×2；`assignedNow:0`（parent!=host 过滤清零）；
   渲染三重坏相——`[object HTMLDivElement]`×2 垃圾文本叠回退行、真身 A/B
   以文档序滞留 body 层渲染（即「反序」假象）、回退内容不消失。
3. **probe3（变量分离）**：`append`×handle 参数 ✓；`append`×id-global ✗
   （坏壳 + 真身不动）；`appendChild`×id-global ✓。钉死 append 变参实现。
4. **probe4（源码身份）**：`host1.append` 与 createElement 产物为同一份
   R117 实现（`_zwValidatePreInsert` + `_appendVariadic(sel, handle, args)`）；
   id-global 元素 `__zwHandle:none __zwSelector:#A`——sel-backed 形态实证。

## §2 修复（part05.js `_appendVariadic`）

镜像 insertAdjacent 漏斗的 R4200 sel 项臂（该处注释原话即同型洞：「此前掉进
下方文本分支（`String(item)` = '[object Element]'）」），在文本分支前加：

- `__zw_insert_adjacent_sel_element(sel, 'beforeend', item.__zwSelector)`
  （R334/R182 wire；'beforeend' = append 尾插；host `insert_nodes_at_position`
  自带 reparent 移动语义）；
- `_zwUnmarkRemoved` 清重插标记（R4200 同款）；
- `_zwSelPendingParent` + `__zwPendReselAdd` + `_zwSiblingBaseInvalidateAll`
  同 turn 可见性（R4200 同款）；
- `added.push(item)`——R117 包装层按 `added[]` 自动接 MO childList notify +
  CE connected + iframe 启动 + img relevant mutation（`__zwImgTreeMutated5277`）。

已知边界（FIXME 注明在码）：handle 父（sel 为空）+ sel 项组合未覆盖——
appendChild trap 的 C3 移动语义块未镜像进本函数；appendChild/replaceChildren
的 handle 容器变体行为不变。R192 去重只对 handle 项记账，sel 项经顺序 wire
自然涌现 spec 后位胜语义（append(x,y,x) → [y,x]）。

## §3 验证账

| 门 | 结果 |
|---|---|
| probe2/probe3 复跑 | `B(append,idglobal):1(m1),1(m2)` 真身归位；006 host1 形态渲染 **B、A 正序**，垃圾绝迹（r3b-probe2-fixed.png / r3b-probe3-fixed.png） |
| solo slot-fallback-content-006 | **✓ 0.07%**（修复前 4.90% 稳定红） |
| DC-4 矩阵 `make reftest-upstream FILTER=shadow-dom` | **33P/2F 达标**（006 翻绿；余 2F = vw-units 2.08% + display-contents-shadow-dom-1 1.97% 在册）——legs8 断言 P=33 F=2 |
| reftest 全量 `make reftest` | **rc=0（725/725 零回归）**（legs8） |
| product-smoke | **rc=0**（legs8） |
| clippy-v8 | **rc=0**（legs8） |
| bench-gate（dom/engine 定向） | **rc=0**（legs7，净树 d34a3404d） |
| fmt | legs8 rc=1——extract.rs/webview.rs 三处 rustfmt 违规系既有提交遗留（全量 --check 首次暴露），机械修整后 legs9 rc=0 |
| B6 基线池 | 15P/86F 基线零位移（legs8 复跑断言） |
| make test 全腿镜像（legs9） | fmt ✓ / quickjs-clippy ✓ / quickjs-ren-bin ✓ / quickjs-ren-lib **370P/0F** ✓ / renderer-bin ✓（0 测试面，与 make test 同型） / renderer-lib ✓；**workspace v8 腿 42P/11F 待重跑**——11F 全部为 webdriver http_session 环境性失败（`target/debug/zero-renderer` 工件被当日多轮 feature 翻转链清掉 → zero-webdriver init 35ms 即退、TCP 探测错过存活窗 → spawn 锁中毒级联；cargo fail-fast 使首败二进制中止整腿，42P 远低于基线 3814P/6F。非本修复行为面——规范 make test 有 standalone binary 构建前置，手搓镜像腿漏了该步；余腿 browser×2/gpu/quickjs-crates/bench 在跑，workspace 腿待链毕带构建前置重跑回填） |

JS 读侧已知近似（不在本修复范围）：`assignedNodes()` 的 parent==host 过滤
比较 wrapper 身份，重插后 live 读可能构造新 proxy 致 `===` 失配
（probe2-fixed 报 `assignedNow:0` 而渲染正确——引擎态 parent 链已正确）。
006 脚本不读 assignedNodes，不影响矩阵；如后续 WPT 需要再立切片。

## §4 语料与探针卫生

- 协同方调查遗留的 6 个 `slot-fallback-content-006-probe*` 页面曾入
  `wpt-data/shadow-dom/`（矩阵目录递归会多计 4 案破坏 DC-4 计数口径），
  已挪 `.acceptance/r3b-collab-wpt-probes/` 保真，语料恢复 35 案规范态。
- 本轮探针（r3b-probe1-4 + 006 渲染取证 PNG）全在 `.acceptance/`，不入库。

## §5 关联

- R3b 主体/活过滤/cfg 门/B6/时序测试修复的落账与门禁账见
  [R3b evidence](2026-11-08-m3-r3b-manual-slot-channel.md)（pull --rebase
  后哈希映射以该文档提交行为准）。
- 本修复使 R3b manual 通道在 006 实际生效——R3b 的「目标案未收口」状态
  解除，控制面已随本轮更新（006 翻绿、矩阵 33P/2F、下一步切净树余腿复核
  + M4 收口）。

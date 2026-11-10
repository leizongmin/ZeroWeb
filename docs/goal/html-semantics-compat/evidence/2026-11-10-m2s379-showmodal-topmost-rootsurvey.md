# M2 切片 379（s379）——dialog-showModal topmost 勘根定性轮（零源码改动）：主阻塞 R-s31 几何缺席 + efp 栈序感知次缺口附注

日期：2026-11-10（承 s378 挂账：dialog-showModal 1F 勘根定性——最后一个未定性流内真案）
域：html/semantics/interactive-elements/the-dialog-element（dialog-showModal）
性质：**勘根定性轮（零代码改动，域计数零位移 15 真案维持）——showModal topmost 1F 定谳 R-s31 UA dialog 定位簇扩容**

## 案面

「when opening multiple dialogs, the most recently opened is rendered on top」5 断言链：d10/d11 showModal 双开 → rect 相等断言（ZW 全零===全零假过）→ `elementFromPoint(中心)` 断言序列：①===d11（后开栈顶）②d10.removeAttribute("open") 后仍 ===d11 ③d10.showModal() 重开后 ===d10（重开提升）④d11.close() 后 ===d10。红态：①即红 `expected <dialog id="d11"> but got null`。

## 探针证据链（zz_probe_s379 双形态，勘根毕删）

1. **无前置内容形态**：r10=r11=(0,0,0,0) 全零；efp(0,0)/efp(1,1) 全返 **p11**——s373 缺席臂兜底（body gBCR 全零 800x0 门开 → 栈顶模态 dialog 视口命中）正确返回栈顶。
2. **前置双 P 内容形态**（复刻真案 body 有布局）：r10=r11 仍全零，但 efp 三点全 **null**——body 有布局 → 兜底门关 → 正常几何扫描命中 body/P（dialog 零几何无候选）→ 命中物被栈顶模态 inert 屏蔽 → null。

**全链吻合真案**：dialog-showModal.html 页面内容多 → body 有布局 → 兜底门关 → dialog rect 全零无法被命中 → inert 内容命中被屏蔽 → null。

## 定谳（双层归属）

**主阻塞 = R-s31 UA dialog 定位簇（渲染流）**：dialog rect 全零（无 UA absolute 定位+fit-content 盒；s376 探针同证 dialog pos=static）→ efp 几何面无法命中 → 5 断言链的第一环（efp 命中栈顶 dialog）即断。R-s31 落地（UA 定位注入使 dialog 有真实盒覆盖中心）后断言 ①②④ 自动通。

**次缺口 = efp 同面积 topmost 文档序近似不感知 top layer 栈序（engine part06 efp 面）**：断言 ③ 重开提升（d10.showModal 后栈序 [d11,d10] 栈顶 d10）——Chrome hit-test 按 top layer paint order 命中栈顶 d10；ZW efp「同面积文档序靠后」近似恒选 d11（文档序靠前）→ 残红。**R-s31 落地前不可验证不可修**（rect 全零走不到同面积比较）——挂账 R-s31 簇附注，R-s31 落地后若残红再修（届时真几何可验证）。

**shim modal 栈序语义本身正确**（源码核实）：showModal `push` 栈尾（part01.js:786/843）、close/attrClosed `splice` 出栈（:880/960）——重开提升语义与 spec top layer 顶部插入一致，非缺口。

## 域口径变化

dialog 域真案 **15F 维持**（零代码零位移），归属结构精确化：**R-s31 簇 14**（abspos 6、bounds 2、multiple-centered、non-modal、scroll-height、top-layer-position、ancestor-is-inert、**showModal topmost 1F 新入簇**）+ corner-cases 1 上游单列。跨流既有面 20 行单列维持；closed-details 出域 1 维持。

## 挂账（移交下一切片）

1. **R-s31 UA dialog 定位簇扩容至 14 案**（渲染流）：新增 showModal topmost（efp null 几何缺席形态）；**附注次缺口**：efp 同面积 topmost 需 top layer 栈序感知（part06），R-s31 落地后残红时修。
2. customizable-select A2 簇、image input replaced sizing 缺口（s378 挂账）、跨流回归面 20 行维持渲染流控制面。
3. 流内剩余真案口径清零：15 真案全部归属挂账簇（R-s31 14 + 上游同败 1）——**流内可施工候选耗尽**，下一切片候选转向基建面（T=3+probe8/9/10 转正决策）或跨流面复验（渲染流消化 R-s31/1bcfaac6d 归因账后全域 join）。

## 验证

- 真案 solo 10P/1F 红态维持（定性轮零代码零位移）；探针双形态实证 + 栈序源码核实
- 探针勘根毕删（zz_probe_s379.html）；零源码改动 → lib/fmt/clippy/product-smoke 全不适用

日志 /tmp/s379/（本地；探针输出见本轮执行记录）。

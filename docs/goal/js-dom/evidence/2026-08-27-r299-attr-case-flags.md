# R299 Evidence — 属性选择器 `i`/`s` 大小写标志（JS 匹配器剥离 + 语义；mixed-case 归因收窄）

**日期**: 2026-08-27
**切片**: M4——R299(a) selector 小簇收尾（mixed-case 1F 首断言域）
**改动面**: `part05.js`（`_parseAttrInner` 值尾标志剥离 + `_matchAttrOf` 的 `i` 折叠）+ `js_dom_bridge_tests/part24.rs`（+1 单测）

## 一、根因（真实文件注入探针，R222/R274 方法论）

WPT `querySelector-mixed-case` 唯一测试是**双阶段**（detached `tree.querySelectorAll` →
append 进 document 再 `container.querySelectorAll`），首断言 Test 20
`[testAttr="alpha" s]` expected 1 got 0。

**探针数据**（assert_unreached 注入，跑完 restore）：

| 阶段 | 形态 | 基线 | 结论 |
|---|---|---|---|
| detached | `[testAttr="alpha" s]` | `s=0` | **JS 匹配器不解析值尾标志**——`_parseAttrInner` 的 `.*?` 把 `" s` 吞进值（去引号只剥首引号）→ `alpha" s` ≠ `alpha` 全 miss |
| detached | `[viewBox]` | `4`（正确） | Exists 形态无回归面 |
| **indoc** | `[viewBox]` | **`all=0 / svg=0 / #html1=0 / vb=0`** | **宿主完全看不见 append 进 document 的 JS 建树**——handle 子树只存在于 pending mutations，sel-based 容器查询走 host 快照 → 0 命中（**基线同败**——预存缺口，非本修复引入） |

即：mixed-case 1F 是**两层**叠加——① JS 匹配器缺 `s`/`i` 标志解析（detached 阶段
Test 20 起 fail）；② 宿主 sel-based 查询对 append-in 的 handle 子树不可见（indoc
阶段 Test 1 起 all=0，双阶段测试在 ① 修复后暴露 ②）。②是 R220
（registry vs host 树分歧）家族的又一实证。

## 二、修复（本切片解 ①）

- `_parseAttrInner`：值尾剥离 `(.*?)\s+([iIsS])$` 形态的标志（值与标志间须有空白
  分隔——裸 `i` 值非标志，`[k=i]` 的值是 "i"）；引号先剥外层再匹配标志（与 dom
  crate `strip_attr_case_flag` R200 同源语义）；
- `_matchAttrOf`：`i` → 值比较双侧 ASCII 小写（六个运算符统一）；`s` → 恒等
  （精确比较天然覆盖）。

探针复验：`s=1|sUP=0|i=2|inc=2|plain=1|bareI=1`（s 精确命中、大小写不匹配
不命中、i 双侧折叠、`*=` 组合、裸 i 值）。

## 三、验证

| 套件 | 基线 | R299 | Δ |
|---|---|---|---|
| querySelector-mixed-case | 0P/1F（Test 20） | 0P/1F（**失败点前移 Test 1 indoc——② 暴露**） | 文件级不变；detached 阶段 41 断言全通过（探针实证） |
| ParentNode-querySelector 全族 | 2050P/5F | 2049P/5F | 持平（-1 为 All-content flaky 已知噪声；Fail 集合不变） |
| case-insensitive（`i` flag 消费方） | 2P/0F | 2P/0F | 持平 |
| Element-matches / Element-closest / Attr 族 | 675P/29P/52P | 同 | 持平 |
| engine 单测 | 2436 | **2437**（part24 +1：s/sUP/i/inc/plain/bareI 六断言） | +1 |
| make test | — | 1F = XOpenDisplayFailed 环境项（fetch_proxy 一次失败为并行 cargo 竞态，复跑消失） | 持平 |
| fmt / clippy | — | 干净 | — |

## 四、归因记档（mixed-case 转深结构）

- **② 宿主 sel-based 查询不见 append-in handle 子树**：`container.appendChild(jsTree)`
  后 `container.querySelectorAll('*')` 恒 0——handle 子树在 pending mutations，
  `apply_pending_query_html` 未桥接 handle-append 形态。与 R291「querySelector
  wrapper identity」/R292 归一/R220 家族同域。**修复方向**：handle-append 的
  sel-side 视图桥（append 时递归物化子树到 host 或 registry-fused 查询）——
  L2 live Document 域深结构，本切片不展开。
- 文件级 mixed-case 1F 从「selector 小簇」转入「R220 深结构」清单；selector 小簇
  （escapes 2F + scope 2F 已解 + mixed-case detached 域已解）至此**收尾**。

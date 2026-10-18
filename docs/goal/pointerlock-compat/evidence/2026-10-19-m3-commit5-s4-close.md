# M3 Commit 5 / s4 收口——FR-7 双域 retarget + corpus 总账（2026-10-19）

M3 最后切片。具体变化：① TBD-1 资产补拉落地（goal 脚本 `fetch_raw`）；②
FR-7 shadow retarget 重写为**双域**实现（实施期发现单 handle 域不可达）；③
corpus 总账收口（通道 19/22、文件 15/18、csv 回填、TBD 清账）。门禁：
make test 全绿、make reftest 712/712 零回归、clippy 干净（make test 内嵌腿）。

## 1. TBD-1 已决：shadow-dom helper 资产

`tests/wpt-runner/scripts/goals/29-pointerlock-compat.sh` 增 `fetch_raw
"shadow-dom/resources/shadow-dom.js"`（pin 31597693，5341 bytes；20-net-api-compat
单文件补拉先例，不入 DIRS 扫描面）。提供 createTestTree/walk/attachShadowFromTemplate。

## 2. FR-7 双域 retarget（实施期发现 + 修复面）

### 2.1 发现链（探针证据）

初始实现按 handle 域 retarget（`_plRetargetViews` 从 `_makeProxy(_plSel,_plHandle)`
上行、`_shadowHandles` 判界、`byRoot[handle]` 键），实跑 `pointerlock_shadow`
报 `expected (object) null but got (undefined) undefined`。逐层探针排除：

| 探针 | 结果 | 结论 |
|---|---|---|
| `fc.getAttribute('id')`/`outerHTML`/`querySelector('#c')` | `"c"`/`<canvas id="c">`/命中 | template.innerHTML 解析丢 id 的疑点**排除**（`.id` undefined 是另一族包装的反射面问题，与本案无关） |
| `createTestTree(host)` keys | 11 键全注册 | ids map 完整——红不在 helper |
| `DOC/R/R2..R5` 六读 | DOC=null、R=null、**R2..R5=undefined** | undefined 只出自轻量 shadow 包装 |
| `R{host=HTMLDivElement}` vs `R2{host=[object Object]}` | 两族 | `ids.root`（走 part04 trap attachShadow → `_wrapHandle`）与 `ids.root2`（走原型**轻量路径**）分族 |
| `clone===p1`（有 `__zwHandle='__n7'`）vs `p2.__zwHandle=undefined` | 两族父 | 克隆树**内部节点是 R307 实例域对象**（无 handle）；`ts[1].parentNode` 的 attachShadow 解析到 `Element.prototype` 轻量路径（part03 R179，`[object Object]` 产物） |

根因：`createTestTree` 的 `walk(node.cloneNode(true))` 克隆树整体在实例域——
锁目标 canvas、五个 shadow root（全部轻量产物）、host2/host3 全部无 handle；
handle 域 retarget 对这棵树**完全不可达**。learning #32（多族包装并存）实例。

### 2.2 修复面（4 文件）

1. **part06**：
   - 锁态新增 `_plTarget`（锁定元素**原对象**，任意包装族）；grant/`exitPointerLock`/
     `_plOnNodeRemoved`/`__zw_reset_form_state`（part01）四站点同生灭。
   - `_plRetargetViews` 重写为**域无关 composed 上行**：从 `_plTarget` 沿
     `parentNode` 上行，parent 鸭面判定 `nodeType===11 && nodeName==='#shadow-root'
     && host` 即跨界（同时覆盖 handle 域 `_wrapHandle` 产物与轻量 shadow 对象）；
     root 视图 = 跨界时节点，`docView` = 最外层跨过的 host。返回
     `{docView, byRoot, roots, views}`——`byRoot` 按 handle（part04 trap 分支消费）、
     `roots`/`views` 平行数组按**对象身份**（ShadowRoot.prototype accessor 消费，
     轻量 shadow 无 handle 可键）。
   - grant 流抽 `function _plRequestLock(rawElem, sel, handle, options)` 共享
     （①options 校验 ②host deny ③激活门 ④同元素同选项 no-op ⑤grant+change+resolve，
     语义与 part04 原内联版逐条一致）；实例域 sel/handle 双 null → `_plKey` 走
     `_ceConnKeyFor` 对象身份盖章（`_elKey(null,null)` 返 null 会毁「已锁」真值判定）。
   - document getter 未跨边界回落改 `_plTarget` 优先（trap 域 = 缓存 proxy 同
     `_makeProxy` 产物；实例域 = 活树原对象——`_makeProxy(null,null)` 无意义）。
2. **part04**：trap `requestPointerLock` 分支改为委托 `_plRequestLock(this, sel,
   handle, options)`（`this` = proxy，`_proxyCache` 缓存 identity 稳定）；行为等价。
3. **part03**：
   - `Element.prototype.requestPointerLock` 轻量回落（guard 幂等，同 attachShadow
     原型面先例）——R307 实例域元素无 proxy trap，经原型链达 `_plRequestLock`。
   - `ShadowRoot.prototype.pointerLockElement` accessor——轻量 shadow 经原型获
     DocumentOrShadowRoot 语义，按对象身份扫 `roots`/`views`；**非 enumerable**
     （WebIDL 原型 accessor 缺省 + R3046 for-in 零变化先例）；handle 域产物由
     part04 trap 分支先答（proxy trap 先于原型查找），不达本面。
4. **part01**：`__zw_reset_form_state` 补 `_plTarget = null`。

### 2.3 实跑验证

`pointerlock_shadow.html` 全案 **Pass**（翻绿）：

- 默认 null 面 6 读（document + root..root5）全 null；
- 锁定后：`document.pointerLockElement === ids.host2`、`ids.root2.pointerLockElement
  === ids.host3`、`ids.root3.pointerLockElement === ids.canvas`、
  `ids.root/id.root4/id.root5` → null——跨族 `===` 断言全过（实例域对象为活树
  原对象，identity 天然稳定）；
- `exitPointerLock` + change 事件 + `test_driver.Actions` 点击链全通。

## 3. 通道终账（Commit 5 代码态二进制）

`testharness-pointerlock` 通道：**19/22 子测试 Pass（86.4%），文件口径 15/18 全绿**。
Commit 4 基线 18/22、14/18 → 本提交 +1 子测试（shadow）、+1 文件。

红案 3（全部记账，不阻塞收口）：

| 案 | 归因 | 处置 |
|---|---|---|
| constructor.html `Default event values for mouse event interface and its pointer lock extensions`（1 子测试） | `Event.NONE` 等 Event 接口 phase 常量缺失——dom/events 域 residual（uievents 50 挂账同源，M1 已记跨域） | 跨域挂账，不占本 goal 分母口径（goal doc 既有决定） |
| wheel 案 ×2 | TBD-3：runner `Actions.scroll` 未实现合成 wheel 流 | 记账 runner 域；桌面锁定 wheel 转发同域挂账（webview 无 DOM wheel 派发底座，Commit 4 已记） |

TBD 清账：**TBD-1 已决**（§1）；**TBD-2 已决**（s2 实跑落定 `screenX==clientX`
合成假设成立，movementX_Y_basic 翻绿）；**TBD-3 记账**（上表 + 桌面 wheel 同域）；
**TBD-4 排除面**（OS 光标隐藏，远期池）；**TBD-5 已决**（C 面实跑残留 = 上表
3 红，归因清晰）。

## 4. 门禁（DC-3）

- `make test`：全腿绿（含 clippy `-D warnings` 腿与全 workspace cargo test）。
- `make reftest`：712/712 = 100.0%，零回归。
- `cargo build --release --bin zero-wpt-runner` 后通道实跑（learning #28：
  shim 修复以最终链接产物为证据，`grep -ac` 特征串核验）。

## 5. 未决（收口后移交面）

- TBD-3 runner `Actions.scroll`/wheel 合成 + 桌面锁定 wheel 转发——runner 域挂账。
- `Event.NONE` 常量——dom/events 域（uievents 50 residual）。
- TBD-4 OS 光标隐藏/显示——平台窗口面远期池。
- 桌面端 GUI 手验（真窗口锁光标观感）——WSL2 环境受限，挂账桌面域。
- 实例域锁的 `__zw_pointer_lock_state_changed` 镜像 payload `selector:null`（实例
  对象无代理坐标）——host 侧锁定转发按 selector 命中，实例域目标收不到锁定输入流；
  WPT 面不触达，记已知限制。

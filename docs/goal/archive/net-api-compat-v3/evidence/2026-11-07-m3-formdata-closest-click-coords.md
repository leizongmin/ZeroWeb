# M3（七）：formdata/submitter 簇 4F → 2F（+2P 零回归，13,253 → 13,255P），余 2F 归因重分类挂账

日期：2026-11-07。前置：M3（六）overridemimetype 族 + 响应解码链批（全量基线
**13,253P**/1,424F/15T，分母 14,692 行 / 14,621 键）。在账「formdata/submitter
簇 4F」逐案归因四分，两案收口、两案重分类挂账：

| # | subtest | 处置 | 根因 |
|---|---|---|---|
| F1 | xhr/formdata.html :: FormDataEvent entries | **修复**（三刀 shim part02/part03/part06 协同） | closest/枚举三重缺口（见 §一） |
| F2 | xhr/formdata/constructor-formelement.html :: form data set | **挂账重分类**（引擎字符串语义桥欠账） | unpaired surrogate 经 UTF-8 host 桥 lossy（见 §四） |
| F3 | xhr/formdata/constructor-submitter-coordinate.html | **挂账重分类**（渲染流域） | runner 布局把 200×200 svg 图像渲染 29px（见 §五） |
| F4 | xhr/formdata/submitter-coordinate-value.html | **修复**（webview 坐标通道 + shim 记账臂） | 合成激活 click 无坐标面（见 §三） |

## 一、F1 归因（closest 三重缺口，探针逐层剥）

xhr.spec / HTML spec 构造 FormData data set 步骤 + 三个临时探针页（breadcrumb
经故意 assert_equals 失败透传，收口前已删）逐层定位：

1. **plain 世界无 closest**——`_zwMEl` 解析产物（innerHTML/insertAdjacentHTML
   同 turn 解析节点、DOMParser/detached doc 树）经 R125 setPrototypeOf 接
   `Element.prototype`，该原型此前无 `closest`（trap 世界 proxy 的 closest 走
   part04 get trap 特判 s81/s82，原型方法只服务 plain 世界——R5238 同款分离）。
2. **sel-proxy 身份是 tag 级模糊 selector**——查询视图物化的同 turn 解析字段
   `__zwSelector = "input"`（无 id），s81 closest 祖先走被
   `_zwSelIdConstraint` 无 `#`/`[` 预门 return null（part05:6865）跳过 →
   回落 host `__zw_closest` 按文档首个同名元素解析（探针：文档含 2 form/
   多 input，`closest('input')` 连 self-match 都 null；`parentNode` identity
   反而正确）。
3. **枚举键碰撞吞字段**——s123 双源并集先到先得去重：doc 级逗号列表按文档
   首个同名 input 解析（probe docQList=1 落在他 form 留存 input 上）与子树
   源我方字段同键 `"input"`，正确者被吞（probe fd=[]）。

## 二、F1 修法（三刀）

1. **part03.js `Element.prototype.closest`（plain 世界）**：s82
   `_zwClosestSingleCompounds` 单 compound 判定 + `_matchCompoundOf` 直读
   属性面（tagName/id/className/getAttribute 对 plain 节点与 trap proxy 锚双
   可读），plain parentNode 链 48 跳封顶；complex/伪类保守不接（同 s82 口径
   ——`_matchCompoundOf` 不求值 pseudos）。参数校验镜像 matches（0 参
   TypeError / 非法 selector SyntaxError）。
2. **part02.js FormData 构造器 owner 判定分源**：字段枚举记源
   （`fieldsSrc`）——②源（form.querySelectorAll 子树查询）结构性属主由查询
   本身成立（spec 枚举 = form 祖先链后代；HTML 无嵌套 form），免 closest
   复验；①源（document 级）owner 判定保留（form= 属性站外关联排除语义不变）。
3. **part02.js 去重键碰撞子树源顶替**：`_fdSeen[k]` 记下标；同键碰撞且后来
   者为子树源时**原位顶替** doc 级占位（树序不变——静态页双源同元素顶替
   无害；s123 序约定不变）。

## 三、F4 归因 + 修法（合成激活 click 坐标通道）

探针定谳：test_driver.click → runner click 臂 → `HtmlUserAction::Activate`
→ `PlannedEvent::simple(target, "click", true)`——**无 clientX/clientY 通道**；
且宿主派发通道不跑页面 dispatchEvent 默认动作（R108 泛型协议），
part04 `_zwImgRecordClickCoords`（trap 闭包内嵌）只在页面事件路径记账，
testdriver 路径的 [0,0] 实为 runner requestSubmit 兜底的 unactivated 语义，
非 recorder 产物（probe：手动 dispatchEvent 记 [14,3]、host 通道记 {}）。

修法（跨三文件，全部 zero-web 流工作面）：

1. **webview `user_actions.rs`**：`dispatch_planned_event` 对 click/dblclick
   探测目标 in-view 中心（新 `probe_in_view_center`——testdriver.js
   `getInViewCenterPoint` 同款语义：视口钳制取中点；`window.innerWidth`
   未定义时只钳负半轴不压 0），经既有 `DomEventDetail.client_x/y` 通道透传。
   探测失败/detached 无盒 → None（缺省面不变——与 unactivated 图像钮
   selected coordinates = 0 语义一致）。
2. **part06.js 泛型 click 分支**：clientX/clientY own-property 注入（trusted
   mouse 坐标同步 floor——part05 `_zwMouseCoordInit` 同口径，手动 MouseEvent
   面经 ctor floor，两通道读数恒等）+ image submit 按钮 selected coordinates
   记账臂（内联同口径：client 坐标 − 盒原点，round + 负值钳 0，键 = `_elKey`；
   只记账不触发提交——激活/提交语义留宿主 plan outcome + requestSubmit 兜底
   防双导航）。offsetX 注入（R3999）取值口径本就是「client 坐标（未注入缺省
   0）− 盒原点」，本刀使其读到真实坐标，host click 的 offsetX/offsetY 同步
   转真（dialog-form-submission t5 断言面）。

定向：`make testharness-net-api FILTER=xhr/formdata` 族内 87P/2F（F1/F4 双
转绿，F2/F3 挂账项保留）；submitter-coordinate-value 双 prong 断言
（testdriver vs 手动 dispatchEvent 中心坐标一致）转绿证明通道口径齐。

## 四、F2 挂账（重分类：引擎 DOMString 语义桥欠账）

`input.name = "submit-me-18-\uDC01"`（unpaired surrogate——DOMString = UTF-16
code unit 序列，lone surrogate 合法）读回 expected `submit-me-18-U+dc01` got
`submit-me-18-�`。引擎属性/IDL 字符串通道为 UTF-8，lone surrogate 无法
表示 → lossy 成 U+FFFD。修复需 surrogate 保真桥（WTF-8 或 code-unit 对 escapes）
——engine↔host 全桥面（attributes/value/name 等所有字符串读写），跨子系统
改造，非 net-api 一批范围。**移交引擎字符串语义欠账**（与 s82 时代 plain 世界
字符串面同源债），不在本 goal 门内。

## 五、F3 挂账（重分类：渲染流域图像固有尺寸）

`expected "23" got "0"`——测试按 `getBoundingClientRect()` 算 click 点
（`(left+right)/7`），期望值 23/46 锚定浏览器 200×200 布局（rect (8,…,208)
→ cx=31 → x=23）。runner 布局把 `100px-green-rect.svg`（实际 `width="200"`)
渲染 29px 宽（probe rect=8,15.848,37,23.848）→ cx=6 < rect.left=8 → 钳 0。
坐标管线本刀后已正确（探针 [0,0] 为 rect 正确钳零；submitter-coordinate-value
转绿互证），剩余是**图像加载 → 固有尺寸 → 布局盒**链——render-foundation/
layout-engine 流域（工作面不重叠，run-rules §9），移交渲染流。

## 六、验证

- 定向：FILTER=xhr/formdata 族 87P/2F（基线 90 案中 4F → 2F）。
- 全量：`make testharness-net-api`（test-guard 包裹，--per-proc-mem 4
  --total-mem 8 --time-limit 900）对 M3（六）基线逐 subtest A/B：
  **分母 14,621 键不变，changed 2 全 F→P，零 P→F，零 T 变化，added/removed 0**。
- clippy `-p zero-webview`：warning 计数与基线持平（7 = 7，零新增）。
- 13,253P/1,424F/15T → **13,255P**/1,422F/15T。
- 日志：基线 `/tmp/netapi-m3-om-family-full-cur.log`；当前
  `/tmp/netapi-m3-fd-family-full-cur.log`；对照脚本 `/tmp/m3-auth-ab.py`。

## 七、挂账

M3 余项收窄：**xhr 在账失败簇 0F**（formdata/submitter 簇余 2F 已重分类
移交他域——F2 引擎字符串语义桥 / F3 渲染流图像尺寸，不计入本 goal DC-3 门）
+ cookies.http.html 1T + redirect-keepalive 1T → M4 coverage 复测收口（DC-4）。

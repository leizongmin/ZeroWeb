# M3 S3 SVG script 外部脚本加载——动态插入面 load 事件语义（2026-10-21）

M3 S2（evidence/2026-10-21-m3-s2-href-reflection-landing.md）后 M3 第三切片。
S2 落地后 href-script-element 3 子转文件 Timeout（首断言 href.baseVal 过、
挂 `waitEvent(script, 'load')`）——本切片回收该文件，落运行期（动态插入）
SVG script 外部脚本加载语义。初文档标记面的加载已在提取管线 R5005
（html-syntax-compat M4 残差重归因，markup 文件已随 S2 连带翻绿），不属本
切片。

## 实现（spec SVG2 interact.html#ScriptElementProcessing + HTML
#prepare-the-script-element SVG 裁剪面）

- **触发点**（part04.js set trap 两方法分支尾部，同 iframe reload 钩子
  形态）：`setAttribute` 写 href / xlink:href / type 到 SVG script 元素、
  `setAttributeNS` 写 local href / type。helper 快门先判 SVG ns + SCRIPT
  tag（HTML input type= 等高频写入零成本返回，不排定时器）。
- **per-element 单次延迟「maybe start load」检查**：触发只置 pending 旗标
  + setTimeout(0)，检查在 fire 时做——同一同步块内 append→type→href 连续
  变异合并为一次判定（createSVGElement 先 append 后设属性的测试形态）；
  pending 去重防同块多次触发排多定时器。
- **type 可执行判定**：缺省/空 = 可执行（HTML spec）；非空按 JS MIME
  essence 面（javascript/ecmascript 子串，镜像提取管线
  `script_type_is_javascript`）；text/plain 等 suppress。
- **URL 解析**：href 属性优先、legacy xlink:href 回落（S2 href 反射同优先
  级）、都无或空串不加载。
- **取源**：host `__zw_fetch_script`（同 Worker importScripts 通道；runner
  侧 wpt_data_script_fetcher 相对页 URL 解析 + MIME 门既有）。
- **执行**：间接 eval `(0, eval)(code)`——经典脚本顶层函数声明必须落全局
  （new Function 体声明是局部的，externalScript1.js 的
  `function loadedScript(){}` 须成 global）。
- **load 事件**：执行成功置 per-element loaded 旗标后派发 load
  （bubbles:false——不冒泡，waitEvent 以元素自身 listener 消费）；
  **已 load 后再改 href/xlink:href 不重载**（WPT 三子定谳的核心语义）；
  取源失败派 error（不置旗标，重试留待后续触发）。

## 设计取舍

- **不钩 appendChild/insertBefore**：attrs-dict 形态
  （createSVGElement(t,'script',svg,{href:…})）的属性经 setAttribute 触发
  后延迟检查兜住（fire 时已 append 完成）；纯 insertBefore/replaceChild 带
  属插入未覆盖（FIXME 记档于 helper 注记）。
- **in-document 连接态门未做**（FIXME）：spec prepare 步骤要求元素已连接，
  WPT 面均先 append 后设属性——detached 设 href 即加载的多余面后续补。
- **setTimeout(0) 而非 microtask `_defer`**：外部取源语义是 task 级；且
  `_deferBudget`（256/脚本执行批）耗尽会静默 no-op，加载通道不宜耦合该
  预算。

## 验证

- **靶面 href-script-element 3/3 + href-script-element-markup 1/1 全绿**
  （r1 首轮即全绿；上轮 S2 定稿时该文件为 Timeout）。
- **全 corpus 对账 r2**（基线 = M3 S2 收口 1224P/487F/42T/1753 = 69.8%）：
  - 原始全量 **1238P/489F/40T/1767 = 70.1%**（+14P/+2F/−2T；分母 +14 =
    −2 文件级 T 行（href-script、script-load-error-events 均解析出子测试）
    + 16 新子（href-script 3 + script-load-error-events 5 + 探针 8））。
  - 转移矩阵（共同行）：1224 Pass→Pass（**零 Pass 回归**、零 DOWN）、
    487 Fail→Fail、40 Timeout→Timeout——共同行零状态变化，全部移动来自
    上述两个文件从整文件 Timeout 转子测试解析 + 探针新子。
  - 探针剔除同分母（1698→1704）：**1169P/487F/42T = 68.8% →
    1175P/489F/40T = 69.0%**（+6P = 靶面 3 + script-load-error-events 3；
    +2F/+1T 净减见下）。
  - **连带解锁**：svg/interact/script-load-error-events.html 整文件
    Timeout → 5 子解析 3P/2F——load/error 事件落地后其等待不再悬挂。
    3P = data: URL 源 1（`data:text/javascript,# comment` bare `#` 起
    fragment 不入 body，空脚本编译成功照常 load）+ non-existing 2（取源
    失败派 error 的正确语义）；2F = `./resources/load-error-events.py`
    动态端点（runner 无 wptserve 动态资源面，**挂账 runner/infra 域**，
    非 shim 面）。
  - 修出一轮：r1 后首版把 host 通道取源失败（runner 通道 Err 与 miss 均
    `unwrap_or_default` 成 `''`，`|| null` 通道无法区分空体与失败）当
    「空脚本成功」——non-existing 两案期望 error 反得 load（探针 P6 同步
    暴露）。修正为通道二分：data: shim 侧直解空体合法（load）、host 通道
    null/'' 一律派 error——与 HTML spec「fetch 失败派 error、空脚本
    compile 成功派 load」一致。
- **探针 m3s3-probe.html 8/8 Pass**（P1 触发+load+全局声明落地、P5 非 JS
  type suppress、P2/P3/P4 三向不重载 + href 优先级 + removeAttribute 不
  重载、P7 IDL href.baseVal 写回触发、P6 取源失败派 error、P8 data: 源
  fragment 剥离空体 load）。
- DC-4：`cargo fmt --all --check` EXIT=0；定向 `cargo test -p zero-engine`
  （test-guard --compile-first 4G/8G/1200s）**3377P/0F，EXIT=0**。

## 划界

- **`.py` 动态端点 2F**（script-load-error-events existing ×2）：runner 取
  源通道无 wptserve 动态资源执行——runner/infra 域挂账，shim 侧已按正确
  语义派 error（期望 load 故 Fail）。
- module 型 type 不走经典 eval（SMIL/M4 面，FIXME 记档）。
- strict payload 的 script 级 strict 语义（间接 eval 的 eval 级 strict 会
  scope 局部）——本仓 WPT payload 均 sloppy，挂账。
- 编译失败与运行时抛错未区分（间接 eval 同栈，统一派元素 error；HTML spec
  运行时异常归全局 onerror 面）——FIXME 记档，WPT 面 payload 无运行时
  抛错。
- error 后重试语义（spec fetch 失败可因属性再变重试）仅保留「不置旗标」
  最小面。

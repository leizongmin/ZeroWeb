# M2 s141 · targetted-form 族勘根轮——空串 formaction 收口 2 键 + apply 滞后一拍统一根因定谳（挂账三立刀）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s140（File 条目层）

## 入场定刀位

控制面 s141 候选①：targetted-form 族 58F 深层面勘根（double-submit
防护族为主）。本轮为**勘根轮**：探针 14 跑钉死三层根因结构，施工
仅收确定性小修（空串 formaction），主干根因挂账独立刀。

## 勘根（探针钉死，14 跑证据链）

探针页 zz-probe-prevsib.html / zz-probe-ds.html（验后删），
FILTER 定向 14 轮。三层根因：

1. **兄弟导航 apply 代际缺口**（探针 1-8）：insertAdjacentHTML
   注入的 iframe+form——静态读 `form.previousSibling` 正常
   （R3254-K3 pending 臂），但**提交触发 apply 代际边界后**
   （`__zw_apply_generation_bump` 定点清除 pending 解析节点——
   设计前提「host apply 后节点已进 host 快照」成立，但 **JS 侧
   proxy 持有的 sel 随换代失效**）落 live 臂：`__zw_sibling_nodes`
   / `_childNodeList` 按 stale sel 查询恒 miss → null。语料
   `let iframe = form.previousSibling; await loadPromise(iframe)`
   在提交后读 → `Cannot set properties of null (setting
   'onload')`——波及 form-submission-algorithm 8F + requestsubmit
   onload 形态案 ~20 键。探针同时钉死：文档未导航、iframe
   named 查询/innerHTML 序列化/children 枚举三通道都在、form
   proxy 身份保留（`formFresh === form`）——缺口精确在 sel 换代
   失效，非树结构损坏。previousElementSibling 在 pending 窗口
   内也恒 null（live 通道盲区旁证）。
2. **apply 滞后一拍 / runner 同文件多跑前遍挂**（探针 14 + 单
   文件跑）：form-submit-action.html 单文件 runner 执行 **3 遍**
   （30 行 = 10 案 × 3），遍 1/2 全挂 `getElementById(...).click
   is not a function`、遍 3 才 8P/2F；form-double-submit-
   requestsubmit.html 同形态（遍 1/2 挂 v1/v2 断言、遍 3 过）。
   **语料本体减半探针（单案冷启动）行为完全正确**（formdata 全
   序 dump：`click-begin; FD n1=v1 n2=v3; after-click fd.n1=v1`
   ——entry 构造在 submit 事件后、A 的 IDL set 落 attr（value
   mode=default）✓、B 的 preventDefault 拦住第二次提交 ✓）——
   原版失败全来自**案间/遍间状态**。统一根因假说：注入节点的
   host 物化滞后一拍（跨 script 执行边界 pending→apply→稳定），
   提交管线 host 通道（gEBI 物化/target 解析）对 pending 失明；
   遍 1 冷、遍 2 半热、遍 3 稳定的三态与两拍模型吻合。
3. **空串 formaction 回落误判**（s141 施工收口）：④ action 解析
   `if (_actionRaw == null || _actionRaw === '')` 把**空串在场
   值**并入回落分支 → 回落 form action 属性。spec #concept-fs-
   action：submitter 有 formaction 属性（含空串）即以其为
   action，空串 resolve 即 base = 文档地址。语料 form-submit-
   action「no formaction, expect to navigate to document URL」
   （submitter formaction=""）expected 文档地址 got a.html——
   button/input 各 1 键。

## 施工（shim 1 文件 1 处）

- part04 `_zwFormSubmitNavigate` ④ action 解析：回落判定改只按
  `== null`（属性缺席），空串保留交 `new URL('', base)` 天然落
  文档地址；注释引 spec #concept-fs-action + 语料案名。 FIXME
  清单行「submitter formaction=''」标记 s141 已修。

## 验证

- shim 13-part 拼接 node --check 过。
- 定向 form-submission-0 全目录（对 s140b 定向基线，(key,state)
  对级 sort -u 后 comm）：**恰 +2 F→P 零其他位移**（button/input
  两案 `no formaction...document URL`），键集合 293→295 对精确
  闭合、零 losses；行数 305→325 差 20 = form-submit-action 多跑
  遍数波动（对账以 pair 为准，多跑机制已挂账）。
- 全量 join（对 s140 full）见控制面状态块数字；engine lib
  **3582/8F**（HEAD 基线在册环境面，s140 stash 对照已定谳非本
  流引入）；runner bin **256/1F**（trend_csv_check encoding 行
  归口不变）；clippy（zero-engine + zero-wpt-runner --all-targets
  -D warnings）零告警；fmt runner 过；语料零改动（探针验后删）；
  无探针页残留。

## 挂账（本切片更新，targetted-form 族三立刀）

- **刀 A（apply 滞后一拍 / sel 换代失效）**：js-dom 节点视图
  基建——`__zw_apply_generation_bump` 清 pending 解析节点时
  「host 已进快照」前提对 sel 消费方不成立（stale sel）→ live
  臂 miss。候选修法：apply 清除时保留旧 sel→新位置签名映射，或
  live 臂 stale sel 回退按父 live 枚举 + 属性签名重定位（注意
  无 name form 的签名歧义——减半探针实证多案同签名不可行，须
  树位置/反链方案）。波及：兄弟导航（previousSibling/
  previousElementSibling）、gEBI 物化 wrapper、提交管线 host
  通道——预计解锁 form-submission-algorithm 8F + requestsubmit
  onload 案 ~20 键 + form-submit-action click 形态 20 键（若
  「滞后一拍」是 click not function 根因——待刀 A 印证）。
- **刀 B（runner 同文件多跑 + 首遍挂执行模型）**：form-submit-
  action 单文件 3 遍、form-double-submit-requestsubmit 同形态；
  多跑机制本身（枚举重复 or 调度重试）+ 首遍冷态行为与遍 3 不
  一致——执行模型面勘根（与刀 A 交叉：先刀 A 后刀 B 印证残留
  面）。对账纪律：多跑下 (key,state) 对级仍闭合，但行数/计数
  波动须以 pair 为准。
- **刀 C（双提交语义残余）**：减半探针证明 pdRS=F/pdSB=T 单案
  语义正确（B 拦第二次）；原版矩阵遍 1/2 的 v1 got v2 归刀 A/B
  状态面。遍 3 全过后若仍有残余案（default-action 4F /
  preventdefault 4F / input-type-change 1F / request-submit-
  activation 1F）再立刀细勘。
- 承袭不变：accept-charset w1252 + 实体回退 + lone surrogate
  USV（候选②，18 案）；engine lib js_dom_bridge 桥接系列 8F
  环境性失败面（s140 挂）；「override builtins」1F 待决；
  encoding CSV 归口；fmt url_idna 归口；echo 回显体观察；其余
  承袭清单不变；**待决（须用户拍板）**：css/text 语料资源、
  green-2x2.png、parser form pointer 缓行（2026-10-23）。

## 过程教训

- **勘根先用语料本体减半，再用自写探针**：自写探针（探针 5）
  复刻不出失败（双提交语义单案正确），语料本体减半（zz-probe-ds
  = 原文件砍矩阵留单案 + dump）一轮拿到「单案对、原版挂」的
  决定性对照——失败在案间/遍间状态，不在语义。自写复刻会无意
  中丢掉触发状态的细节（本案：多案矩阵 + runner 多遍）。
- **同文件多跑的日志形态要先进对账框架再解读**：22F 里 20 案
  click not function + 2 案 formaction 断言，直接读会当成两类
  各自的根因；单文件跑暴露 3 遍结构后，20 案实为「遍 1/2 全挂
  ×2 + 遍 3 过」的一个状态面。计数解读前先看执行遍次结构。
- **探针页要躲 runner 静态 skip**：`source.contains("<iframe")`
  排除规则会静默跳过含字面的探针页（零输出零报错）——探针用
  运行期拼接构造；探针注释里也不能写字面（第一次踩：注释含
  `<iframe` 又被 skip，日志零输出的排查成本高于写探针本身）。

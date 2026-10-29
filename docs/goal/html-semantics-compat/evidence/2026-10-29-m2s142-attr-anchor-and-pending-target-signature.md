# M2 s142 · name/target 属性锚 + pending 签名 +target——targetted-form 族 21 键翻红 + 1 键揭旧疾（片段双重物化）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s141（formaction 空串 + apply 滞后勘根）

## 入场定刀位

s141 挂账刀 A（apply 滞后一拍 / sel 换代失效）候选①。本轮施工两处：
host 侧选择器总漏斗加属性锚 + shim pending 签名扩 target——
把「无 id/class/name 的 form / 有 name 的 iframe」从**位置锚**
（`:nth-child` 绝对位置，afterbegin 连环插入即漂移）换成
**属性身份锚**（位置漂移免疫）。

## 施工（2 文件）

- **Fix 1** `crates/engine/src/js_dom_bridge.rs`
  `unique_selector_for_node`：`#id` → `tag.class` → 裸 tag 均不唯一
  后、结构路径前，插入属性锚臂——`name` / `target` 属性值域限
  `[A-Za-z0-9_-]`（无转义面），`doc.query_selector_all` 验唯一才
  采用。iframe（`iframe[name=form-test-target-N]`）与 form
  （`form[target=form-test-target-N]`）就此获得稳定身份；无
  name/target 的歧义节点零额外宿主查询，行为同旧。
- **Fix 2** `crates/engine/src/js_dom_shim/part03.js`
  `_zwPendingParsedForSel` 签名 `tag|id|name|class` →
  `tag|id|name|class|target`（节点侧 `attrs.target`、sel 侧
  `__zw_get_attr_lw(sel,'target')`）。targeted-form 语料族
  populateForm 的 form 无 id/class/name 全同签 `FORM|||`，
  多 form pending 时 hits>1 不归一 → 落 pending 失明的 live 臂 →
  `form.previousSibling` 崩 null（s141 勘根主根因之一）；target
  与 host 侧属性锚同源（frameName 每次唯一），签名歧义消解。

## 验证

- shim 13-part 拼接（part01,01b,02..07,08a,08,08c,08b,08d）
  node --check 过（109157 行）。
- 定向 A/B（s142 会话内 stash 对照，form-submission-0 全目录）：
  fixed 287 对 vs base 295 对；gains 36P+3F，losses 48F+1P——
  losses 全为 crash→hang 递进（先前快速崩 Fail 的案推进到更深
  层 hang 在 loadPromise → 文件级 Timeout 隐藏键）+ 多跑波动。
- **全量 join**（/tmp/zw-s142-full-base3.log 真基线 vs fixed，
  (key,state) 对级 sort -u + key 级 state-set 分类）：
  - base 8001 行 = **7000P/904F/97T**（与 s141 控制面在册完全
    吻合，确认基线无误）；fixed 7994 行 = 7037P/858F/99T。
  - **21 键 Fail→Pass**：form-double-submit-default-action 4、
    form-double-submit-requestsubmit runTest 族 7、form-double-
    submit-preventdefault-click 2、input-type-change-submit 1、
    request-submit-activation 1、form-submission-algorithm 5
    （firing submit click/image-button/requestSubmit/requestSubmit(null)/
    firing-submission-events-reentrant）、implicit-submission 邻域外
    全部集中在 form-submission-0 targetted-form 族。
  - **17 键新上报 Pass**（base 侧因早期崩溃未上报）+ 2 新上报
    Fail + 1 Timeout（多跑/深推进波动）。
  - **2 键 Fail→Timeout** + **26 键 base-only**（26F+1T，先前已在
    失败，fixed 侧因文件级 hang 不再上报）——crash→hang 递进
    记账，非回归。
  - **1 键 Pass→Fail**：implicit-submission.optional「Submit event
    with a submit button」——见下节勘根，旧疾揭露非本修引入。
- engine lib（rebase 后合并树）**3588/9F**，失败名单与 stash 基线
  **逐字节相同**（3581/9F 同名单）——9F 全环境面归口；s141 在册
  8F → 本轮 9F 为轮间环境漂移（两侧同现，非本流引入）。
- runner bin **256/1F**（trend_csv_check encoding 行归口不变）。
- clippy（zero-engine + zero-wpt-runner --all-targets -D warnings）
  零告警；fmt 本切片改动区域零 diff；语料零改动（探针验后删）。

## s142b 探针勘根（implicit-submission 1 键 Pass→Fail 定谳）

探针 zz-probe-s142b（v1 结构 dump / v2 requestSubmit / v3 真实
send_keys 流 / v4 时线，验后删），FILTER 定向 5 跑 + stash 对照 1 跑：

1. populateForm 单独调用后 form.submitButton 单元素（v1），requestSubmit
   后亦单（v2）——纯结构无双插。
2. **v3 真实流（send_keys 隐式提交）复现**：提交导航后
   `body.innerHTML` 出现**两份逐字相同的 (iframe+form) 片段**
   （同名同 target 属性），`form.submitButton` 经
   `form[target=...]` 作用域返回 2 元素集合；
   `event.submitter` 为正确单元素（本修使其语义正确）。
3. **stash 对照（基线）同样双份**——片段双重物化为**既有引擎
   旧疾**，非本修引入。基线过测是双向一致错误（基线命名作用域
   按结构路径钉单位置 → 单元素 = submitter 单元素 → 相等）；本修
   的属性锚在复制树上失唯一（逐字副本共享全部属性，任何属性
   方案不可分辨，位置方案则漂移别名）→ 集合 2 元素 → 与正确
   单元素 submitter 不等 → 翻红。
4. v4 时线：`querySelectorAll('form')` 在 populateForm 后瞬时
   双计（pending 窗口 JS 视图双视，apply bump 后归一）；v3 的
   双份为提交导航后**真实再物化**（frame 导航路径触发，独立于
   pending 窗口）。本轮 in-submit 快照偶发缺失为 send_keys 隐式
   提交触发不稳定旁证（v3 两侧均正常触发）。

**结论**：该 1 键的正确修法是消灭片段双重物化（frame 导航 /
apply 基建，刀 A 同族主根），任何选择器方案都只是症状面。
记账：净真增益 +21 翻红 +17 新上报，−1 揭旧疾。

## 挂账（本切片更新）

- **刀 A′（片段双重物化，原刀 A 升格）**：提交导航触发
  (iframe+form) pending 解析片段在 body 真实再物化一份（v3 探针
  实证，基线同现）。修复后预期：implicit-submission 1 键回归
  Pass + crash→hang 递进面（2 键 Fail→Timeout + 26 键 unreported，
  form-double-submit-requestsubmit runTest2 re-entrant hang /
  form-submission-algorithm 第 9 子测 requestSubmit(submitter)
  hang）的 sel 消费方失配面同步收敛。勘根入口：`_zwFormSubmitNavigate`
  目标解析 → frame 导航物化路径 → `__zw_apply_generation_bump`
  清除时序。
- **刀 C（re-entrant submission hang，从刀 A 分立）**：剩余 hang
  全为 submit handler 内 re-entrant requestSubmit/submit 形态
  （runTest2 矩阵 + form-submission-algorithm 第 9 子测），即使
  sel 面修复后仍需独立语义收口（firing submission events 重入
  防护 spec 步骤）。
- 承袭不变：刀 B（runner 同文件多跑 + 首遍挂执行模型）；accept-
  charset w1252 + 实体回退 + lone surrogate USV（候选②，18 案）；
  engine lib js_dom_bridge 桥接系列 9F 环境面（本轮重验逐字节同
  名单）；「override builtins」1F 待决；encoding CSV 归口；fmt
  url_idna 归口 + **新增**：rebase 后 4f2230a6d（url-setter 落地）
  区域 9 处 fmt diff 与 4 处 clippy -D warnings 阻断（本轮已顺手
  修 4 处 lint，fmt 未顺手重排归口该流）；echo 回显体观察；待决
  （须用户拍板）：css/text 语料资源、green-2x2.png、parser form
  pointer 缓行（2026-10-23）。

## 过程教训

- **key 级 state-set 分类是对账的正确粒度**：对级 losses（51）/
  gains（44）粗看吓人，key 级分类后收敛为 21 翻红 + 17 新上报
  + 2 递进 + 26 unreported + 1 揭旧疾——多跑与 crash→hang 把
  对级数字放大近 3 倍，直接读对级 diff 必误判。
- **join 后必须查「单侧独有键」**：文件级 Timeout 会把后续键整段
  隐藏（26 键 base-only），join 默认只输出两侧共有键——漏查会把
  「已在失败的键不再上报」误读成修复面。
- **rebase 撞并行流后门禁必须重跑**：pop 自动合并成功 ≠ 门禁仍
  有效——本轮 merge 后 clippy 暴露上游落地即红（4 处 lint）+ fmt
  9 处漂移；「我区域干净」不能代替树级绿灯，lint 机械修复就地
  收口并归口记录，fmt 不顺手重排他人区域。
- **stash A/B 的时序要为编译让路**：stash→launch→pop 间隔过短
  时，cargo 可能在 pop 后才读源，「基线」实为修复版。判据：pop
  后带修复源重编译是否触发（本轮 `--no-run` 触发 `Compiling
  zero-engine` =  stash 轮二进制确为基线内容，A/B 有效）。

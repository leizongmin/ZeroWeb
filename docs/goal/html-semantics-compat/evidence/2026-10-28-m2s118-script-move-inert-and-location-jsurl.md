# M2 s118 · innerHTML 产物 script 移动惰性章 + location 族 javascript: URL 求值面

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s117（连接态 sel 形 SCRIPT src 后设 prepare 触发面）

## 刀位与判据链

execution-timing 余 F 复盘定刀（承 s117 下一步候选一），三面并审：

- **033**「innerHTML and scripts moved in DOM」1F：`div.innerHTML` 写入 inline+外链
  script → 逐个 `appendChild(d.firstChild)` 移入 body → spec prepare the script
  element「fragment parsing algorithm 创建的 script 元素置 already started」——移动
  不得执行；本架构两脚本全执行（日志 5 项 vs 期望 3 项）。
- **028/079**「location javascript: URL」两页 2F：`location.replace('javascript:…')`
  （028，assert_any 双序）与 onload 内 `window.location='javascript:…'`（079，严格序
  「onload ends → javascript: URL → timeout」）——体从未求值。`_navAnchorNavigate`
  的 M2-S4U 面已有 javascript: defer-eval 先例（间接 eval + `_defer` 微任务），
  location 三入口（`_locationReplace`/`_locationAssign`/`_setLocationPart` href 臂，
  后者承 `location.href=` 与 `window.location=` 赋值形）全部缺同一拦截。
- 排除 041（nested write 插入序——append 模型 vs 解析位语义，须写流队列模型重构，
  承袭挂账不作常规刀位）；排除 031（focus/blur 事件不派发——共享焦点事件面，
  单列后续刀位）。

## 施工（净变更四文件）

1. **part03**：`_zwScriptMaybeStartDynamic` 顶部加 fragment 产物 already-started 章
   门（`el.__zwFragNoExec` 属性判定，s117 证 sel 盖章路线不可行后改对象属性键）。
2. **part04 innerHTML setter**（sel/handle 双路径共用的 `_ihAdded` 面）：解析产物
   深走打章（SCRIPT 置 `__zwFragNoExec`）；**R368 盖 handle 章点**章转移
   run-once 表（`_zwRanScripts[handle]=true`）。
3. **part02**：共用 helper `_navLocationJavascript`（前导空白 strip + scheme 前缀
   大小写不敏感 → `_defer` 间接 eval 落全局 → true=已拦截；结果字符串导航体面未
   建模——S4U 同款近似，语料体均 undefined）+ 三入口门（assign/replace 顶部、
   `_setLocationPart` href 臂；javascript: 不派 navigate 事件与 S4U 同源）。
4. **testharness.rs**：bin pin ×3（innerHTML 移动惰性 / location= 严格序 /
   location.replace assert_any 双序；sel 形容器——createElement 容器走 handle 形
   物化路径不在本面）。

## 首刀教训（pin 红定谳）

marker 初版用对象级 Set：插入臂收到的是 **R368 身份盖章后的 handle 代理**
（`_zwMEl` 解析代理 append 时无身份 → 盖 `__zwHandle` → notify 链换 proxy），
Set 键身份断链、两脚本照执行（pin 首跑日志实证：inline 产物在 appendChild 同步
栈内执行）。改对象属性章 + R368 转移 run-once 表后 3/3 绿——章键必须落在移动
换代后的可寻址身份（handle）上，与 s117 的「章键落执行时身份」结论同构。

## 验证

- bin pin ×3：新 shim **3/3 绿**；旧 shim stash 往返 **3/3 红**（行为失败：
  location 两页体缺席、innerHTML 页产物执行——非编译失败）。
- 定向 `FILTER=execution-timing/0`（69 行面）：**028/033/079 三页 F→P**；全行
  归一 diff 唯一变化 = 此三页（031/041/056/063/068 维持原状），69P+8F+1T→72P+5F+1T。
- **全量 A/B**（/tmp/s117-full-suite.log → /tmp/s118-full-suite.log，全行归一
  diff）：**6607P+1191F+105T → 6610P+1188F+105T（+3P −3F ±0T）**；diff 唯一
  变化 = 028/033/079 三页 F→P，其余状态行零变化 → **真翻红 = 0**（forms
  jsurl-navigation-then-form-submit 等 javascript: 相邻页无翻转——location 三
  入口门与 form submit 路径无交集）。
- engine lib **3570/0**；guard 内 clippy（zero-wpt-runner --all-targets +
  zero-engine）零告警；fmt 过；shim 拼接语法 OK（真实装配序 part01+01b+02..07，
  `node --check`）。

## 挂账（本切片新增/更新）

- **031 focus/blur 事件不派发**（复盘明确化）：焦点事件面在焦点基建，非脚本
  触发链；单列后续刀位。
- **041 nested write 插入序**（承袭维持）：append 模型 vs 解析位语义，须写流
  队列模型重构（大基建，不作常规刀位）。
- **FIXME(spec-gap) iah/clone fragment 章**：insertAdjacentHTML/cloneNode 重解析
  产物同属 fragment parsing already-started 面，暂不打标，待语料需求补章
  （033 需求面只覆盖 innerHTML setter）。
- **html/browsers javascript-url 族页**（security-check/task-queuing 等）在
  html-semantics 域外——location 三入口门的域外回归面无本域 A/B 覆盖，S4U
  先例（navigate-to-javascript 已绿）为语义背书；后续跑 browsers 域时复核。
- 承袭：写流执行身份盖章断链 FIXME、part02 mut-tree setAttribute 绕 notify 臂
  （gap 2）、execution-timing 构成面 056/063+051/052 单根（增量解析大基建）、
  css-module 族 corpus 门（待决须用户拍板）、www2 可达性、modulepreload
  referrerpolicy 族、module-import-referrer TLA、script-crossorigin-network
  cookies、redirect-relative-urls、export-from with FIXME、script-charset-02/03、
  143 write、defer 多写完成序、dynamic import() with 透传、classic 外链 SRI、
  接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text
  数据模块语料资源缺失、parser form element pointer 4F 缓行（2026-10-23）。

## 遗留近似

- location javascript: 求值的**字符串结果导航体面**未建模（spec：结果 DOMString
  → 以结果导航；S4U 同款近似，runner 单文档通道，语料体均 undefined 返回）。
- `location.assign('javascript:…')` 门与 replace/href= 共享 helper，无独立 pin
  （语料无 assign 形 javascript: 页）。
- fragment 章的**写流回放基线**同样打章（回放基线 innerHTML= 经同一 setter）——
  回放执行走 part06 水印执行器不经 helper，行为不变；换代重注册 sel 形代理是
  新建对象，026 面 src 后设不受影响（全量 A/B 026 维持 P 实证）。

# M2 s154 · adopt/innerHTML 身份双表示——stylesheet 触发臂两缺口收口
## （宿主 absent 属性 '' 回退 + 身份键单发重查/取回印记）

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s153（跨源 stylesheet CORS 豁免）

## 入场定刀位

s153 下一步候选①：document-without-browsing-context 2F（Create a document,
adopt the node + Create a stylesheet in innerHTML document——adopted/innerHTML
物化的 link 未触发 stylesheet 取回，count 端点 expected "1" got "0"）。

## 勘根（探针五轮 + shim breadcrumbs 两轮收敛）

runner stylesheet.py 端点（s150）：资源请求（无 count 参数）take+put(count+1)
返 CSS 体；count 请求 take 不回填。got "0" = 该 id 的资源请求从未到达端点。

1. **探针 v1-v2（wrap 法——包 globalThis._zwStylesheetMaybeStart 与 fetch）**：
   两案形态复现（createElement 控制组 Pass）。MO 挂 head/div 零记录 + 臂调用
   日志实证：createElement 形 notify 汇流臂正常触发取回；**adopt 形 4 参臂已带
   章（h=__n1）但不 fetch；innerHTML 形两臂触发但均不 fetch**（attr 臂 3 参
   h=__n2 + 汇流臂 4 参 h=null plain 形）。
2. **shim breadcrumbs（探针页 URL 旗标门控）**：adopt 形门链
   `tag=LINK → conn=true → attr(rel): h="" el=<same> → BAIL:rel`——宿主 handle
   读对未回填属性返 **''（非 null）**，`_ssReadAttr` 的 `== null` 回退不触发，
   elObj（DOMParser M 节点）明有 'stylesheet' 而被跳过。
3. **案 1（adopt）根因**：R368 盖章路径的 R4004 属性回填依赖
   `child.getAttributeNames`——DOMParser 解析树 M 节点无该方法（探针实证
   `getAttributeNames=undefined THROW`）→ 宿主属性表恒空 → handle 读 '' →
   `== null` 判不中 → 回退失效 → rel='' 分词 miss → bail。
4. **案 2（innerHTML）根因 = 身份双表示时序缺口**：innerHTML setter 物化是
   **身份分裂**的——mutation 记账时 addedNodes 还是解析树 plain M 节点（汇流
   臂 4 参 h=null → TAG 门 `_realTag(null,null)` 恒 DIV、豁免需 handle →
   bail）；物化产物是**另一 JS 对象**（盖章 handle __n2，原 M 节点永不盖章
   ——重查 elObj 现读恒 null 旁证 + probe v1 持引用 parent=null 旁证）。可
   恢复面是物化 attr 臂（3 参 h=__n2 已章）：`_zwNodeParent` 反链注册滞后于
   属性写 → 连接门 bail。两臂当时态都判 not connected。
5. **修复迭代（v1 失败教训）**：重查首版从 elObj 现读身份——身份分裂使
   innerHTML 案的 elObj 恒无章 → 重查自弃（dbg 无 enter 实证）。改 **身份键**
   （`_elKey(sel,handle)`）设计：重查按触发臂给定的 (sel,handle) 原样重跑
   （attr 臂 3 参身份已正确）；取回印记/重查印记同键表驱动（elObj 节点属性
   印记覆盖不到 3 参调用，身份键对两臂一致）。

## 施工（part03 helper 净 +59/−4）

1. **`_ssReadAttr` 回退判据放宽**：`== null` → falsy（宿主 absent '' 与解析
   树 null 同判 miss；rel/href 均非空串语义，空值本就该 miss）。
2. **身份键取回印记表 `_zwSsFiredMap`**：同 (sel,handle) 同解析 URL 只发起
   一次（发起前置位）。保住既有「先赋 href（detached bail → 重查排程）后
   append（插入臂直取）」形防双请求；href 变更解析 URL 不同印记失配照常重
   取；无身份键（静态扫描空槽形）不设印记照旧直取。
3. **单发身份键重查 `_ssScheduleRetry`**：TAG/连接门 bail 且身份键存在时
   setTimeout(0) 按原身份重跑一次（身份缺失回落 elObj 现读——R368 盖章形
   append 臂 elObj 后到章）；`_zwSsRetryDone` 单发印记防同身份多臂重复排程
   与 detached 恒态循环。

## 过程事故（勘根教训）

debug breadcrumbs 残留行引用已删 `var _ssDbg` → 读未声明标识符 ReferenceError
→ helper 外层 catch 静默吞 → 修复版首跑假阴（定向仍 2F）。二轮 breadcrumbs
复跑定位。教训：门链插桩移除必须与声明同批全清，`grep 标识符` 收尾核验（本
轮收尾 grep `_ssDbg|__zwSsDbg` 零残留）。另：make 后台跑 cwd 不保持，须
`sh -c 'cd <repo-root> && make ...'` 显式锚定（两度「没有规则可制作目标」）。

## 验证

- **语法**：shim 规范序拼接 node --check SYNTAX_OK 110325 行（s153 110270，
  净 +55）。Rust 面零改动（clippy/fmt 零波及，未跑）。
- **定向 the-link-element 全目录（113 键对 s153 全量基线 join）**：靶面 2 案
  F→P、**LOST 0、GAINED 0、其他位移 0**（键 113=113 精确配平）。
- **全量案级 join（对 s153 /tmp/zw-s153-full.log，(key,state) 对 defaultdict
  (set) 对比）**：8080 对 = 8080 对（7193P/799F/88T → 7195P/797F/88T，ΔP=+2
  ΔF=−2 ΔT=0）——**F2P 2（靶面）+ P2F 0 + other-diff 0 + LOST 0 + GAINED 0**
  全域零位移。重查/印记对 fetch API/XHR 面、link-multiple（先赋后插形）、
  s152 静态扫描面零波及实证。
- 探针语料（zz-s154-probe.html / zz-s154-dbg.html）验后已删（工作树零残留）。

## 挂账（本切片更新）

- **收口：adopt/innerHTML 身份双表示触发通道面**——document-without-
  browsing-context 2F 全收口。遗留边角新记：sel 重解析同槽同 href 复用（
  `_zwSsFiredMap` 身份键跨代际沿用——同 sel 新元素同 href 不重取；语料无该
  形态，FIXME 在册不建模）；innerHTML 身份分裂的表示面本身（查询/树视图
  统一）仍是深水面，触发通道收口 ≠ 表示面收口。
- **承袭不变**：LinkStyle sheet 属性面 1F；rel=preload fetch 面（link-multiple
  案 5 + 案 6 页级 10s 时序必然）；stylesheet-media-change-no-fetch 1F；meta
  long × corpus 通道放宽失效；details 族 lib flake；promise_test 串行启动面；
  待决三项（css/text 语料资源、green-2x2.png、parser form pointer 缓行）须
  用户拍板。

## 下一步（s155 候选）

①rel=preload fetch 面（link-multiple 案 5 + 缓存复用——s124 modulepreload 臂
同型扩展）；②LinkStyle sheet 属性面 1F；③link-load-event document load 阻塞
面（Check if the stylesheet's load event blocks the document load event 1F）。

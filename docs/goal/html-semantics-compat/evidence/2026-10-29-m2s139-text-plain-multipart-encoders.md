# M2 s139 · text/plain + multipart/form-data 编码器 + echo 端点 CR/LF 字面交付——form-submission-0 三文件镜像 36P/26F（72 F→P）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s138（form-submission-0 提交主干）

## 入场定刀位

控制面 s139 候选①：text/plain + multipart/form-data 编码器（s138
主干落地后 urlencoded2 已 36P/26F，text-plain / multipart-formdata
仍 0P/62F 全 F——序列化层缺）。

## spec 核实（不靠记忆）

WebFetch 对 html.spec.whatwg.org 不可达，换 web-reader 抓取
form-control-infrastructure.html 全文，三点定谳：

1. **Convert to a list of name-value pairs**：File 值取
   filename；name/value lone CR/LF → CRLF 归一——urlencoded 与
   text/plain 共用（File→filename 即 urlencoded File 案期望仅
   filename 序列化的 spec 出处）。
2. **text/plain encoding algorithm**：`name=value\r\n` 逐对，
   零转义——NUL 字面、无百分号编码（语料 `a\0b=c\r\n` 期望吻合）。
3. **multipart/form-data encoding algorithm + RFC 7578**：part 头
   name/filename 经 escape a string（仅 LF→%0A、CR→%0D、`"`→%22，
   其余字节字面进 UTF-8 头）；非 File value **不转义**（仅换行
   归一）；boundary 不得出现在任何 name/value 中。语料
   `expected.name` 是**转义后形态**（`a\nb` → `a%0D%0Ab`、
   `\n\rb` → `a%0D%0A%0D%0Ab`），expectedBuilder 的 raw 模板收到的
   本就是转义串——无矛盾（开场担心的转义/字面 CRLF 冲突不存在）。

## 施工（2 文件：shim 1 + runner 1）

- **part04 `_zwFormSubmitNavigate`**：enctype 读从 ⑤b POST 分支
  提升到 body 构建前（submitter formenctype > form enctype >
  缺省，非法值回 urlencoded）；新增 `_normNL`（lone CR/LF → CRLF，
  convert 步语义，三序列化共用，`_encPair` 改用之）；POST body 按
  enctype 分型——text/plain `name=value\r\n` 零转义；multipart
  boundary（固定前缀 + 随机 24 字符 = 47 字符 < spec 70 上限，
  与 name/value 撞串重掷 8 次兜底）+ part 头 escape a string +
  非转义 value + `--B--\r\n` 终结；`_urlBody`（urlencoded）另存，
  GET query 恒用它（spec GET 分支固定 urlencoded 序列化，与
  enctype 无关）；⑤b headersWire multipart 值带
  `; boundary=` 参数。
- **runner echo-content-escaped 端点**：CR/LF 改字面交付（0x0D/
  0x0A 不转义）——s138 转义为 `\x0D\x0A` 与 helper `unescape()`
  首步 `/\r\n?|\n/g → "\r\n"` 归一不兼容（归一只作用于字面
  CR/LF、先于 `\xNN` 解码，转义形态解码后不再归一，text/plain
  `\n` 展开案必错）。单测重写：helper unescape 的 JS 等价实现
  （含首步归一 + `\xNN` 还原）双向校验 + lone LF 案形态
  （`a\nb=c\n` → `a\r\nb=c\r\n`）实证。

## 验证

- 定向 form-submission-0 全目录：**三文件各 36P/26F 完全镜像**
  （text-plain 0P→36P、multipart 0P→36P、urlencoded2 36P 不变）；
  残余 26F/文件 = File/filename 层（s140）+ w1252/surrogate
  （候选④）；唯一 Timeout = form-double-submit-input-type-change
  （s138 在册 targetted-form 族，候选③面）。
- 全量 join（对 s138 full log，(key,state) 对级去重后）：
  **72 F→P 全部为 text-plain/multipart 字符串案**（各 36），键零
  消失零新增，无其他任何状态位移；行数守恒精确闭合
  （6866+1018+97 → 6938+946+97，ΔP=+72/ΔF=−72/ΔT=0）。
- engine lib **3590/0**（与 s138 同口径基线）；runner bin
  255P/2F（+1F = 本轮端点单测首跑暴露自身笔误——helper unescape
  的 `take(2)` 从 `x` 起收吃掉首个 hex 位，修复后过；另一 F =
  trend_csv_check encoding CSV 行错位，归口 encoding 流不变）；
  clippy（zero-engine + zero-wpt-runner --all-targets -D
  warnings）零告警；fmt runner 过、engine 仅在册 url_idna.rs
  违例（并行流 14dbfb011，不代写）；shim 拼接（13 part 按
  js_dom_bridge.rs 序）node --check 过；语料零改动。

## 挂账（本切片更新）

- **form-submission-0 残余**（每文件 26F 同构）：①File 条目层
  （s140 候选②：file input entry 贡献 + input.files/DataTransfer
  面 + FormData append File 通道——formdata 通道 File 案 got
  `a=%5Bobject+File%5D`；multipart File part 另需 filename/
  Content-Type/File bytes 面）；②accept-charset windows-1252 +
  `&#NNNN;` 实体回退 + lone surrogate USV（候选④，每文件
  4-6 案）；③targetted-form 58 新见 F 深层面勘根（候选③）。
- **echo 回显体 HTML 解析面观察**（零语料暴露，不施工）：iframe
  子文档按 HTML 解析回显体，`body.textContent` 回读——可打印
  ASCII 的 `&`/`<` 字面若出现在提交体（上游 text/plain iframe
  不走 HTML 解析）会被解析面吞；当前三语料字面 `&`/`<` 仅存在
  于 w1252 案期望侧、提交体永不携带，无暴露面。未来语料扩展时
  复核。
- 承袭不变：「Named elements should override builtins」1F 挂
  待决；encoding-legacy-mb CSV 行错位归口 encoding 流；fmt
  url_idna.rs 违例归口并行流 14dbfb011；RadioNodeList live 语义
  近似、past names 条目不显式清理、expando store/target 双轨、
  part03 raw 链 type 读通道未勘 + reset 臂无还原臂、engine lib
  details flake、reset.html form element pointer 面（待决缓行）、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme
  21F、写流盖章断链、part02 setAttribute 绕 notify、041 nested
  write、deep clone 非 HTML ns、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with
  透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象
  基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live
  树逗号列表 qSA 根缺口、渲染流面让位累积、s131 插入步缺口细化、
  radio 现象⑤/checked 属性增设/defaultChecked 无独立 corpus 面、
  s135 已知限制 3 条；**待决（须用户拍板，既有）**：css/text
  数据模块语料资源缺失、green-2x2.png 语料资源分发、parser form
  element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **重复键会污染键级 join**：同文件在全量日志内可跑多次（同键
  同跑内 2F+1P 三行——form-submit-action.html 实测），键级 join
  对重复键叉乘出假 P→F/F→P（本轮 16 假 P→F 险些误判回归）。
  权威对账须 **(key,state) 对级 sort -u 后 comm**：位移只看
  pair 增删（本轮 losses 全为旧 F 态、gains 全为新 P 态、同键
  一一对应），再辅行数守恒（ΔP=+72/ΔF=−72）双向闭合。
- **转义方案必须吃透消费方的完整还原序**（s138 `\x5C` 教训的
  姊妹案）：helper `unescape()` 首步换行归一先于 `\xNN` 解码，
  端点对 CR/LF 转义会让解码产物脱离归一步——端点行为不能只对
  「可逆」负责，要对消费方管道的每一步顺序负责。
- **跨通道字符保真前提先查再写**：text/plain 裸 NUL/非 ASCII
  进 POST body 是 urlencoded 通道从未走过的路（encodeURIComponent
  全编码）——施工前先核 QuickJS 回调参数转换
  （JS_ToCStringLen2 按长取字节 + from_utf8_lossy：NUL 保真、
  lone surrogate → U+FFFD）才敢落笔。

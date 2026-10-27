# M2 s122 · classic script 编码链三面收口（characterSet meta 嗅探回退 + 文档编码解码回退 + charset 属性感知）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s121（a/area URL 分解组件三面）

## 入场勘根（三候选）

1. **form-submission-0 219F 定谳**（挂账类精确化，不施工）：186F
   （urlencoded2/text-plain/multipart 各 62）= POST body 传输缺失——host 导航
   契约（`ScriptSourceFetcher`/导航回调族）无 method/body 通道 + echo 端点
   （echo-content-escaped.py 等）不在语料 pull 面——双大基建；33F（algorithm
   8 + action 10 + newline 12 + checks 3）= form `target≠_self` 命名 iframe
   导航——runner 无 iframe 文档/几何管道（testharness.rs:3320 既有判定）。
   附属发现：**newline-normalization 12F 独立单根**——FormData 构造换行归一
   过度（纯 shim 面，无基建依赖），转常规刀位候选。
2. **links 域剩余散 F 复盘：0F**（仅 4 既有 Timeout 导航面）——候选出局。
3. **新刀位：script-charset-02/03**（runner + shim 双面，s115 账本挂账收口
   ——s115 翻绿 charset 族 17F 时明确挂起「②文档编码 windows-1250 未建模
   （02 假设臂 3F）③运行期 charset 属性变脸无元素上下文（03 1F）」）。

## 根因链（探针页四轮递进定谳，全部 gitignored corpus 验后删除）

初版三刀（runner 嗅探 + part06:2986 面新 getter + part03 gate charset 感知）
落盘后 02 仍 3F，逐层勘出三个叠加根因 + 一个施工形态错误：

1. **JS 对象字面量重复键覆盖**（初版刀B 无效根因）：shim 主文档是
   `globalThis.document = {`（part06:2986 起）**单个巨型字面量**（约 2986→
   5700+ 行），其内 R5000 片 c 已有 `get characterSet()`（约 5568 位）——
   后键胜，s122 初版在 2986 邻位新增的同名 getter 整个被 R5000 面吃掉。
   施工形态纠正：删新增重复键，嗅探回退并入 R5000 既有 getter（host 值优先
   保 R5000 判据面零变化）。**字面量内同名属性唯一化**为后续同面施工红线。
2. **harness 前导注入挤出 prescan 窗**（刀A 无效根因）：runner 装载链
   `replace_script_source("/resources/testharness.js", harness)` 把 ~200KB
   harness 内联在 case 原 `<script src=testharness.js>` 位——02 语料该引用
   在 byte 593 meta **之前**，注入后 meta 被推至 ~200KB。嗅探若吃注入后
   html，1024 字符窗内全是 harness 前导 → doc_encoding=None → 解码链回退
   UTF-8 lossy（arm2 得 lossy mojibake、arm3 得「正确解码」反向失败）。
   纠正：`inline_local_scripts` 参数化 `doc_encoding: Option<&str>`，调用方
   对**注入前原始 source** 嗅探（主装载链 9521 位 + fetch 通道 8209 位对
   substituted 同款）；6 个既有测试点传 None。
3. **host 回调恒 truthy 兜底短路嗅探臂**：`__zw_get_character_set` 无
   label 时 `unwrap_or_else(|| "UTF-8")` 恒返 truthy → shim getter 的
   host 直读臂永不落空。同根复现：host 预扫描读的是 dom_html 快照
   （= 注入后 html），1024 窗同样被 harness 挤空。纠正：host 无 label 返
   **空串**（falsy 让位），truthy 直读路径不变（R5000 quotes-in-meta /
   meta-inhead-insertion-mode 判据面零变化）。
4. **head innerHTML 树位源结构性失效**（02 假设臂真根因）：02 语料 head 内
   `<div id="log">` 触发 spec tree construction **隐式 body 化**（in head
   模式遇 div 即关闭 head 开启 body），其后 `<meta charset>` 被解析进
   **body**——探针实测 `headLen=201337 / metaAt=-1`（charset= 在 head
   innerHTML 中不存在）。真浏览器不受影响靠的是**字节流预扫描**（解析前），
   任何以解析后树位（head.innerHTML）为源的嗅探都结构性漏配。纠正：shim
   嗅探源改 `querySelectorAll('meta')` **元素遍历按属性读**（charset 属性
   直读 + http-equiv=content-type content 解析两形；兼消除脚本原文 `<meta`
   字面量的正则误匹配面）——meta 元素是真实节点，树位无关。

另两个探针方法论落坑（复用警示）：

- **探针页须带 `/resources/testharness.js` 引用**——无引用页被
  `html_semantics_case_skipped` 排除，跑出零输出零 Fail 的伪空跑。
- **windows-1250 字节表勿背书**：首版 pin 以猜的 `9C E6 A5 8F BF` 写
  "śćążź" → 解码得 "śćĄŹż" 假红；corpus external-script-windows1250.js
  实测真值 `9C E6 B9 BF 9F`（ą=B9、ż=BF、ź=9F）。

## 施工（净变更 part03/part06/callbacks.rs + testharness.rs）

1. **runner**：`sniff_document_encoding`（meta prescan 1024 窗 + encoding_rs
   for_label 归一）；`decode_classic_script_bytes` 4 参（BOM > ct charset
   参数 > charset 属性 > **文档编码** > UTF-8 lossy）；`inline_local_scripts`
   增 `doc_encoding` 参（doc 注明「入参方须对注入 harness 前的原始 case html
   嗅探」）。
2. **shim part06**：R5000 面 `get characterSet()` 扩展——host truthy 直读 →
   `__zwMainDocEnc` 缓存（'' 哨兵）meta 元素遍历嗅探（`_zwEncodingFromLabel`
   htmlMeta 归一）→ 'UTF-8' 缺省（runner 本地通道页面恒 UTF-8 交付，缺省
   变更会翻动全量基线）；charset/inputEncoding 别名自动同源。字面量内
   同名键唯一化注释固化。
3. **bridge callbacks.rs**：`__zw_get_character_set` 无 label 返空串。
4. **shim part03**：`_zwPrepareSrcScriptGate` 动态外链读 charset 属性（sel
   形 `__zw_get_attr_lw` / handle 形 `__zw_get_attr_handle` 双路）→
   `_zwEncodingFromLabel(label,false)` → then 链 TextDecoder 按该 label 解码
   （无 label / 无 TextDecoder / 无 arrayBuffer 回退 `_r.text()` 原路）。

## 验证

- 定向 `FILTER=script-charset` **10/10**（01 六保持 + 02 三翻绿 + 03 翻绿）。
- **全量 A/B**（/tmp/s121-full.log → /tmp/s122-full.log，逐行 status join）：
  **恰 4 对 Fail→Pass**（02 三 + 03 一），**真翻红 = 0**（6635P/1163F/105T →
  6639P/1159F/105T）。
- bin pin `script_charset_s122`（两面：characterSet 嗅探 [div 后 meta 推入
  body 形] + 文档编码回退解码 [windows-1250 字节文件]）：新代码绿；engine
  三文件 stash 往返**红**（`characterSet meta 嗅探: UTF-8` 行为失败，非编译
  失败）。face ② 旧码红由全量 A/B 基线（s121-full.log 02 三 F 实消息）承载
  ——runner 修复与 pin 同文件不可分面 stash，诚实记法。charset 属性变更脸
  （03 形 async_test）由套件 corpus script-charset-03 本身覆盖，不入 pin。
- engine lib **3570/0**；runner 测试模块 **254/0**（76 既有 ignore 不变）；
  guard 内 clippy（runner/engine --all-targets）零告警（修一处
  needless_option_as_deref）；fmt 过；shim 拼接语法（真实装配序
  node --check）过。渲染面零依赖——reftest/product-smoke 未跑。
- 语料零改动（探针页 probe-s122-docenc/pad/pre 三个验后删除）。

## 挂账（本切片新增/更新）

- **script-charset-02/03 收口移出承袭清单**（本切片三刀 + 上记根因链）。
- **form-submission-0 219F 挂账精确化**：186F POST body 传输契约（host 导航
  契约扩展 method/body + echo-content-escaped.py 等端点仿真——语料外）+
  33F form target≠_self 命名 iframe 导航（runner iframe 文档管道）——均
  大基建类。
- **newline-normalization 12F 单根候选**：FormData 构造换行归一过度（纯
  shim 面，无基建依赖）——s123 首选刀位候选。
- 承袭不变：写流执行身份盖章断链 FIXME、part02 setAttribute 绕 notify 臂
  （gap 2）、execution-timing 构成面、041 nested write、lazy-loading 26F、
  meta color-scheme 21F、css-module corpus 门（待决）、www2 可达性、
  modulepreload referrerpolicy 族、module-import-referrer TLA、
  script-crossorigin-network cookies、redirect-relative-urls、
  export-from with FIXME、143 write、defer 多写完成序、dynamic import()
  with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、parser form
  element pointer 4F 缓行（2026-10-23，并入 declarative shadow DOM parser
  批次）。

## 遗留近似

- shim 嗅探为**全文档 meta 元素遍历**（首个有效 charset label 定码），与
  spec 1024 字节 prescan 窗不严格同界：解析位移（div 推 meta 入 body）反而
  比 prescan 更宽容；动态建 meta 后首次 characterSet 读会命中（prescan 不
  会）。现语料无漂移面（全量 A/B 翻红 0）；若未来 corpus 出现「meta 后置/
  动态 meta 不应生效」判据需复查。
- host 空串让位后，真实引擎管线中「解析 prescan 无 label + head 含 meta」
  的页面 characterSet 从 'UTF-8' 变为嗅探值——方向 spec 正确（characterSet
  应反映文档声明编码），全量 A/B 证零回归。

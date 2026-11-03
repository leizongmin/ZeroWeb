# M2 s258 — import map 映射值注册期归一化（共享解析层小刀；5 钉全绿 + s257 口径勘误）

日期：2026-11-03。切片号 s258（承 s257 挂账「import map 映射值归一化缺口」，
s257 下一步候选 #1）。单层小刀：共享解析层（script-sandbox import_map.rs）
一处归一化，webview / renderer / tab 三消费面一并收敛；零 app 代码改动。

## 勘根（承 s257 挂账）

1. **缺口本体**：`normalize_specifier_map`（import_map.rs:163）把字符串
   映射值**原样存储**（无基址归一化）——`resolve_imports_match`
   （import_map.rs:199）对尾斜杠前缀键目标裸 `Url::parse`（无基址），
   path-absolute（`"/assets/utils/"`）/relative（`"./shim.js"`）目标解析
   失败 → 误 **Blocked**；精确键相对值原样返回（`Resolved("./shim.js")`），
   靠接线侧 `resolve_document_url` 再拼基址偶然可用，而 `resolve_mapped`
   消费方（webview apply_import_map / js_worker·tab collect_module_deps /
   script_prefetch）拿到的是裸相对串——按导入方基址错误抓取或失败。
2. **规范核实（勘定 s257 挂账口径错误）**：sorting and normalizing a
   module specifier map 步骤 3-6——字符串映射值经 resolving a URL-like
   module specifier 按基址解析；解析失败（步骤 5）或尾斜杠键的地址序列化
   不以尾斜杠收尾（步骤 6）→ **置 null（阻断条目）并继续**，该成员仍
   注册。s257 evidence 勘根 #3 与挂账「归一失败**整张 map 拒注册**」与
   规范不符：整张拒注（TypeError）仅限 JSON 非法 / imports·scopes 非对象 /
   scope 内层非对象。本切片按规范实现置 null 语义。
   https://html.spec.whatwg.org/multipage/webappapis.html#sort-and-normalize-a-module-specifier-map
3. **消费面漂移预核对（零漂移证据）**：grep 全部消费面测试 map JSON——
   es_module（2038/2072）、js_worker（3128/3159/3177）、tab_js_worker
   （1035）均为规范形态绝对值（尾斜杠键地址带尾斜杠）→ 行为零变化；
   parse 的 `Err` 面不变（值级失败从不产 `Err`）→ 宿主「首张合法 map
   生效」语义不受影响；corpus（wpt-data/html/semantics）grep importmap
   零命中 → runner 路径无可观察位移（非 s257 的「路径不变」论证，是
   「语料无此通道」的强论证），无全树 A/B。归一化只在 parse 时执行一次
   （非热路径），resolve 路径代码未动 → 无 bench-gate 项。

## 施工（1 刀 + 5 钉 + 1 钉数据校正）

1. **归一化刀**（import_map.rs `normalize_specifier_map`）：字符串映射值
   经 `resolve_url_like`（即规范 resolving a URL-like module specifier，
   与键归一化同一函数）按基址解析，存序列化 URL；解析失败或尾斜杠键的
   地址不带尾斜杠 → `None`（阻断条目）。非字符串值原置 None 不变
   （步骤 3）。
2. **注释面**：`resolve_imports_match` 目标解析失败臂注明对 parse 产物
   不可达（注册期已归一化为绝对 URL——阻断面收敛），保留作存储值不变式
   守卫；`parse` / 模块文档同步值归一化口径。
3. **钉面 ×5**（import_map.rs tests，s258 后缀）：前缀键 + path-absolute
   值（resolve + resolve_mapped 双臂）；精确键 + `./`·`../` 相对值
   （resolve 绝对化 + 存储值断言）；裸串值 → Blocked（步骤 5）；尾斜杠键
   地址不带尾斜杠 → Blocked（步骤 6）；scope 内值归一化。
4. **钉数据校正 ×1**：`non_special_url_skips_prefix_matching` 的 map 值
   `"https://x.test/a"` → `"https://x.test/"`——旧值带尾斜杠键 + 不带尾
   斜杠地址，新归一化下正确置 null；钉面意图（data: 非 special 跳前缀
   匹配走 asURL 回退）不变，数据改规范形态。两侧行为等价（新旧代码同
   绿）。

## 验证

- **红态实证**：5 钉先于修复单独成 commit 前，在 HEAD（旧代码）跑
  `cargo test -p zero-script-sandbox import_map`——恰好 5 失败且失败模式
  逐一对准目标行为：前缀键 path-absolute 值 `Blocked`（应 Resolved）、
  相对值原样 `Resolved("./shim.js")`、裸串 `Resolved("bar")`（应
  Blocked）、尾斜杠不一致 `Resolved("https://cdn.example/a.js")`（父级
  路径，应 Blocked）、scope 值 `Blocked`。旧版因目标行为失败证据在账。
- **绿态**：修复后 `cargo test -p zero-script-sandbox`（default v8，
  test-guard 内存/墙钟守卫）**390 全绿**（385 基线 + 5 钉；含 es_module
  v8 面 + worker 面）。
- **fmt/clippy**：`cargo fmt -p zero-script-sandbox` 零改写（diff 仅本次
  施工行）；`cargo clippy -p zero-script-sandbox --all-targets`（test-guard
  守卫）零告警。
- **无全树 A/B**：语料零 importmap 面（勘根 #3）→ runner 通道确定性零
  位移；消费面测试 map 全规范形态 → 三面行为零漂移（#2）。

## 挂账更新

- **s257 口径勘误**（本切片记档，不改已归档 evidence 原文）：s257
  evidence 勘根 #3 及挂账中「归一失败整张 map 拒注册」→ 实为置 null
  条目并继续（见勘根 #2）。
- **resolve_imports_match 回溯臂缺口**（新记档，s257 阻断面收敛的残余
  面）：规范 resolve an imports match 步骤 8——resolutionResult 序列化
  不是 normalized 序列化的代码单元前缀（afterPrefix 回溯越出前缀域）→
  TypeError；当前实现缺该臂（如 `"lib/": "…/x/"` + afterPrefix `../a.js`
  解析出前缀域外 URL 按成功返回）。与 Blocked 降级语义同域可并刀
  （Blocked 降级：null 条目规范要求 TypeError 终止全部回退，现按 Miss
  降级——`String` 返回签名无错误通道，属接口面改造）。
- 承前不动：import maps 剩余语义面（多张 map 整页失败语义 / 动态插入）；
  data-zw-harness 投递标记保真面（s255 候选 #3）；credentials 面 /
  shared-dependency 取回计数 / microtask checkpoint 时序族 / 并发慢
  import dedup（s256）；customizable-select 177F（挂起待用户拍板
  GB-20261031 征询在途）；renderer SetDomSnapshot entries 化（s174，
  低优先）。

## 下一步候选

1. import maps Blocked 降级语义 + resolve_imports_match 回溯臂
   （同域并刀——`ImportMapResolution` 错误通道改造：Blocked/回溯越界 →
   TypeError 终止全部回退，动 es_module resolve_registered_specifier /
   宿主回调 / webview apply_import_map 三接线面）。
2. data-zw-harness 投递标记保真面（s255 候选 #3 承前）。
3. import maps 剩余语义面其余（多张 map merge / 动态插入）。

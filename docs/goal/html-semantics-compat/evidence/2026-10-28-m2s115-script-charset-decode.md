# s115：script charset 响应解码序——serve-with-content-type.py 仿真推广，charset 族 17F 翻绿

日期：2026-10-28。目标域：`scripting-1` charset 族（script-charset-01 +
module/charset-01/02/03）。两文件轮：runner `testharness.rs`（取回/内联两取回
路径）+ `tests/wpt-runner/Cargo.toml`（+encoding_rs，工作区既有 0.8）。

## 入场定谳（失败面分组）

charset 族五组失败三根因：①`serve-with-content-type.py` **动态端点不在语料
pull 面**——classic 面文件读落空整段透传外链、宿主取回 Err 不执行（getSomeString
undefined，script-charset-01 全 6F）；模块面 s107 仿真被 JSON 意图门收窄，非
JSON ct → Err 网络错误（module/charset-01/02/03 全 11F 根因）。②文档编码
windows-1250 未建模（script-charset-02 假设臂 3F）。③运行期 charset 属性变脸
无元素上下文（script-charset-03 1F）。②③本轮挂账；css-module/text-module
charset 5T 属 css/text 数据模块语料缺失待决（须用户拍板）。

## spec 模型（两取回路径分流）

- **classic**（fetch a classic script）响应解码序：BOM 嗅探 > ct charset 参数 >
  charset 属性 > 文档编码（runner 恒 UTF-8）。parser-inserted classic 走
  inline_local_scripts 内联投递——元素上下文在位，charset 感知在此完成。
- **module**（fetch a single module script）：**恒 UTF-8**——charset 参数/属性/
  BOM 三忽略（module/charset-01 判据即 mojibake 应答 `assert_not_equals`；
  charset-02 UTF-8 BOM 体内 U+FEFF 为 JS 空白照常执行、UTF-16 内容乱码语法错
  → executed_* undefined）。模块走 wpt_data_script_fetcher。

## 修复

1. **共享查询解析** `parse_serve_with_content_type_query`（fn/ct 百分号解码，
   两面共用）。
2. **fetcher scripty 意图分支**：`.py` 意图非 JSON 而 scripty（essence 判定
   `mime_is_scripty`）→ 交付 fn 文件，解码**恒 UTF-8 lossy**（fetcher 无脚本
   类型/charset 属性上下文；BOM 不嗅探——体内保留 U+FEFF 即 spec 语义）；
   非 scripty 非 JSON → Err（s107 口径保持）。charset-01/02/03 全绿。
3. **inline classic 仿真分支**：`.py`（文件名匹配，与 fetcher 同口径）+ **非
   module 型**→ 交付 fn 文件，`decode_classic_script_bytes`：BOM（encoding_rs
   for_bom）> ct charset 参数 > charset 属性 > UTF-8 lossy。script-charset-01
   全绿。
4. **首版乌龙**：inline 分支未辨 module——module 外链被 charset 感知内联改写
  （charset-01 余 4F 反向翻红、charset-02 1F→2F：BOM 嗅探令 UTF-16 模块照常
   执行）。修正为 module 型整段透传（ASCII 剥白 + ASCII-ci 本地判定，与 s113
   分类器同口径不扩 engine 公共面）后族内全绿。

## 验证

- runner bin **240/0**（+2 钉：fetcher scripty 意图、inline 解码序四臂+module
  透传臂）。
- charset 族定向（FILTER=charset）：script-charset-01 **6P**、module/charset-01
  **6P**、module/charset-02 **3P**、module/charset-03 **4P**——17F 全翻绿；
  script-charset-02/03、css/text-module charset 维持挂账形态；json-module/
  charset 既有 Pass 不动。
- **全量 testharness 套件 A/B**（vs s114 全量基线，test-guard 包裹）：
  6545P/1220F/112T → **6565P/1202F/111T**（+17P/-17F 即 charset 目标面）。归一
  per-page diff 三处：①charset 四页 17F→P（目标）；②focus 页 1T+77P→80P
  （上轮已定性负载 flake，本轮满绿复原）；③modulepreload 3F+1T→2F+1T（该页
  常态 2F+1T 与 s113 基线/两轮定向重跑一致，s114 全量时的多余 Fail 行为
  completion-callback 超时注入的假行）。**全树零 P→非P 成立**。
- fmt + clippy（zero-wpt-runner --all-targets -D warnings）过。渲染面零依赖
  ——reftest/product-smoke 未跑。
- 依赖：runner Cargo.toml +`encoding_rs = { workspace = true }`（工作区根
  Cargo.toml 已有 0.8，dom/net/engine 在用；本提交同步）。

## 挂账

- **script-charset-02**（3F）：文档编码 windows-1250 未建模（runner 全链 UTF-8
  读语料；涉及 corpus loader 编码面基建）。
- **script-charset-03**（1F）：运行期 charset 属性变脸——shim fetch(url) 裸
  接口无元素上下文，需 fetch 桥带 charset 提示（跨界契约变化）。
- **css-module/text-module charset**（5T）：css/text 数据模块语料资源缺失——
  待决（须用户拍板）承袭。
- classic 运行时取回（mutation 触发臂等）经 fetcher 无 charset 属性上下文，
  恒 UTF-8 近似（parser classic 面已感知；语料无此形）。
- 其余承袭 s114 挂账不变（143 write 注入 `</script>`、defer 多写入完成序近似、
  063 Timeout、non-external-no-import、parse-error 族、referrer-policies 等）。

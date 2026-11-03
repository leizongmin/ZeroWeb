# M2 s262 — import map 注册序面：映射表与 scope 代码单元降序排列（最长前缀 / 更具体 scope 优先）

日期：2026-11-03。切片号 s262（承 s260 挂账 #2 / s261 下一步候选 #1
「import maps 剩余语义面」——三面勘根分型：键排序序偏差本切收口，
多张 map 合并 / 动态插入两面勘根落账按架构级缓议）。

## 勘根

1. **三面分型**（s260 挂账 #2 的三个子面）：
   - **① 键排序序偏差（本切施工面）**：模块 doc 偏差注记（import_map.rs
     旧 10-11 行）与知识 modules.md 在案「当前 scopes/prefix 的遍历也不能
     保证规范最长前缀」。spec 精确口径（HTML webappapis，2026-10-14 版
     实抓原文）：`sort and normalize a module specifier map` 与
     `sort and normalize scopes` 收尾步均为「sorting in **descending**
     order … a's key is **code unit less than** b's key」，规范注记明言
     降序把 `"foo/bar/"` 排在 `"foo/"` 之前、解析期优先级更高。本实现
     `normalize_specifier_map` 按 `serde_json::Map`（未启用
     `preserve_order` → BTreeMap）**升序**迭代入 Vec，
     `resolve_imports_match` / `match_map_tables` 按序首个命中生效——
     嵌套尾斜杠前缀键最短前缀遮蔽、嵌套 scope 最浅遮蔽，方向恰反。
     缺口两面：解析行为面（resolve/resolve_mapped 共用
     match_map_tables 顺序）+ 存储面（注册期 Vec 序）。
   - **② 多张 map 整页语义（缓议）**：spec 已改 merge 模型
     （register an import map = "Merge existing and new import maps"，
     含 **resolved module set** 护卫——已解析模块不受新规则回溯影响；
     无效 map 不注册且不阻断后续）。现行 webview（webview.rs
     `find_map` 首张合法生效）与 renderer `parse_page_import_map`
     （js_worker.rs:2752 同型；既有钉
     `parse_page_import_map_first_valid_wins` 钉「无效忽略取后续合法张」
     ——该子行为与新模型可观察面一致）为「首张合法生效」。缺口 = 两张
     合法 map 应合并（冲突旧胜 + resolved module set 护卫），现行第二张
     整张忽略。**架构级**：需 per-document 解析记忆（静态+动态全通道）
     与 webview/renderer/tab 三宿主同布，corpus 零 face（全域唯一
     importmap 语料页单张合法 map）——本切不施工，挂起候审。
   - **③ 动态插入（缓议）**：spec「A Document can have multiple import
     maps processed, which can happen either before or after any modules
     have been imported」；现行 `extract_import_map_json`（engine
     extract.rs）只抽静态 HTML，脚本运行期插入的 importmap 永不注册。
     与②同属注册通道架构面，corpus 零 face——一并挂起。
2. **消费面盘点**：全部消费方（webview apply_import_map /
   `__zw_importmap_resolve`、renderer js_worker / script_prefetch、
   tab_js_worker）单点经 `ImportMap::parse`——共享路径一处修复全覆盖
   （renderer import 过滤 9 钉、browser bin blocked 过滤 4 钉在册验证）。
3. **corpus 面**：全域唯一 `type="importmap"` 页
   import-meta-resolve-importmap.html 键全 exact、无尾斜杠前缀键、
   无 scopes——排序不可观察，零 A/B face（s258 先例同判：corpus 零
   importmap 排序 face → 无全树 A/B）。
4. **红态可行性**：2 枚自包含行为钉（嵌套前缀键 / 嵌套 scope），HEAD
   可实证恰反序解析。

## 施工（1 文件 2 刀 + 2 钉 + 1 知识面）

1. **`code_unit_cmp` 助手**（import_map.rs）：UTF-16 代码单元逐一比较
   （`encode_utf16().cmp`）——与 Rust 字节序仅非 BMP 字符与
   U+E000..=U+FFFF 区段交错时有别，按规范精确实现，不留偏差。
2. **注册收尾双降序**：`normalize_specifier_map` 出口按键降序排列 +
   `parse` scopes 列表出口降序排列；两处附 spec 链接。函数 doc 补
   排序步骤；模块 doc 偏差注记改写：排序面已合规，残余微偏差 =
   多个不同原始键归一化为同一键时取值归属按排序序而非 JSON 插入序
   （`preserve_order` feature 为 workspace 级架构面，不动）。
3. **s258 钉存储序断言随新序更新**：
   `relative_value_normalized_at_registration_s258` 期望 Vec 改
   up→shim（降序）——该钉意图为值归一化非排序，断言注释标明 s262。
4. **钉 ×2**：`nested_prefix_keys_longest_prefix_wins_s262`
   （imports 嵌套前缀键 `a/` vs `a/b/`，resolve + resolve_mapped 双面）
   + `nested_scopes_most_specific_wins_s262`（scopes 嵌套 `/s/` vs
   `/s/t/` 主臂 + 浅层 scope 判别臂）。
5. **知识 modules.md 边界条目重写**（随本切代码提交）：s260 Blocked
   三态化欠账一并事实化；边界收窄为多张合并 / 动态插入两面（架构级
   缓议），「不能按解析器结果枚举推出端到端合规」保留于该两面。

## 验证

- **红态实证**（HEAD + 钉，test-guard）：两钉恰反序 FAIL——
  `nested_prefix_keys` 得 `https://x.test/short/b/c.js`（最短前缀
  遮蔽）、`nested_scopes` 得 `shallow.js`（最浅 scope 遮蔽），与勘根
  方向判定逐位吻合；既有 24 枚 import_map 测试绿。
- **绿态**（test-guard）：
  - `cargo test -p zero-script-sandbox --lib` **395 全绿**
    （393 基线 + 2 钉）。
  - `cargo test -p zero-webview --lib` **839 全绿**（== s261 基线）。
  - `cargo test -p zero-renderer --lib import` 过滤 **9 全绿**。
  - `cargo test -p zero-browser --bin zero-browser blocked` 过滤
    **4 全绿**（含 tab_collect_module_deps_blocked_entry_terminates_s260）。
- **fmt/clippy**：`cargo fmt --all --check` clean；clippy
  script-sandbox `--all-targets` 零告警。
- **全树 A/B 免跑判据**：改动臂 = 嵌套尾斜杠前缀键 / 嵌套 scope 重叠
  时的注册序，corpus 唯一 importmap 页键全 exact（勘根 #3），消费面
  套件绿态兜底——引 s258 先例免跑。

## 挂账更新

- **s260 挂账 #2 三面销一**（键排序）；两面缓议新记：**多张 map merge
  注册（resolved module set + 跨宿主同布）与动态插入注册通道**——
  架构级（per-document 解析记忆、三宿主、serde_json preserve_order
  权衡），corpus 零 face，挂起待用户拍板（同 GB-20261031 形态，
  不阻塞本流）。
- 知识 modules.md 边界条目重写随本切（s260 三态化欠账一并事实化）。
- 承前不动：data-zw-harness 基建脚本注入序（s255 D 刀余项）；microtask
  checkpoint 时序族（s255）；credentials / shared-dependency / 并发慢
  import dedup（s256）；customizable-select 177F（GB-20261031 挂起）；
  renderer SetDomSnapshot entries 化（s174，低优先）；engine lib 5 红
  （R-t2qq adopted-stylesheets 系列既有，归因该流自查，本流不入侵）。

## 下一步候选

1. data-zw-harness 基建脚本注入序（s255 D 刀收窄余项）。
2. scripting-1 module 面勘根分型（s248 遗留候选：css-module redirect
   对 3F，module 链独立根车道）。
3. s255 候选清单其余（见 s255 账页）。

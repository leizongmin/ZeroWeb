# M2-S165 meta color-scheme 文档 scheme 三层栈 + gCS 结构性重放通道

日期：2026-10-31。基线：s162 终案对账（`/tmp/zw-s162-full1.log` 7012P/783F/91T 子测级；该日志 00:17 晚于 s163 提交 00:01:54，树态已含 s163 per-sheet styleSheets 变更——conditionally-block 双断言基线侧为 P，归因注记见 join 仲裁）。切片号承并发注记顺延（s163 候选清单「下一步 s164 候选」②本刀入场，然并发会话已按候选①施工并实际占用 s164 号——`/tmp/zw-s164-*` 工件族可证，本刀施工期注释原标 s164，落账改号 s165、注释同步）。

## 切片目标

s163 候选②设计面：meta color-scheme 语料 24F 收口——used-scheme 动态系统色、文档级 meta 扫描、computed 序列化三态语义三层栈补齐。

## 勘根（三层栈缺位 + 注入机制冲突）

1. **used-scheme 系统色静态化**：CanvasText 族系统色按 prefers-color-scheme 偏好一次性解析，不随文档 used scheme 翻转（meta dark 后 CanvasText 仍黑——color-scheme 语料 24F 主形态）。
2. **文档级 meta color-scheme 整体缺位**：无 HTML §4.2.5.3 文档 scheme 扫描（树序首个有效 meta、无效 content 跳过、shadow tree 排除、显式 `normal` 回落文档 scheme 非 parent）。
3. **既有 R2285/R2286 注入机制冲突**：engine pipeline 以 `html{color-scheme:content}` 最低优先级 stylesheet 注入使 used-scheme 面生效，副作用 = 根 computed color-scheme 序列化变 "dark"（违背 gCS `normal` 断言）+ shadow 泄漏——used-scheme 面与 gCS 序列化面需求相反，注入路线不可两全。
4. **gCS 结构性失明**：computed_style_cache replay 通道仅处理 attr/style 四类变异，结构性变异（insert/move/remove/text）静默丢弃——meta-color-scheme-insert/remove/remove-head 族在 gCS 视图永不生效（探针定谳：JS 视图正确而 gCS 偏差，真因为跨段 handle 身份丢失——同 turn 内 gCS 与变异交错推进切段后 fresh nodes map 不跨应用留存，ref=前缀段创建元素的 InsertBefore 失配）。

## 施工面（12 文件：rust 10 + shim 1 + 测试 1）

1. **css-parser `values/color.rs`**：`parse_color_with_scheme` 尾部 dark 面板——CanvasText 族五键按 used scheme dark=true 翻白（CSS Color 4 系统色 used-scheme 面板）。
2. **style-system `lib.rs`**：`scan_document_color_scheme`——compute 入口扫描文档树序首个有效 meta color-scheme（空串/逗号形无效跳过、祖先链命中 shadow host 排除），产出文档 scheme dark 位（HTML §4.2.5.3）。
3. **style-system `inheritance.rs`**：color-scheme 属性解析 Concrete 臂——显式声明落 `color_scheme_decl`；`normal`（无 light/dark token）回落文档 scheme。
4. **style-system `property/apply.rs`**：主循环 color-scheme 臂收口到预解析臂（`return true` 不覆写）；`_prefers_dark` 参数收编改名注释。
5. **style-system `property/computed_style.rs` + `default_impl.rs`**：`color_scheme_decl: Option<String>` 字段。
6. **engine `pipeline/mod.rs`**：删除 R2285/R2286 注入块（style-system scan 统一接管）+ 孤儿测试/helper/doc 清理。
7. **engine `js_dom_bridge/computed_style.rs`**：gCS "color-scheme" 序列化臂（decl None→"normal"）；全量兜底换权威 applier（树级变异并入重放，nodes 表回填）。
8. **engine `js_dom_bridge/computed_style_cache.rs`**（核心）：replay 通道并入树/文本级变异——`is_tree_mutation` 判定（attr/style 四类 + handle 四类 + 表单状态三类之外的变体）；树级后缀不走增量重放，快照+全队列从零重建（`compute_document_styles_with_inline_overrides_css`，规避跨段 handle 身份丢失）；E14 drain-gen 前进队列重长时 `start` 归零防错切。
9. **engine `js_dom_bridge.rs`**：applier `InsertBefore`/`InsertBeforeByHandle` ref 解析 handle 表回落（ref 为 handle-only 元素——createElement 产物无 selector，selector miss 后 handles 命中）。
10. **shim `part04.js`**：insertBefore 链 sel 父 + handle-only ref 分支——ref 字段填 handle 字符串，applier 侧 miss 回落（置于 R5058 文本分支前）。
11. **style-system `tests/color_scheme_synthesis.rs`**：+4 用例（文档 scheme dark 根翻白 + decl None 序列化保持、first-valid-wins、shadow tree meta 排除、显式 normal 后代随文档 scheme 翻转）+ `meta_doc`/`metas_of` 辅助。

## 验证

- shim 按 include_str 顺序拼接 `node --check` SYNTAX_OK；`cargo fmt` ✓；三 crate clippy `-D warnings` 清（style-system/css-parser/engine）。
- style-system 单测 **2461 + 5 双 target 全过**（0 failed）。
- engine lib 测试 **3612 过/10 败**——10 败经 git stash 对干净基点 b51d8981a 复验同败（fetch wire 9 + ua_display s24_flip 1），clone 环境既有问题与本刀无关。
- 定向 corpus：color-scheme 语料 run5 **25P/0F/0T**（24 靶断言 + 探针；探针删除后复跑 engine lib 确认零新增回归）。
- 全量案级 join（full1 对 s162 full1，(key,state) 对 sort -u + awk 键级分类）：**F2P 24 全靶面**（meta-color-scheme-* 十四文件 + http-equiv-and-name-1，attribute-changes/insert/remove/remove-head/normal-descendant-change/single-value-in-shadow-tree 全族）/ **P2T 0 / T2P 0 / T2F 0 / F2T 0**；P2F 2 = conditionally-block 双断言，归因**基线树态差异非本刀回归**——基线日志树含 s163 修复而本刀基点 b51d8981a 先于 s163 提交，solo 复跑（guarded）该 2 断言在本树确定性 F 确证，rebase 后由 s163 提交带回；LOST 4 / GAINED 154 全落交互时序抖动族文件级截断镜像（select-event/dialog/focus/homeend/selection 族，base 截断→now 完整改善方向，s163 同款归因）。终账 **7172P/776F/88T**（子测级）。
- rebase 后投影：+s163 净位移（conditionally-block 2 键回 P）≈ **7174P/774F/88T**。
- reftest/product-smoke 未跑——本刀无渲染管线变更（系统色面板在 style 解析面、gCS 通道 readback-only、shim 变更限 insertBefore wire 形态），且兼容性回归已由 color-scheme 语料 24 断言常驻 corpus 承载，沿 s161-s163 shim/style 切片先例记账。

## 记账面（本切片不解决）

1. **color_scheme_decl 继承序列化偏差**：无声明后代经 parent-copy 获 decl（序列化现父值），spec 无声明时应表达 used/normal 回落语义——语料未覆盖该形态，FIXME。
2. **跨批 handle 引用边界**：增量通道仍限 attr/style 八类；跨批 handle 引用依赖树级全量重建路径一次性构建 nodes 表，纯增量前缀段的跨段引用失配风险仍在（现由全量重建规避，非结构化修复）。
3. **media 求值视口 1280×800 FIXME** 承袭（s160-s163 同债）。
4. **style-element-media-match-block-script 1F**（s163 记账）——并发会话 s164 施工中，不重复入场。

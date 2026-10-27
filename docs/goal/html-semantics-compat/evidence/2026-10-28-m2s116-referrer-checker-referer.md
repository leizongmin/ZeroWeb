# M2 s116 · referrer 族 JS checker 面（determine-request's-referrer 仿真）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s115（script charset 响应解码序）

## 刀位与判据链

`module/referrer-*.sub.html` 七页 + descendants 九断言 + strict-policies：页面 inline module
跨源导入 wrapper / 直引 `resources/referrer-checker.py?name=X`，checker（上游 wptserve
Python handler，不在语料）回体 `export const referrer = '<Referer>';`，页面断言各臂
Referer 值。判据链完整：上游 handler 语义（GitHub Contents API 实读）→ Fetch
determine-request's-referrer 计算表 → Referrer-Policy 交付链（响应头 pipe 模板 > 文档
`<meta name=referrer>` > spec 默认 SWCO）。入口 0P + 15F + 12T。

预期翻面：6 个 referrer-*.sub.html 页 + descendants 页 9 断言。credentials 12F
（cookies 基建 + www2 可达性）与 modulepreload 族（echo-referrer.py server-state +
link preload 面）入场即排除挂账。

## 施工（按根因发现序）

1. **checker 端点仿真**（`wpt_data_script_fetcher` 合成块）：命中
   `the-script-element/module/resources/referrer-checker.py` → 按
   `compute_module_referer(引用源, 请求 URL, 有效策略)` 回体。引用源 = fetcher 首参
   （顶层即文档 URL、包装子跳即包装 URL——`collect_module_deps_recursive` 逐层穿
   parent）；八策略计算表 + 降级判定（https 源 → http 请求）+ 同源三元组
   (scheme, host, 缺省端口归一)。文档策略经闭包捕获
   `extract_meta_referrer_policy`（窄解析小写双引号形）。
2. **host 别名面**：`strip_wpt_host_prefix` 收窄 `wpt.test | www1.wpt.test`——
   **www2 保持不可达**（dynamic-imports-fetch-error "cross-origin without CORS" 判据
   页依赖 www2 导入必败，放行即 P→F）。
3. **window.origin**（part01.js）：`location.origin` 序列化、opaque → 'null'；
   configurable accessor（`var origin` 可遮蔽）。descendants 部分断言经
   `new URL(rel, window.origin)` 锚。
4. **split_statements `from` 续行守卫**（es_module）：深度 0 行尾 `from`
   （`statement_ends_with_from`，含 `platformfrom` 粘连负例）不得切语句——
   `import {x} from\n "spec"` 多行形此前被劈成两截不可用语句，提取与变换双面死。
5. **页 HTML wptserve 变量替换**（testharness_html_semantics 主文档通道首接
   `apply_wpt_substitutions`）：`{{domains[www1]}}` 字面残留曾直进 import spec →
   图取回必败（ZW_SHAPED_ADVANCE_TRACE 探针实证 DepFetch + token URL）。
6. **端口替换值对齐上游真值**：`{{ports[http][0]}}` 80→**8000**、http[1] 8000→**8081**、
   https[0] 443→**8444**、SW fixture 专用替换同步——上游 get-host-info.js
   `http:[8000,8081] https:[8444,8443]`。[0] 必须非默认端口：真 WPT 下字符串字面量
  （`"…:8000/"`）与 `new URL().origin` 序列化形（非默认端口保留）两判据形殊途同归，
   默认端口值令 URL 序列化剥端口、同页内 expected 无 :80 / got 带 :80（owco、
   same-origin remote_remote、descendants 臂实测）。SW fixture origin 校验同步 :8444
   （SW 注册 URL 走共享替换，脚本体与校验须同值防脑裂）。
7. **policy 链逐臂过滤**：`pipe.or(doc).filter(valid)` 会把合法 doc policy 连带丢弃
   （descendants invalid-policy 臂 expected "" got 全 URL 实测）→ 改 pipe/filter →
   doc/filter → default。
8. **export-from 再导出补进图收集提取**（es_module
   `extract_static_module_imports_with_attributes` 新增 export 臂）：此前仅 import
   语句（s105 口径注释自认）→ 纯 re-export wrapper（`export { referrer } from '…'`）
   的 checker 依赖从未入图 → 链接期 "does not provide an export named referrer" →
   顶层模块死 → tests:0 Timeout（referrer-strict-policies 实证；DBG 探针打印注册表
   仅 2 wrapper 键定谳）。三 wrapper（import + 局部 const 导出形）因导出表直查命中
   幸免——形态差异即病灶对照。s107 pin 契约同步修订（export-from 产出
   `("./m.json", None)`；`with` 属性解析 FIXME 挂账；动态 import() 仍不收）。
9. **SCRIPT.referrerPolicy 反射门**（part04 R5009 片 e tag 门 + SCRIPT）：limited-to-
   known-values 反射枚举 invalid → ''（script-referrerpolicy-idl 判据页）。

## 验证

- 定向 `make testharness-html-semantics FILTER=referrer`：**0P+15F+12T → 41P+5F+6T**。
  6 个 referrer 页 + descendants 9/9 + strict-policies 全绿；script-referrerpolicy-idl
  1F→P。余 5F6T 全归属挂账面（见下）。
- 全量 A/B（/tmp/s115-full-suite.log → /tmp/s116-full-suite.log，行级归一 diff）：
  **6565P+1202F+111T → 6606P+1192F+105T（+41P −10F −6T）；真翻红（基线 Pass 行 ×
  当轮 Fail/Timeout 行交集）= 0**。+41P 全落 referrer 族 + idl 页。两处红形状漂移
  均定谳为本切片修复的良性后果：
  - `script-crossorigin-network` F→T：基线树 `cross.port = {{ports[http][1]}}` 裸
    token 落 JS 代码位 = 语法错误（页级 Fail 行 = 脚本抛错路径）；页替换修复后页面
    首次真正运行、8 个 async_test 创建后等待 credentials 断言（cookies 基建挂账面）
    → Timeout。页内二分实证（字面 8000 / 注 cookie 行 / 禁 4-8 均仍 T——挂起与
    端口值无关）。页本身零绿损失。
  - `css-module/redirect-relative-urls` T→F：页替换 + 图收集修复后新断言面暴露
    （动态 CSS module 重定向相对 URL 面）1F，页从 tests:0 变为可执行。
- pin 面新增/修订：`module_referer_policy_table_s116`（八策略 + 降级 + 端口归一 +
  origin 形）、`referrer_checker_endpoint_and_host_alias_s116`（七臂：别名 :8000、
  meta、pipe origin、默认 SWCO 全 URL、wrapper 体替换 :8000、降级、invalid-pipe
  顺延 doc）、`meta_referrer_policy_extract_s116`、
  `test_split_statements_multiline_import_from_continuation`、
  `test_export_from_reexport_chain_absolute_dep_s116`、
  `test_attributes_extractor_includes_export_from_reexport_s116`。sandbox lib
  **376/0**、runner bin **243/0**（76 ignored）。
- 质量门：shim 拼接语法 OK（真实装配序 part01+01b+02..07，`node --check`）；fmt 全过；
  `make guarded-clippy` workspace 过；`make test` 三跑收口（/tmp/s116-make-test{,2,3}.log）：
  首两跑 image_batch_e2e 1F——target-disk-guard 根因（全量测试构建堆 target/ 至 76GB
  超 50GB 阈值 → tier-3 全量清空 → make test build 行不含 zero-browser）；第三跑
  （target/ 重置 33G）90/91 suite 绿，唯一失败 zero-renderer lib
  `mutation_arm_does_not_apply_across_document_reset_window` 满载时序 flake
  （2026-10-25 aa66d1530 引入，滞留窗观测断言：满载 340.93s / 单独 1.63s 绿 /
  守卫包 renderer lib 全套 352/0 绿）——预存 flake 非本切片面，挂账。

## 挂账（本切片新增）

- **www2 host 可达性**：CORS 语义 + cookies 基建前置（credentials 12F、css-module
  credentials 臂同面；放行需同批复核 dynamic-imports-fetch-error 判据翻转）。
- **modulepreload referrerpolicy 族**（modulepreload-referrerpolicy /
  -cross-origin- / -inline-，3F+2T）：echo-referrer.py server-state 端点 +
  `link rel=modulepreload` 取回面 + link.referrerPolicy。
- **module-import-referrer.html**（T）：top-level await 面未建模 + 动态 import()
  referrer 语义（inline=文档 URL、external=importer URL）。
- **text/css/json-module referrer-policies**（3T）：数据模块（type:text/css 等）checker
  体形态 + CSSOM 断言面——并入既有 css/text 数据模块语料缺失待决（须用户拍板）。
- **image-loading-lazy-referrerpolicy-change**（1F，前序已有）：img 懒加载推迟取回
  按最新 referrerpolicy 面。
- **script-crossorigin-network**（F→T）：cookies/credentials 基建（页内 8 断言全依赖
  cookie 随跨源脚本请求携带）；挂起机制 = 测试创建后等待。
- **css-module/redirect-relative-urls**（1F，新暴露）：动态 CSS module 重定向后相对
  URL 解析基。
- export-from `with {…}` 属性解析 FIXME（提取器 None 顺延）。

## 遗留近似

- split_statements `from` 续行守卫：行尾 `from` 后接行注释再接字符串的形态不覆盖
  （语料无此形）；`statement_ends_with_from` 粘连标识符负例已测。
- 页替换面 64 token 页分类（s115 基线 3 绿/15 红/46 无模块支持）——A/B 实证零外溢。

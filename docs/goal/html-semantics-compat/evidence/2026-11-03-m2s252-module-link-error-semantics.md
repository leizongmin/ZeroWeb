# M2 s252 — module 链接/instantiation 错误语义（4 键 F→P，第十一个连续完美 A/B）

日期：2026-11-03。切片号 s252（承 s251 候选 #1）。根：s87 注记明文预账
「instantiation/link 期失败上游语义为逐次新对象，transform 管线暂并入
③——WPT 判据 instantiation 行仍红，归 module 基面切片」。

## 入场定刀位

s251 后对 module/dynamic-import 族全景分型：24+ F 键 7 簇
（defer-eval ×3 / credentials ×12+1T / script-error ×2 / delay-load
×1 / shared-dep ×2 / propagate-nonce ×1 / importmap ×4）。本切选
**instantiation/link 错误语义 4 键**（静态 instantiation-error-1/2
消息身份 + 动态 dynamic-imports-script-error 分类 ×2，一个根）；
其余簇挂账不动（credentials 重切片、defer 语法、有状态端点）。

## 勘根（registry 双键现实 + 消息身份分裂）

- **探针定谳双键**：runner pipeline 把外链模块体相对 import 重写为
  路径绝对形（`/html/.../missing-export.js`），模块体内层 import 保
  raw 形（`./missing-export.js`）——同一模块在 registry 出现**双拼写
  双键**（临时探针 eprintln 实证
  `keys=["/html/.../missing-export.js", "./missing-export.js"]`）。
- **静态面失败签名**：direct 项（entry 体自含失败 import）经
  `resolve_registered_specifier` raw-hit 命中路径绝对键 →
  `ensure_module_export` 消息带 `/html/...` 形；nested 项（entry 副作用
  导入，失败在依赖链内层）raw-hit 命中 raw 键 → 消息带 `./` 形。
  WPT 判据 `assert_array_equals(log, [msg,1,...,msg,5])`（msg=log[0]
  自引用一致性）→ 双形消息即红。
- **动态面失败签名**：`compile_dependency_iife(raw_spec)` 同构 +
  `Err(CompileError) => String::new()` 空契约 → TypeError「Failed to
  fetch module」逐次新——WPT 判据 SyntaxError + differentErrorObjects
  双红。
- **对照面**：instantiation-error-3（不可解环）/4（已错图传播）已
  Pass——环检测与错误传播机制在，缺的是跨拼写归一与链接失败分类。

## 施工（Fix A abs 键归一 + Fix B tag 3 契约）

- **Fix A（registry abs-only）**：`collect_module_deps_recursive`
  注册键 raw spec → **解析后 URL**（`registry.register(&url, …)`——
  主路径 + json/css/text 合成臂 ×3）；s101 环臂「按当前拼写补注册」
  删除（跨拼写查找由 `resolve_registered_specifier` URL-join 臂统一
  归一到同键，钻石依赖/自导入语义自然覆盖）；`fetched` 账本参数
  全删（唯一读点随环臂删除而孤立）；`compile_module_contract` 入口
  编译键 raw → `resolve_document_url(parent, spec)`。
- **Fix A 波及面补刀（s101 钉红实证）**：静态入口转换基
  `compile_module_script(&code, url=…)` 旧传**页面 URL**——abs 键
  口径下入口体 import 经 URL-join 归一，基错即 join 落页目录 → 假
  `Module not found`（`external_module_dep_base_is_module_url_s101`
  先红后绿）。修正：入口转换基 = `module_dep_base`（外链 = 模块自身
  绝对 URL；内联 = 页面 URL），与收集基同源。
- **Fix B（tag 3 链接失败态）**：契约新增
  `3\u{1f}resolved\u{1f}message`——`Err(ScriptError::CompileError)`
  → tag 3（其余 Err 维持空串）。三消费方：① webview
  `compile_module_contract` 产；② es_module prelude
  `__zw_load_module` 在缓存/失败表读取**前**插臂（`sep === 1 &&
  charCodeAt(0) === 51`）→ 同步 throw `new SyntaxError(message)`
  （`__zw_dynamic_import` try/catch 转 rejected promise；逐次新对象
  不入 `__moduleCache`/`__moduleFailures`）；③ v8_runtime 原生回调
  `strip_prefix("3\u{1f}")` 臂 → `Exception::syntax_error` reject
  （先于 `__moduleCache` 记忆化读）。s87 ③ 旧口径「instantiation
  暂并入」翻案独立成态 ④；未发标记宿主（renderer/tab）tag 3 不产，
  优雅退化不变。
- **s250 动态 src module 臂（shim）**：tag 3 落 ③ 形执行——体为
  `resolved\x1fmessage` 非 JS 文本，eval 抛 SyntaxError → window
  报告 + load（链接失败 = 取回成功后失败，spec 事件语义同向；
  FIXME 记报告消息未走 tag 3 message 通道）。注释注记，行为零改动。
- 附注：abs 键下 raw-hit 臂不再遮蔽 import map（map 阶段按 spec 序
  真正生效）——全树零位移证明存量语料无依赖旧序的页。

## 验证

- 单测：s87 身份钉扩 **四态**（新增 `linkfail.js` tag 3 臂 →
  `SyntaxError:false` 逐次新对象）+ v8 原生回调 tag-3 新钉
  （`test_native_dynamic_import_link_failure_s252`：SyntaxError ×
  两次 import 新对象 + 消息穿传）。script-sandbox 379 全绿 +
  webview 835 全绿（s101 钉中途红一轮 → 入口基补刀后绿）。
  fmt/clippy 干净（仅 video_registry 既有 6 warning 非本轮面）。
- solo（FILTER=scripting-1/the-script-element，RAF frame-driven +
  test-guard，evidence/logs/s252-solo.log）：**988P/75F/25T/5N**
  （对 s251-solo 984P/79F/25T/5N：ΔP=+4、ΔF=−4，F/T/N 零位移）。
  键级 join：**4 F→P 全为目标键零回归，LOST=GAINED=0**。
- 全树 A/B（evidence/logs/s252-ab.log vs s251-ab.log，make
  testharness-html-semantics 无 FILTER）：行级 7785P/447F/77T/11N
  （ΔP=+4 ΔF=−4）。键级 join **8317/8317 对，迁移恰 4 键全 F→P
  （四目标），LOST=GAINED=0，P→F 0，F/T 零位移——第十一个连续
  完美 A/B**。
- 对照面终态：instantiation-error-3/4 维持 Pass；6/7（星号导出歧义
  blame 模块）仍 Fail——失败签名 = log 序列形状（歧义检测缺失：
  每元素各报错误 + 无单错归并），真机制缺口非消息身份，挂账。

## 挂账更新

- s87 ③ 注记翻案收口（instantiation/link 语义独立成态 ④，
  prelude/v8 两侧同实现）；dynamic-imports-script-error 页余 8 键
  维持 Pass 不动。
- 新挂账：instantiation-error-6/7 星号导出歧义检测
  （`module_provides_export` 扩展：`export *` 链歧义归并 + blame
  模块定位）。
- 其余历史挂账延续：defer-eval ×3（import defer 语法）、credentials
  ×12+1T（cookie jar 基建重切片）、delay-load ×1（window load 时
  序）、shared-dep ×2（serve-custom-response.py 有状态端点）、
  propagate-nonce ×1（CSP eval 包装内部模块求值豁免）、importmap
  ×4（`_importMeta.resolve`）、charset ×6 Timeout、
  http-accept-header ×2、CSS module 基 URL/复用面、referrer-policies
  ×3、s97 insertBefore inline-module 面、动态 src module specifier
  解析失败 window TypeError 报告面（shim tag 1 空码臂 FIXME）+
  tag 3 message 通道 FIXME、in-order 队列 FIXME（async=false 动态
  模块按 asap）；customizable-select 177F 挂起待用户拍板
  （GB-20261031 征询在途）。

## 下一步候选

1. propagate-nonce ×1（CSP `script-src 'nonce-correct'` 页 meta
   面——宿主 eval 包装误伤内部模块求值通道，nonce 检查豁免内部
   compile 通道；小刀位）+ delay-load ×1（window load 时序）。
2. importmap ×4（`_importMeta.resolve` 特性面——import map 解析
   已在 `resolve_registered_specifier` 就位，补 JS API 面）。
3. instantiation-error-6/7 星号导出歧义检测（transform 管线内
   export * 链歧义归并 + blame 定位，中刀位）。

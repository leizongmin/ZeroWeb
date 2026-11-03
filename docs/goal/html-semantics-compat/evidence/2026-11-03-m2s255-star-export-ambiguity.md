# M2 s255 — 星号导出歧义检测 + module 求值错原值报告 + 注册序确定化（3 键 F→P）

日期：2026-11-03。切片号 s255（承 s254 候选 #1）。目标三键全 F：
instantiation-error-6/7（ambiguous star exports → instantiation error
+ 正确模块 blame）与 error-type-3（instantiation error 较 evaluation
error 优先 + 原值抛出对象断言）。

## 勘根（四根因，探针 + Node oracle + 代码链定谳）

1. **歧义检测缺失**：`module_provides_export` 布尔归并——星号导出
   同名经多路径可达 / 子链歧义一律 `true`，链接期不报。tc39
   ResolveExport 的歧义语义（star 臂子解析 null → 整体 null）完全缺
   位。Node 24 oracle 实证两形：6 形（6b 双星号臂 6c/6d 同名 foo →
   SyntaxError）与 7 形（7a→7b→7c→{7d,7e} 子链歧义 + 兄弟臂 7f 可消
   解**不豁免**，仍 SyntaxError——歧义经星号传播）。
2. **star/re-export 内联无记忆化**：`transform_export` 三处 Fresh
   臂内联裸 IIFE 不走 `memoized_dep_expr`（R5182 只改了 import 侧）
   ——prelude 急切臂已缓存后仍重跑。ierr-6 红态 got 序列 6c/6d 各
   双份实证。
3. **求值错报告面双折损**：module 体抛错走
   `script_report_module_failure` → `reportError(new
   SyntaxError(msg-text))` 丢原值对象（error-type-3 `log[1].foo` 对
   `throw {foo:true}` 断言必挂）；且 shim `reportError` 自身
   `error: (instanceof Error) ? reason : null` 对非 Error 值二次折
   null——spec report the exception 对任意 throwable 原值透传。
   https://html.spec.whatwg.org/multipage/webappapis.html#dom-report-error
4. **prelude 注册序哈希化**：急切注册臂 `registry.specifiers()` 为
   HashMap 迭代序——模块体副作用顺序（可观察面）不确定。ierr-7 红
   态实测 got [7e,7d,7c,7f,7b]（哈希序），spec 要求依赖图
   depth-first post-order（[7d,7e,7c,7f,7b]——恰好后序）。
   https://tc39.es/ecma262/#sec-moduleevaluation

期望序列（语料判据 + spec 事件面）：ierr-6 =
[exn(SyntaxError), 1, "6c","6d","6b", 2]；ierr-7 = [exn, 1,
"7d","7e","7c","7f","7b", 2]（7b 自身可链接——星号歧义只影响导入方
7a，故 7b evaluate 后照 log；7a 链接失败不 evaluate）；error-type-3
= ['throw', {foo:true}, 1, SyntaxError, 2]（元素 1 求值错原值上报 +
fire load——fetch 成功后的失败照 load，仅网络失败 fire error；元素 2
链接错 SyntaxError）。

## 施工（四刀 + 2 solo 暴露修复刀）

script-sandbox（es_module.rs）：
1. **A** `ExportResolution` 三态枚举（Provides/Ambiguous/Missing，
   tc39 ResolveExport 静态近似）——星号臂收集 `star_provides` 计数
   + `star_ambiguous` 传播旗；显式重导出臂仅 Provides 提前返回（歧
   义/未命中继续）；尾归并 `star_ambiguous || star_provides >= 2 →
   Ambiguous`。`ensure_module_export` Ambiguous →
   `CompileError("Module {spec} contains conflicting star exports for
   name '{name}'")`（webview 侧 CompileError 统一映射 SyntaxError
   报告面，s102 通道）。
   https://tc39.es/ecma262/#sec-resolveexport
2. **B** `transform_export` 三处 Fresh 臂（star-as / star-from /
   re-export-list）裸 IIFE → `memoized_dep_expr`（R5182 import 侧
   同型——急切注册/入口体/内联三路共享实例）。
3. **E**（solo 首轮暴露）`module_graph_postorder`：prelude 急切注
   册序从 registry 哈希序改为入口图 depth-first post-order（依赖
   提取复用 transform 同款 helper：import/export-from 的 module
   request，源内出现序；动态 import() 非链接依赖不入；未注册依赖
   与解析失败说明符跳过——定序尽力而为，错误报告仍走 build 路径；
   入口图不可达余量按字典序追加保持全量急切注册覆盖、仅定序）。

engine（script_gen.rs + js_dom_shim/part01.js）：
4. **C1** `script_module_eval_error_report(script_ordinal)`：JS 侧
   取 `__zwModEvalErrObj` 哨兵 → `reportError(原值)` → 元素照常
   dispatchEvent('load')。
5. **C2** shim `reportError`：`error:` 非 Error 折 null → **原值透
   传**（spec 语义；全仓两调用方——s102 臂恒 Error 实例不受扰，
   C1 臂即本切片消费方）。

webview（webview.rs）：
6. **C3** module 编译成功臂 try/catch 包裹（`{prelude}\n
   {transformed}` 抛错 → 哨兵暂存**原值对象**——execute 的 Rust Err
   只剩文本）；包装级 parse 失败整体仍 Err → s102 臂照旧。
7. **C4** post-check：module 面执行 C1 取哨兵上报；返回非空 =
   求值抛错已报告 → s107 成功 load 臂跳过（防 load 双发）。

runner（testharness.rs）：
8. **D** module 投递单元 `data-zw-unit` 标记 + 尾片按属性查回元素
   dispatchEvent（弃 `document.currentScript`——module 体恒 null，
   spec：仅 classic script 非空；弃序号口径——JS 字符串内
   `<script` 文本会令计数膨胀）。附带面：404 module 单元 error 尾
   从死代码变活（fetch 失败 onerror，spec 正确方向）。

## 验证

- **单测**：script-sandbox **384 全绿**（381 基线 + 3 新钉：
  `test_star_export_ambiguity_s255` 三形——6 形 CompileError 断言 +
  7 形传播 + 单星号路径 over-block 对照；`test_star_inline_dep_
  evaluated_once_s255`——依赖体恰好一次；`test_prelude_registration_
  postorder_s255`——注册序 b,c,a）。engine **3714 过 + 5 失败 =
  HEAD 既有对端账**（js_dom_bridge CSSOM 5 案，82153965c 提交消息
  在账；stash A/B 定谳——HEAD 同挂 5 失败，非 s255 回归）+ webview
  **835 全绿**。fmt/clippy 四 crate（script-sandbox/engine/webview/
  runner）零告警。
- **solo 终态**（守卫终态代码，evidence/logs/s255-solo-ierr.log +
  s255-solo-errtype.log）：instantiation-error **7P+1T**——ierr-6/7
  F→P（8 号 Timeout 既有不动，红态同形在账）；error-type **3P**——
  error-type-3 F→P。恰 3 键迁移与目标键全等。
- **中间红态两轮**（记档供排查方法论）：首轮 solo ierr-7 仅 7d/7e
  顺序颠倒 → 暴露根因 ④ 哈希注册序 → E 刀；二轮 error-type-3 仍
  `log[1] null` → 暴露 shim reportError 折 null（根因 3 第二折点，
  C2 刀）。红态断言文本逐字比对定位折点，未走加日志盲猜。
- **全树 A/B**（evidence/logs/s255-ab.log vs s254-ab.log，窗口独占
  ——s254 教训执行：运行窗口零并发编译/测试负载）：行级
  7793P/440F/77T；键级 (key,state) 对 **8303→8307（LOST 5 / GAINED
  9），迁移恰 3 键 = 三目标键全数 F→P**（error-type-3 +
  instantiation-error-6/7——与 solo 终态全等）。三处页级完成度位
  移（无一为 s255 语义回归）：
  ① focus-dynamic-type-change-on-blur：文件级 Timeout 线消失、页面
  跑满（+1 子测试 Pass「text -> week false」）——在册抖动族（s254
  轮文档），本轮向完成方向摆动；
  ② select-home-end-pagedown-pageup.optional：文件级 Fail → 子测试
  级惯常 4 Fail（s251 起在档的同形基线态）；
  ③ microtasks/evaluation-order-1：文件 Timeout 截断线内首次到达
  「Module script queueing a microtask」→ Fail（solo 复现同断言
  ——expected ["body","microtask"] got ["body"]，与页内长期在账的
  「Classic script queueing a microtask」同机制同形：microtask
  checkpoint 排空时序，非模块图语义；s252/253/254 三轮同页恒 2 子
  测试 + 文件 Timeout，本轮 B 刀去重复求值页面变快截断后移掩盖解
  除）——归因：**新可见既有失败**，非回归。

## 挂账更新

- **renderer/tab 宿主 import.meta.resolve 接线**（承 s254 不动）。
- **import maps 剩余语义面**（承 s254 不动：多张 map / 动态插入 /
  Blocked 降级）。
- **microtask checkpoint 排空时序族（新记档）**：evaluation-order-1
  「Module script queueing a microtask」本轮截断掩盖解除后首次在
  A/B 可见——与页内 classic 变体长期在账同机制（microtask 排空
  晚于断言步）；classic/module 两变体同修面。
- **data-zw-harness 投递标记保真面**（承前不动；D 刀已按 unit 标记
  查回，保真面收窄为 harness 脚本注入序）。
- **s250 动态内联 module 臂 eval 通道**（承前不动）。
- **instantiation-error-8 文件级 Timeout**（既有，红态同形；s227
  教训——先验语料资产存在性再勘根）。

## 下一步候选

1. 动态 import 异步通道（delay-load 根因切片；顺带覆盖 trickle 类
   慢模块时序族）。
2. renderer/tab 宿主 import.meta.resolve 接线（小刀，补多进程面对
   称）。
3. data-zw-harness 投递标记保真面（D 刀后续收窄）。

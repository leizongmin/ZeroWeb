# M2 s260 — import map Blocked 终止跨面接线（resolve_mapped 三态化；webview/宿主/prefetch 一次收口）

日期：2026-11-03。切片号 s260（承 s259 挂账——「webview/宿主编译面 Blocked
终止（resolve_mapped 三态化 + resolver 错误通道 + compile contract 失败
tag 路由——三面一次收口）」，s259 下一步候选 #1）。

## 勘根

1. **Blocked-bypass**：`resolve_mapped` 返回 `Option<String>` 签名把
   Blocked（null 条目）塌缩为 None——与 Miss 不可区分。各接线侧一律按
   Miss 处理后走 parent-join 继续抓取：页面显式以 null 条目阻断的 URL
   只要可取回就照常装载。规范「resolve a module specifier」：解析命中
   null 条目即失败，不得回退任何解析路径。
   https://html.spec.whatwg.org/multipage/webappapis.html#resolve-a-module-specifier
2. **五面接线清单**（Blocked 须各自落到既有失败词表）：
   - webview shim `__zw_compile_module`（webview.rs ~4250）与 native
     resolver（~4279）——都经 `apply_import_map`；空契约 = TypeError
     拒绝面（s256 契约词表；与 CSP violation 拒绝同通道）。
   - renderer `collect_module_deps`（多进程依赖收集）——解析失败返 Err
     即编译终止。
   - renderer `__zw_compile_module` handler（运行时动态 import 面）——
     返 `String::new()` → v8_runtime `settle_dynamic_import` TypeError
     reject（"Failed to fetch module: '…'"）。
   - tab_js_worker 同形两处（进程内单进程面，与 renderer 跨 crate 无共
     享面，独立实现）。
   - script_prefetch 静态依赖展开——Blocked 跳过（解析将在编译面终止，
     预取被阻断 URL 属噪声抓取）。
3. **不涉面**：宿主 `__zw_importmap_resolve`（import.meta.resolve 回调）
   Blocked → 空串 → prelude TypeError 守卫臂 s257 已规范正确，本切片不
   动；es_module/compile 面 s259 已终止，不动。
4. **红态可行性**：renderer/tab collect 面可在 HEAD 编译红态钉（fetch
   闭包记录调用，断言被阻断 URL 不得抓取）——HEAD 实证抓取发生（bypass
   在账）。webview 面不可行：`apply_import_map` 签名随修复而变（编译失
   败不冒充行为失败），且 `compile_module_contract` 自由函数根本不接触
   map（map 只在两个回调点应用）——红态证据由同形塌缩的 collect 钉承
   担，webview 三态钉作为签名变更同伴后补。

## 施工（5 刀 + 5 钉）

1. **三态刀**（import_map.rs `resolve_mapped`）：`Option<String>` →
   `ImportMapResolution`（Resolved/Blocked/Miss）。base 解析失败仍返
   Miss（既有守卫臂）；match_map_tables 内部本就三态，直报不再拍平。
   模块文档「已知偏差」#2 更新为全面终止口径。
2. **webview 刀**：`apply_import_map` → `Option<String>`——映射命中
   `Some(resolve_document_url(base, mapped))`、Blocked → `None`、未命中
   → `Some(spec)` 原样（fetch 侧按 referrer join，本函数只改写映射命
   中）。两调用点 let-else → 空串契约（shim 契约空串 = TypeError 拒绝
   面；native resolver 空串 = v8_runtime TypeError reject）。
3. **renderer 刀**（js_worker.rs）：新增 `pub(crate) resolve_import_dep`
   共享 helper（映射/Blocked None/未命中 join 三态）；`collect_module_deps`
   let-else → `Err("module specifier '…' is blocked by a null entry in
   the import map")`；`__zw_compile_module` handler let-else →
   `String::new()` + tracing::warn。
4. **tab 刀**（tab_js_worker.rs）：本地私有同形 helper（注明与 renderer
   跨 crate 无共享面）；collect Err + handler String::new() 同 renderer。
5. **prefetch 刀**（script_prefetch.rs）：静态依赖展开 let-else →
   `continue`（跳过被阻断 URL）。
6. **钉面 ×5**：
   - `collect_module_deps_blocked_entry_terminates_s260`（renderer）+
     `tab_collect_module_deps_blocked_entry_terminates_s260`（tab）——
     红态钉：HEAD 抓取被阻断 URL 且收集成功（"被阻断依赖应以解析错误
     终止: ()"），修复后 Err + 不回退抓取。
   - `resolve_mapped_null_entry_is_blocked_not_miss_s260`（import_map.rs，
     签名变更同伴——旧 Option 签名下无法编译）——精确键与 URL 形态键
     两臂 Blocked 直报。
   - `resolve_import_dep_trinity_s260`（renderer helper 三态 + 无 map
     passthrough 臂）。
   - `apply_import_map_trinity_s260`（webview，签名变更同伴）——映射/
     passthrough（原样非 join）/Blocked None 三臂；passthrough 断言先
     写 join 期望红过一次，勘正为原样契约（fetch 侧 join）。
   - 既有钉适配：renderer `parse_page_import_map_first_valid_wins` 的
     `resolve_mapped` 断言、import_map.rs `resolve_mapped_only_reports_
     actual_matches`（None → Miss）与 s258 归一化钉的 resolve_mapped 臂。

## 验证

- **红态实证**：2 collect 钉先于修复在 HEAD 跑（renderer 与 tab 双面
  同错）——被阻断 URL 被抓取、收集成功。旧版因目标行为失败证据在账。
- **绿态**（test-guard 守卫）：
  - `cargo test -p zero-script-sandbox` **393 全绿**（392 基线 + 1 新钉）。
  - `cargo test -p zero-renderer` **371 全绿**（370 基线 + 1 新 helper
    钉；2 红态钉翻转）。
  - `cargo test -p zero-webview` **836 全绿**（835 基线 + 1 三态钉）。
    期间 `proxy_mode_decodes_png_correctly` 红 = target-disk-guard 清理
    删 `target/debug/zero-image-decoder` 产物（`current_exe` 上溯找不到
    可执行文件），重建后单跑绿——环境缺产物非回归。
  - `cargo test -p zero-browser --features v8 --bin zero-browser`
    **740 绿 + 7 红**：5 个为 s257 已知 HEAD 红（compositor reaping /
    service-worker 多进程重启 / default_actions / cpu-gpu matrix /
    paste_and_go）；2 个（`compositor_frame_decodes_page_primitives_
    and_metadata`、`page_context_menu_save_as_writes_page_html_to_
    download_dir`）全量并行跑红、**单跑均绿**——子进程并行负载 flaky
    非回归（两测试不触及 import map 面）。新 tab 钉绿。
- **fmt/clippy**：`cargo fmt` 四 crate 过；clippy --all-targets
  script-sandbox/webview/renderer + browser 双形态（v8 / default）零告警。
- **消费面覆盖核对**：`grep resolve_mapped` 生产调用仅三处（renderer
  helper / webview apply_import_map / tab helper）——全部本切片改造；
  prefetch 经 renderer helper 间接覆盖。
- **无全树 A/B**：corpus 零 importmap 面（s258 grep 证据承用）→
  runner 通道确定性零位移；Blocked 臂只在 null 条目存在时触发，无 map
  页面路径逐字节不变。

## 挂账更新

- 承前不动：import maps 剩余语义面（多张 map 整页失败语义 / 动态插入 /
  键排序序偏差）；data-zw-harness 投递标记保真面（s255 候选 #3）；
  credentials 面 / shared-dependency 取回计数 / microtask checkpoint
  时序族 / 并发慢 import dedup（s256）；customizable-select 177F（挂起
  待用户拍板 GB-20261031 征询在途）；renderer SetDomSnapshot entries 化
  （s174，低优先）。
- 传递依赖面备注：map 只在顶层契约入口应用（传递依赖从不看 map）——
  既有形态，Blocked 在依赖链的终止由各层 collect 的 Err 传播承担；若
  后续 WPT 面要求传递依赖可见 map，另立切片。

## 下一步候选

1. data-zw-harness 投递标记保真面（s255 候选 #3 承前）。
2. import maps 剩余语义面（多张 map 整页失败语义 / 动态插入 / 键排序序）。
3. s255 候选清单其余（见 s255 账页）。

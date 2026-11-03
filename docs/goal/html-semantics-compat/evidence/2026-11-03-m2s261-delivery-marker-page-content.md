# M2 s261 — data-zw-harness 投递标记保真：页面内容单元 `data-zw-inlined` 分型 + CSP 检查点按原始 src 走外链 URL 路径

日期：2026-11-03。切片号 s261（承 s260 下一步候选 #1 = s255 候选 #3
「data-zw-harness 投递标记保真面」——s253 新挂账「CSP 页外链页脚本整体豁免
eval 门禁」错进错出面的收口）。

## 勘根

1. **面定义（错进错出）**：runner 本地投递 `inline_local_scripts` 把 case
   引用的 helper .js / 页面外链脚本整体内联投递，此前页面内容单元与基建
   （harness/testdriver/inline_extras）统一打 `data-zw-harness`。webview
   CSP script 检查点（security-hardening sh1-m2s1）与 eval 门禁包装安装门
   （s253）都以 `!zw_harness` 为豁免门——CSP 页的**所有**外链页脚本借基建
   标记逃逸全部 `script-src` 约束。真实浏览器对外链脚本做源匹配（'self'
   同源放行 / nonce / 异源阻止），本引擎「错进错出」。
2. **通道盘点**：CSP 检查点豁免门（webview.rs `run_page_scripts` 的
   `!zw_harness`）；eval 包装安装门（webview.rs ~4785，同 `!zw_harness`，
   s253 通道）；`data-inline` 属性投递格式既有（记录原始 src）但 webview
   侧无人消费。
3. **红态可行性**：runner 投递面钉（`prepare_harness_html` 纯函数，HEAD
   实证页面单元打 harness 标记）；webview 检查点钉三枚自包含（不依赖
   runner，HEAD 实证同源原始 src 被内联 hash 路径阻止 / 异源原始 src 被
   'unsafe-inline' 放行执行）；extract 签名同伴钉（旧 `ScriptElementCsp`
   无 `inlined_src` 字段不可编译——编译失败不冒充行为失败）。

## 施工（3 面 4 文件）

1. **runner 投递刀**（testharness.rs `inline_local_scripts`）：页面内容
   单元改打 `data-zw-inlined`（`data-inline` 继续记录 case 目录归一后的
   语料相对原始 src）；harness/testdriver/inline_extras 基建块保持
   `data-zw-harness`。
2. **extract 刀**（engine extract.rs `ScriptElementCsp`）：+
   `inlined_src: Option<String>`（`data-inline` 提取，trim + 空滤），与
   nonce / zw_harness 同在全量 script 文档序向量对齐。
3. **webview 检查点刀**（webview.rs）：`external_src.or(inlined_src)`——
   内联投递页面脚本按原始 src 走既有外链臂（`resolve_document_url` 绝对
   化 + `check_script` URL 路径，nonce 照常参与）；纯内联脚本 hash 臂不动；
   `data-zw-harness` 基建豁免不动。eval 包装门无需改动——豁免门只认
   harness 标记，标记翻转后内联页脚本自动进入包装覆盖（伴随钉护轨）。
4. **钉面 ×4**：runner 投递保真钉 `delivery_marks_page_units_inlined_
   not_harness_s261`（基建保持 harness / 页面单元 inlined + 原始 src
   记录 / 单元无 harness 标记）+ webview 三钉 `csp_gate_inlined_page_
   script_url_path_s261`（同源放行零 violation）、`csp_gate_inlined_page_
   script_cross_origin_blocked_s261`（异源阻止 + blockedURI 绝对 URL，
   csp_gate_external_source_match 镜像）、`csp_eval_gate_wraps_inlined_
   page_script_s261`（eval 包装伴随钉——护轨非红态，红态证据承 runner 钉）。

## 验证

- **红态实证**（HEAD，stash 法隔离双侧）：
  - runner 钉 FAIL（页面单元打 harness 标记、缺 `data-zw-inlined`）。
  - webview 同源钉 FAIL（`__ran`=undefined——原始 src 同源脚本被内联 hash
    路径阻止）；异源钉 FAIL（`__blocked_ran`=1——整体豁免放行执行）。
  - eval 伴随钉 HEAD 即绿（包装门本就只认 harness 标记，与「护轨非红态」
    设计一致）；extract 同伴钉旧签名不可编译（编译契约钉）。
- **绿态**（test-guard 守卫）：
  - `cargo test -p zero-webview --lib` **839 全绿**（836 基线 + 3 钉）。
  - `cargo test -p zero-wpt-runner --bin zero-wpt-runner` **282 全绿**
    （281 基线 + 1 钉；76 ignored）。
  - `cargo test -p zero-engine --lib` **3720 过 + 5 失败，与 HEAD 全量
    逐测试同集**（3720+5 == HEAD stash 法 3720+5；5 红 = R-t2qq
    adopted-stylesheets 系列 81b00542d 等区域既有红，见挂账）。
- **fmt/clippy**：`cargo fmt --all` clean（WIP 一处测试断言串续行格式
  落 fmt 修正后复验）；clippy --all-targets engine/webview/runner 零告警。
- **solo security**（evidence/logs/s261-solo-security.log）：**24P/1T**
  与 s248 基线逐位同（1T = pattern.fillStyle.sub 文件级 Timeout，
  SVG/video settle 媒体面在账挂账，非本切通道）。
- **全树 A/B**（evidence/logs/s261-ab.log vs s256-ab.log 基线；同命令
  `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1 test-guard … zero-wpt-runner
  testharness-html-semantics`、同 8 job、binary 经 cargo build no-op
  实证含本切改动）：行级 **7794P/439F/11N/77T 双侧逐数同**；(key,state)
  对 **8318/8318 join LOST 0 / GAINED 0——完美 A/B（零位移）**。
  机理一致：改动臂只在 CSP 页生效——corpus 无依赖整体豁免的钉面（CSP 页
  外链 src 同源过 'self'、nonce 属性投递保留参与检查），非 CSP 页检查点
  与 eval 门禁双不激活，标记翻转无观察面。

## 挂账更新

- **CSP 页外链页脚本整体豁免面（s253 新挂账）销案**；s253「错进错出」
  勘根闭环。
- data-zw-harness 保真面收窄余项：基建 harness 脚本**注入序**（s255 D 刀
  后续，承前不动）。
- 承前不动：import maps 剩余语义面（s260：多张 map 整页失败语义 / 动态
  插入 / 键排序序偏差）；microtask checkpoint 时序族（s255）；credentials
  / shared-dependency / 并发慢 import dedup（s256）；customizable-select
  177F（挂起待拍板 GB-20261031）；renderer SetDomSnapshot entries 化
  （s174，低优先）。
- 新记档：engine lib 5 红（p56_adopted_snapshot_restore_hook /
  test_p99s_rule_style_preserves_custom_property_case /
  test_stylesheet_add_rule_remove_rule_r3276 / test_stylesheets_rule_
  style_r2810 / test_stylesheets_write_r2809）——R-t2qq adopted-stylesheets
  系列（81b00542d 返修2 等）区域 HEAD 既有，与本切无关；归因待该流自查，
  本流不入侵。

## 下一步候选

1. import maps 剩余语义面（多张 map 整页失败语义 / 动态插入 / 键排序序
   偏差——s260 挂账 #2）。
2. data-zw-harness 基建脚本注入序（s255 D 刀收窄余项）。
3. s255 候选清单其余（见 s255 账页）。

slice37 根因排查证据存档（quickjs 腿 zero-webview 4F → 双层根因定位）
日期：2026-10-07（临时诊断代码已全部还原，本目录为观测值文字存档；诊断性 Rust/JS
插桩改动未入库——more_coverage.rs 零 diff 复核）

## 决定性复现

- `cargo test --no-default-features --features quickjs -p zero-webview --lib`
  （solo，无并行）→ 4F 确定性复现：
  - tests::coverage::test_document_current_script_classic_r3258（currentScript NULL）
  - tests::csp_gate::csp_gate_blocks_inline_and_dispatches_violation_sh1_m2s1
    （violation target=document 而非被阻止 script 元素）
  - tests::csp_gate::csp_style_gate_blocks_inline_style_sh1_m2s3（retained=0）
  - tests::more_coverage::test_execute_script_valid_syntax_but_error（is_err=false）
  - v8 腿同二进制 742-test 全绿；基线 quickjs（687b12ecd stash）681/681 绿
- 推翻初版「guarded 并行 flake」定性（manifest 2026-10-06 版），4/4 挂全为
  quickjs 确定性回归。

## 分层观测值（temp_diag 插桩 execute_script 直读）

- 自由变量面：execute("qqzzUndefinedDiag37") → Ok("undefined")（应 Err ReferenceError；
  strict IIFE 内同样 Ok("undefined")）；typeof Window=function、NPO class string
  [object WindowProperties]、gsp gOPD/gPN/hasown 反射面全部正确（miss 报 miss）
- 文档级集合面：document.getElementsByTagName('*').length = 0（'script'/'body'/'html'
  同 0）；对照 querySelectorAll('script').length = 2、
  document.body.getElementsByTagName('script').length = 2、
  String(__zw_query_all_tagged('*')) = 107 字节 5 元素 payload 正常、
  __zw_dom_view_stamp() 正常、元素 e.nodeType=1/number/[object HTMLScriptElement] 正常
- currentScript 插桩日志（setter/cleaner/getter 有序）：S1 G1:undef G1:undef CC
  ——setter 被调（idx=1）、getter 两次均取到 scripts[1]=undefined、然后 cleaner×2；
  根 = 文档级 gETN 空集合，非 setter 未调
- C 层源定位：rquickjs-sys-0.7.0/quickjs/quickjs.c
  - :7428-7443 JS_GetPropertyInternal2 原型链走查 `if (em->get_property) { return
    em->get_property(...); }`（返回值直接当命中，不续链、不抛）
  - :46769 js_proxy_exotic_methods `.get_property = js_proxy_get`（Proxy 在链上即触发）
  - :9863 JS_GetGlobalVar：global_var_obj own miss → 同一 walk → 自由变量解析丢
    ReferenceError
  - V8 对照：变量解析按 gOPD/has 语义判缺失（test 4 v8 恒绿实证）；属性读的链走查
    两引擎一致（spec OrdinaryGet 调父级 [[Get]] → get trap）——第一版探针误测属性读
    续链面，node -e 复现 t.marker37 === undefined on V8 too 后改测变量解析面
- 递归面（gETN 空集合机制）：查询机器内 typeof 守卫读（typeof __zw_query_all_tagged
  等）→ miss 全局读 → 链上 NPO get trap → _zwNPOMainLookup → _zwNPOIfrScan →
  getElementsByTagName('iframe') → _zwDocAllElements → 再 miss 读 → 重入；
  _zwNPOMainLookup 重入门（wired 腿防御）+ quickjs 不接线后消除

## 修复后双腿验证

- quickjs zero-webview lib：681/681 GREEN（原 4F 集全部转绿）
- quickjs renderer lib（--test-threads=1，make test 同款）：218/218 GREEN
- v8 zero-engine lib：2852/2852 GREEN（含 part38 六钉，NPO 全语义面）
- v8 renderer lib：220/220 GREEN（wired 臂断言 NPO 面）

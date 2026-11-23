# DC-2 切片 5 刀③⑤ 证据：trusted-types 名单指令 enforcement（2026-11-23）

## 定位

part14 前刀面只判 `require-trusted-types-for`+`'script'` 子串（FIXME(3b) 挂账），
`trusted-types <list>/'none'` 指令未解析：名单外 createPolicy 不阻断、名单域
重名不阻断。基线残面 10F：TrustedTypePolicy-CSP-no-name（空名单值）1F、
cspTests-case2（'none' 大小写变体）1F、cspTests-noNamesGiven 1F、
cspTests-none-none 2F、cspTests-none-none-name 1F（'Can create SomeName' 基线
即 P——无门时全放行）、inheriting-csp-for-local-schemes（srcdoc 臂）1F、
duplicate-names-list 1F、sandbox-allow-scripts（allowlist 臂）1F、
wildcard-with-trailing-characters（`*X` 无效表达式）1F。

## 交付面（engine js_dom_shim 双件；引擎 Rust 零改动）

- **part14**：`_zwTtEnforced` 子串判据升级 `_zwTtParseCsp()` 指令分词——
  enforced 位（`require-trusted-types-for` 值含 `'script'`，大小写不敏感）
  语义不变；新增名单态 `hasList/lists/allowDuplicates`。单 meta=单政策、
  分号分指令：政策内 `'*'` 优先（名单项忽略）、`*X` 类无效表达式＝无有效
  tt-expression→block 一切、`'none'` 大小写不敏感且独占＝空名单 block 一切
  （与其他表达式并存时被忽略）、`'allow-duplicates'` 置位、其余 token 入
  名单。spec
  https://w3c.github.io/webappsec-trusted-types/dist/spec/#should-block-create-policy
  导出 `__zwTtCspState` / `__zwTtPolicyAllowed`（任一名单政策 block 即
  block；`'*'` 政策不 block；空名单政策 block 一切；'default' 须显式入名单）。
- **part12**：`_zwTtCreatePolicy` 名单门——名单域存在时 `allow-duplicates`
  缺席→重名 TypeError（`_zwTtCreatedNames` 追踪；无指令页保持重名容忍，
  createPolicy-unenforced 语料实证）；名单外名字 TypeError（spec 拒绝文案）。
  装配序 part12 先于 part14，运行时经 globalThis 桥消费解析态（惰性 meta
  扫描缓存与 sink 面共享）。
- iframe/本地 scheme 跨 realm CSP 继承域（srcdoc require 臂、data/blob/
  about:blank 组）不在本刀——影子 realm 面，随 worker 通道刀位（part14
  头注释已划界）。

## 语料对账（/tmp/k35b-tt-full.log → evidence/dc2-knife35-tt-full.log）

- **TT 全量：1528P/562F/64T/17NR → 1538P/552F/64T/17NR（+10P/−10F）**。
  逐行 comm（对称排除 zz-* 探针页，双侧 2099 行恒等）：**10 对 Fail→Pass
  完美翻转闭合、零未知漂移、零 P→F**——翻转集与定位残面逐键对应（上列
  10 键）。TO 键集合 64 键逐键恒等（comm 零行）、NotRun 17 恒等。
- 判读字节绑定：发射前 md5 快照 part12 `386826a9…` / part14 `b471b364…`，
  判读后复验一致；release 重建仅 engine→page-runtime→webview→wpt-runner
  43.54s（TT-EXIT=2＝存在 Fail 的常规退出码，判读以计数为准）。
- 页级探针留痕：前轮 10 页 base/post FILTER 探针（/tmp/k35-*，跨轮部分被
  清）＋终态 3 页 re2 复验（cspTests-case2 1P / none-none-name 2P /
  none-skip 3P）与全量账一致。
- **zero-engine lib unittests 4252P/0F**（test-guard `--compile-first`，
  17.66s）＝③③b/③③a 记录值精确恒等——名单门仅在 meta CSP trusted-types
  指令存在时激活，无指令 fixture 全量零扰。

## 回归网

- diff 仅 JS 双件（+122/−19）；`node --check` 双绿、`cargo fmt --all --
  --check` 绿、`cargo clippy -p zero-wpt-runner --all-targets -- -D warnings`
  绿（12.6s；Rust 面零改动，workspace clippy 不在本刀影响面，沿 knife
  定向先例）。
- enforced 位与 sink 管道零语义变化（`_zwTtEnforced` 委托新解析态）——TT
  全量除 10 翻转外逐行恒等即证；sidecar meta 注入面（刀③④）走同一解析
  通道，注入页翻转已含在上账（sandbox-allow-scripts/inheriting-csp 为
  sidecar 形态页）。

## 挂账（本刀收口后余面）

- iframe/本地 scheme 跨 realm CSP 继承残面：inheriting-csp-for-local-schemes
  残 1F+1T（require 臂在 srcdoc 影子 realm 缺 enforcement）。
- creation-be-blocked-by-csp-001/002/003/005 页级 TO（pending 24/21，完成
  回调未调用）与 cspTests-none 页级 TO——预存债，本刀不动。
- eval 域整体挂账（直接 eval 编译器特判、globalThis.eval 替换破坏局部作用
  域）随引擎宿主桥臂——控制面既有。

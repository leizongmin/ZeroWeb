# DC-2 切片 5 刀③④ 证据：.headers sidecar 17 页 runner 预处理（2026-11-23）

## 定位

trusted-types 语料 45 个 `.headers` sidecar（全 wpt-data 215 个横跨 12 目录），
顶层 HTML 页 sidecar 恰 **17 页**＝控制面口径。runner 无响应头投递——上游
wptserve 按 sidecar 下发 HTTP 头，本 runner 页面级 CSP 从未生效：enforcement
页（CSP 仅来自头）的 TT 门全缺席。基线形：default-policy.html 18F+1T、
empty-default-policy 3F+1T、sandbox-allow-scripts 2F、inheriting-csp 2F+1T、
creation-005/sink-003/navigate-008 各 1T、trusted-types-reporting 零结果行
（iframe/Worker skip 谓词跳过，非本刀面）。

## 交付面（tests/wpt-runner/src/testharness.rs，runner 侧预处理）

- `run_corpus_subdirs_ext` 增 `inject_csp_sidecar: bool` 参数，**仅
  trusted-types 套件入口传 true**（其余 ~15 调用方经缺省委托 false 逐位
  不变；cors 直呼点补 false）——blast radius 控制：全局开启会横跨
  import-maps/subresource-integrity 等 6 个 sidecar 页放大到其他套件账面。
- `inject_csp_sidecar_meta`：`<page>.html.headers` 逐行解析，只取 enforce
  面 `Content-Security-Policy:`（**精确全键**大小写不敏感——不吞
  `-Report-Only` 前缀碰撞；Set-Cookie/Expires 等无关键跳过），head 开标签
  后插入 `<meta http-equiv="Content-Security-Policy" content="...">`
  （词边界判 `<head` 防 `<header` 误匹配；`"` → `&quot;`；无 head 兜底
  文档首前插）。无 sidecar / 无 enforce 政策 → 原文零变化。
  https://www.w3.org/TR/CSP3/#delivery-html-meta-element
- **引擎零改动**：注入复用既有双通道——shim `_zwTtEnforced` 惰性 meta 扫描
  （part14）+ extract.rs `extract_meta_csp_policies`（head 内多 meta 并集、
  任一阻止即阻止）。sidecar 多 CSP 头 → 多 meta → 政策并集语义正确。

## 探针（zz-knife34-probe.html + .headers，临时件不入库）

页面 HTML 无任何 meta CSP、enforcement 由 sidecar 唯一驱动：**4/4 全绿**
（string 双 API TypeError、default policy TT 实例 fire、函数参透传）。

## 语料对账（/tmp/k34-tt-full.log → evidence/dc2-knife34-tt-full.log）

- **TT 全量：1527P/578F/64T → 1528P/562F/64T**。逐行 comm 对账（对称排除
  全部 zz-* 探针页，双侧 2081/2082 行）：**16 对 Fail→Pass 完美翻转闭合**
  ——default-policy.html **12F→12P 全翻**（script.src/script.text/
  div.innerHTML 四 sink × default/null/throw/undefined/no-default-policy
  五臂）、empty-default-policy 3F→3P（三 sink default 臂）、
  sandbox-allow-scripts 1F→1P（default policy in sandboxed page——
  require-TR 门激活即翻，sandbox 指令面残 1F）；TO 键集合 **64 键逐键
  恒等**（diff 零行）。
- 语料净账 **+16P/−15F**：另 1 行为 trusted-types-report-only.html 新增
  Fail（`violation report-only: assign string to script url`）——该页
  基线页级 TO 掩蔽全部子测，注入后页推进多上报 1 子测 F＝violation
  report 通道（SecurityPolicyViolationEvent/report-uri）**既有债现形**
  非回归；页级 TO 持位。
- clippy `collapsible_if` 修复（嵌套 if 折叠 let-chain，行为中性）晚于
  判读二进制——按字节绑定纪律 release 重建后重绑判读面：探针 4P、
  default-policy 12P/6F/1T（6F＝深层 default-policy 语义残债与基线恒等，
  非本刀波）、empty 3P/1T、sandbox 1P/1F 逐位复现 ✓。

## 挂账（本刀收口后余面）

- **`trusted-types` 名单指令 enforcement**：shim 只判
  `require-trusted-types-for`+`'script'` 子串，`trusted-types <list>/'none'`
  指令未解析（part14 FIXME(3b) 已记）——inheriting-csp-for-local-schemes
  残 2F+1T、creation-005 残 1T、sandbox allowlist 臂残 1F 持位，随指令
  解析刀位。
- RO/violation report 通道（预存）：report-only 页族 Timeout/F 面不动。
- connect-src 'none' enforce 面（navigate-008/sink-003/reporting 页注入
  后）行为面与 reporting/violation 通道耦合，随其刀位。

## 回归网

- `inject_csp_sidecar=false` 套件构造性零变化（共享面仅签名增参；cors
  直呼点显式 false；其余套件 sidecar 不被读取）。
- 产品代码零触碰：release 增量仅重编 zero-wpt-runner（15.95s 单 crate）
  ＝zero-engine/lib 面与③③b 记录 4252P/0F 持位（无代码路径交集）。
- `cargo fmt --all -- --check` 绿、`cargo clippy -p zero-wpt-runner
  --all-targets -- -D warnings` 绿。

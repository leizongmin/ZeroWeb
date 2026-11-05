# M0 甄别 —— LIKELY 五面 + mixed-content 窗口清单（2026-11-06）

对账基线 coverage-2026-11-02（全量头条 67.54%）；语料 pin
`315976933870b34d6ea30e3f6643403edae678ba`（`fetch-gap-batch1-subset.sh` 批拉）。
甄别三分类：**in-scope**（已实现域、通道可执行）／**engine-gap**（需引擎工作，
挂账他域或本 goal M3+ 切片）／**环境受限**（runner 管道或执行环境缺位，
zw-static-corpus 标记记账）。

## referrer-policy（对账 8,815 子测试 / 1,483 文件）

- 树形：gen/ 1,185（继承/worker 包装生成面）+ 4K·4K±1 三套镜像（各含 gen/+
  generic/）+ generic/ 33 + css-integration/{child-css,font-face,image} 26。
- **in-scope 池 = 69 文件**（generic/ 33 + css-integration/ 26 + 4K*/generic
  镜像——fetch 侧已按此清单拉取，gen/ 三处不拉）。
- **engine 域**：`referrerPolicy` IDL 反射（part01 `_REFLECTED_STRING_MAP`）与
  fetch `init.referrer`/`init.referrerPolicy`（part01:5513）已在；`<meta
  name=referrer>` 策略解析、子资源请求 Referer 计算面待 dry-run 对账。
- **环境受限**：gen/ 与 4K*/gen 的 iframe/worker 继承面（runner 无 wrapper
  管道，goal 划界挂账）；`.headers`/`.py` 服务端策略交付面（runner 无动态
  header 端点——`generic/` 内 meta-refresh 与 helper messaging 页逐案 dry-run
  对账）。**数值门 = M3 干跑实数落定**（池 69 文件 × 干跑通过率；预登记
  公式门：in-scope 子测试数 ≥ 干跑 P 数，超额部分为通道增量不设上限）。

## input-events（对账 3,378 / 35 文件）

- **in-scope 池 = 25 文件**（顶层全量；含 getTargetRanges tentative 11 页）。
- **engine-gap（主簇）**：`getTargetRanges` 全缺（shim 零定义——11 页 tentative
  判据面）；typing/cut-paste/spin-button 群依赖编辑管线 + `test_driver`
  真实键鼠注入（InputEvent 合成面 part01:8596 已有 inputType/data/isComposing，
  真实输入派发面缺）。InputEvent 构造器/IDL 静态面为可过域（同类域 IDL 面
  通过率参照 ~90%）。
- **数值门**：M3 干跑实数落定；预登记 = in-scope 干跑 P 数（编辑管线依赖簇
  如实落 F 账，不进门）。

## preload（对账 922 / 87 文件）

- **in-scope 池 = 51 文件**（`{{...}}` 模板与 iframe/worker 面剔除后顶层
  testharness 形态；resources/ 54 为资产页不按案跑）。
- **engine 域**：`crates/engine/src/preload.rs` 资源提示队列已有（preload/
  prefetch/preconnect/dns-prefetch 解析）；可观察面（link load/error 事件、
  performance entry、onload 延迟语义）待 dry-run 对账；modulepreload 面
  依赖模块管线（import_map.rs 已有，modulepreload 九页可能部分可过）。
- **数值门**：M3 干跑实数落定。

## wai-aria（对账 588 / 35 文件）

- **in-scope 池 = 3 文件**（`accessibility_properties_basic.tentative.html` +
  `aria-actions/idl-reflection.tentative.html` + `role/basic.html`）；manual/
  234 页为手工辅助 UI 面（非自动化判据，排除不拉通道）。
- **engine 域**：ARIA enumerated 反射表已实现（part01 R5009 片 e——
  ariaAtomic/ariaChecked/ariaCurrent 等 elements-aria-enumerated.js 全表），
  in-scope 三页预期高通过（同类域 IDL 反射面 ~90%）。
- **数值门**：M3 干跑实数落定（池极小，预期个位数子测试）。

## storage（对账 482 / 56 文件）

- **in-scope 池 = 2 文件**（persist-permission-manual.https.html 为 manual
  形态应再排除——实际可执行 ≈ buckets/detached-iframe.https.html 1 页）；
  主面 27 个 `.any.js`（estimate/persist/persisted/buckets）fetch 侧不拉
  wrapper（timing 先例），runner 无 .any.js window 通道清单。
- **engine-gap（全域性）**：StorageManager 仅 OPFS getDirectory
  （part02:9649 诚实范围注释）；`estimate/persist/persisted` 全缺。
- **处置**：不设数值门（池 ≈ 0 可断言子测试）；engine-gap 清单
  （StorageManager estimate/persist/persisted + buckets API）落本文件，
  M3 按簇立项建议（storage 域引擎工作归 webstorage/OPFS 邻接域）。

## mixed-content 窗口可执行清单（M2 / DC-3）

- **in-scope = 顶层 2**（blob.https.sub.html / imageset.https.sub.html）+
  **tentative/autoupgrades 4**（audio/image/video-upgrade + mixed-content-cors）
  = 6 文件 11 子测试；generic/ 2（test-case.sub.js + sanity-checker.js，
  TOP 面 helper 资产）；resources/ 资产页。
- **gen/ 512**（window.js/iframe 包装生成面）不拉不扫——runner 无 wrapper
  管道，挂账 runner 基建（goal 划界同域）。
- **实跑对账（2026-11-06，pin 31597693）**：6 文件 11 子测试全入账
  （0P/10F/1T）；既有账 blob 1T + imageset 1F 逐位不变（零回归锚）；新增
  autoupgrades 面 9F = autoupgrade 语义引擎缺位（如实落账）。missing=0。

## DC-4 数值门汇总（决策规则对位）

- CONFIRMED 六面：M1 干跑基线实数落门（见
  [M1 基线报告](2026-11-06-m1-confirmed-channel-baselines.md)）。
- LIKELY 五面：池已甄别（69/25/51/3/≈1 文件），数值门 M3 逐面干跑实数落定；
  预登记规则 = 门 ≥ 干跑基线 P 数（零回落约束），storage 不设门。
- 环境受限面（referrer-policy gen/ 镜像、input-events 编辑管线簇、storage
  .any.js 面、mixed-content gen/）：zw-static-corpus 标记记账，不进门。

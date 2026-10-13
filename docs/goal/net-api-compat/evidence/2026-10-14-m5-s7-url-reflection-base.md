# M5-S7 — URL 反射基址接线（form.action / src 族 / 提交导航 / Attr.baseURI，2026-10-14）

**背景**：M5-S6 残余记账——`_zwDocBaseURL` helper 已就位，其余以页面 location
为解析基的 URL 反射/导航点逐点接线（spec：url 型 IDL 反射与请求派发均对
document base URL 解析）。

## 接线点（shim 五处，零 .rs 变更）

1. **`_zwResolveFetchUrl`（part01）**：共享解析器基改 helper（无 helper 回落
   `_zwCurrentHref`）——一次覆盖 fetch 请求派发（part01:2714/3547）、XHR 相对
   URL（part03 resolveXhrUrl 回落支）、track/SOURCE/media/iframe src 加载链
  （part04/06 各 settle 面）。img.src setAttribute 的 CSP/media 钩子路径同经此。
2. **`form.action` IDL getter（part03 FORM 反射分支）**：解析基 helper。
3. **`_zwFormSubmitNavigate`（part04）**：提交导航 mutate action 的基改 helper
  （spec form submission algorithm）。
4. **通用 URL 反射 getter（part04:9215，R5009 片 e `_REFLECTED_URL_TAGS` 全族）**：
   poster / track.src / input.src / script.src / frame.src / cite / object.data /
   lowsrc / longDesc / codeBase 解析基 helper。
5. **`formAction`（part04 form 关联元素反射）+ `Attr.baseURI`（part03 R130）**：
   同接线（formAction 空/missing 缺省 = 文档基 URL；Attr.baseURI = 文档基 URL）。

BASE/LINK href getter 维持对 fallback（location）解析——spec 防自引用语义，不改。

## 探针实证（5/5，跑后已删）

`url/zz-probe-reflect.html`（动态设基——静态 `<base href>` 会把 `/resources/*.js`
根相对解析进 base 域，harness 自身加载即被改写，**恰证资源加载面接线生效**；
另探针页含 `createElement("iframe")` 字面量会被通道 iframe 筛减静默跳过——探针
须避开该字面量）：

- document.baseURI = 动态基址 ✓
- form.action 相对 → 基址解析；missing action → 文档基 URL ✓
- src IDL 反射（FRAME.src 走 `_REFLECTED_URL_TAGS` 通用 getter）→ 基址解析 ✓
- Attr.baseURI = 文档基 URL ✓
- XHR 相对 URL responseURL = 基址解析 ✓（fixture 404 路径 responseURL 保真）
- **img.src**（`_REFLECTED_URL_TAGS` IMG 条目，同通用 getter）✓——末段无斜杠基址
  按 URL 语义丢末段（首版探针期望写错非引擎缺陷，双形态基址复跑全绿）

## 验证

- **六 corpus 受控 A/B（对 M5-S6 树）**：除探针页（4 子测试，上轮残留本轮删）
  外**逐页字节级持平**（9041P/3024F/10T 不变）——无 base 元素时全部接线点行为
  逐字节不变（helper 纯回落），预期零位移成立。
- `make test` 全绿 **20657P/0F**（R5009 反射桥测面全保持）；clippy
  （engine/wpt-runner，-D warnings）零告警；fmt 本流 crate 干净。
- bench-gate（zero-engine 定向）：FAIL 五项（end_to_end_pipeline /
  incremental_vs_full / paint 两项 / first_paint_wall_ms）——全部纯 Rust 微基准
  （不执行 JS shim），M5-S6 轮已 stash 对照实证同族 FAIL 为机器态漂移既有
  （基线 2026-09-15 @ a732a493d）；本轮 FAIL 集较上轮扩大（paint_simple
  121µs→149µs），跨轮漂移形态与代码无关。不放宽。

## 残余记账

- 资源加载面（script/img/track 实际 fetch 的 URL 构造）已随 `_zwResolveFetchUrl`
  接线，但 runner fixture 通道的 URL 改写（`strip_prefix("https://wpt.test")` 双
  scheme，M5-S5）未含 http/https 外第三形态基址域——语料无此形态，观察项。
- `_zwCurrentHref`（网络面 Origin/Referer 语境）与 fetch 资源计时登记
  （part01:3266 RT bookkeeping）维持页面地址——语义非文档基 URL 面（origin/
  referrer 不随 `<base>`），不改。

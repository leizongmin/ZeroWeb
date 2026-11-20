# DC-2 · html/dom 批一：variant 展开与 idlharness 解锁

2026-11-20。DC-2 第一批落在 html/dom 簇（11-20 快照缺口 13,415 子测试）：查账先行
（wpt.fyi 分母逐文件对 ZeroWeb CI 产物分子），按缺口形态拆三臂——variant 展开机械臂、
idlharness 解锁资产臂、（挂账）成员面/innerText getter 引擎臂。html-syntax 套件实测
+1,888 Pass、共享键零回归。

## 查账口径（与 coverage 快照同链）

- 分母：wpt.fyi aligned stable chrome 158.0.8127.36 / firefox 160.0.1 / safari 27.2
  @ `4b26db8978`（本地缓存 `tests/wpt-runner/wptfyi-cache/summary-*.json`）；
  html/dom 顶级 = 389 文件 / 72,400 子测试（132 零子测试 reftest/support 形不计）。
- 分子：run 44668230656（2026-11-20 weekly）`zw-results-syntax-dom` 分片
  `testharness-html-syntax.json` + `testharness-dom.json`（dom 顶级不跑 html/dom，
  html/dom 全部出自 html-syntax 套件），同文件多套件取最完整一次（coverage 口径）。
- 查账结果：html/dom 分子 59,035，缺口 13,365（快照 13,415 的当代形，差值 = 上游
  漂移/跨日）。缺口构成（子测试数）：
  - idlharness 4 键 6,939（`?include=HTML.+` 3,955 / `?exclude=...` 1,878 /
    `any.worker` 808 / `?include=(Document|Window)` 298）——通道 `idlharness.*`
    与 `.https.html` 排除规则门控，分子 0
  - reflection-* 10 文件 ~3,000（每文件分子 ≈90–96% den，Fail=0＝页面少生成块，
    非 Run 截断——reflection-text 本地实测 2.4s rc=0 排除超时假设，缺的块是 shim
    DOM 面缺失导致页面数据表静默缩量）
  - innertext-with-white-spaces 12 variant 键 948（此前只跑 bare 键 79＝等价
    `?white-space=pre` 面）
  - getter.html 277（innerText getter 需布局信息，86 Fail + 1 Timeout）
  - elements/ 1,511、render-blocking 91、documents 84、partial-updates 22（叶面）

## 变更（两文件 + 语料）

1. `tests/wpt-runner/src/testharness.rs`——`run_html_syntax_cases` 开
   `expand_variants`（navigation-compat S4K 先例：变体键替代基础键，wpt.fyi 分母
   variant URL 独立键同形）；`html_syntax_case_skipped` 提前返回例外放行
   `html/dom/idlharness.https.html`（`.https` 命名不设障碍——runner 恒
   `https://wpt.test`、isSecureContext 恒真；页内 hidden iframe 是取 Document 的
   测试夹具，R115 iframe 管道成熟；WebIDLParser.js / idlharness.js 已在 corpus）。
   单测 `html_syntax_skip_idlharness_carveout_and_variant_metas` 锁例外不外溢
   （worker 形 / secure-context 命名页仍排除）与正则字符变体 content 保真。
2. `tests/wpt-runner/scripts/goals/40-html-syntax-compat.sh`——补拉
   `interfaces/html.idl` + 页面 dependencies 11 个依赖 IDL（wai-aria/SVG/cssom/
   touch-events/pointerevents/uievents/xhr/FileAPI/mediacapture-streams/
   performance-timeline/trusted-types；`.idl` 后缀不在 fetch_dir_html 模式，显式
   单文件；idl_test 运行期 fetch 走 page fetch 静态文件回退路径——webgl idlharness
   先例）。`common/subset-tests-by-key.js` / `get-host-info.sub.js` 已在 corpus。

## 结果（release runner，test-guard 包裹，8 job）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| html-syntax 套件全量 | 61,840 Pass / 273 键 | 63,728 Pass / 454 键 |
| innertext-with-white-spaces 12 variant | bare 键 79 | 570（pre/pre-wrap 8 键各 79 全过；pre-line 3 键各 32；normal 3 键 0） |
| idlharness.https.html 3 variant | 0（排除） | 1,397（`?include=HTML.+` 923/3,896；`?exclude` 426/1,628；`(Document\|Window)` 48/298） |

- **共享键零回归**：全量键 diff——共享 269 键 Pass 计数无一变化（无回归无意外增益），
  GONE 4 键 = 3 html5lib bare + 1 innertext bare（均被变体键替代），NEW 185 键 =
  12 innertext variant + 3 idlharness variant + 170 html5lib variant（`?file=` 形，
  write-mode 面仍单 synthetic Fail，分母 61 variant 键/浏览器同形对齐）。
- innertext 12 variant 键与分母逐字对齐（den-not-run 空）；idlharness 3 variant 键
  同形（第 4 键 `idlharness.any.worker.html` 808 仍 worker 面挂账，未解锁）。
- 净账：html/dom 分子 59,035 → 预计 60,923（+1,888），簇缺口 13,415 → ≈11,527。

## 挂账（html/dom 簇后续批）

- **IDL 成员面战役（~4,425 Fail）**：idlharness 失败清单即精确缺员表——高频形态
  `assert_inherits: property "X" not found in prototype chain`（setRangeText×96 /
  setSelectionRange×48 / stepUp+stepDown×92 / align×43 / width×40 / height×33 /
  disabled×31 等）与缺全局接口（OffscreenCanvasRenderingContext2D×88）。逐接口补
  shim prototype 表可批量回收；本批不动引擎，留批二。
- **innerText getter 渲染语义（~576）**：getter.html 277 + innertext normal
  （白空间折叠）299——需布局信息（rendered text collection steps），渲染面挂批三。
- **reflection-* 块缺生成（~3,000）**：需逐文件对 wpt.fyi 每文件子测试清单 diff
  定位缺失块（数据表驱动的静默缩量），批二/三。

# R5507 min-width content 关键字 × definite width 内容基地板（2026-11-12）

## 一、盘点与择案（R5507 下一步①）

本轮 corpus 现基线 = 690F（main 50dfbd4f7 全量重跑；4 个相对 688F 的 mover solo×3 双态 12/12 全 PASS＝窗快照噪声，真态 688F 等效——tao/insert 在册家族无新扩员）。

**双引擎 triage**（#28 行为判别扩展到逐案）：22 个未在册精确名候选全量双渲——ZW（REFTEST_DUMP）vs 本机 chromium headless（puppeteer-core + 本地静态 HTTP server 供 wpt-data，.xht MIME 映射 application/xhtml+xml）像素对比。定性结论：

| 案 | diff% | 定性 |
|---|---|---|
| margin-bottom-applies-to-012/013/014/015 | — | **upstream-red 排除**（chromium 与 ZW 像素同形：loose 手写 ref 连 chromium 也不符，#33 oracle 法） |
| text-decoration-inset-025 | 17.1 | 字体度量噪声（ZW fallback 字体更宽→右列折行；装饰几何一致） |
| bidi-007b/009a/009b/010b、first-line-014/015/016 | 2.4–16.5 | bidi/行盒装饰域（对端面/字体耦合，非轻量） |
| uri-013 | 13.9 | tokenizer 错误恢复（双侧均不符 ref，多子案非轻量） |
| list-style-position-024 | 13.6 | RTL marker 定位 + bidi 标点序耦合（非轻量） |
| tao-155/156 | 6.3/6.1 | 表格匿名对象修复域（复杂域，tao 家族邻位） |
| text-overflow-013、position-absolute-in-inline-004/005 | 2.3–11.0 | 溢出/abspos-in-inline 结构性发散（深域） |
| fit-content-block-size-fixedpos | 4.6 | fixed 容器 height:fit-content 特征缺口（非单点） |
| **fit-content-min-inline-size** | 2.3 | **min-width:fit-content 关键字未生效**——单点，本面（sizing），择定 |

## 二、归因链

**驱动案**：`outer(width:fit-content) > inner(min-width:fit-content; width:10px) > 2×inline-block(50px)`，期望 inner/outer 均 100（chromium 100×100 绿方块 oracle）；ZW 实测 inner/outer 均 10。

**临时探针三变体**（布局几何单测，修复前）：V1 主形 outer=10 inner=10；V2 孤立 inner=10；V3 min-content 关键字同构 =10——**min-width content 关键字 × definite width 交叉域整体塌声明宽**（连 R4151 既有域同塌，缺口早于本轮）。

**仪表定谳**（sizing.rs R4149 臂 eprintln，用后还原）：`intrinsic=10 should_apply=false`——测量侧就塌，写入臂从未执行。逐层核对：

1. **解析层 ✓**：活路径 `css-parser/src/values/types.rs:2363`（R1018）bare `fit-content` → `MaxContent` 正确（parse_basic.rs 同名逻辑是死代码，读码陷阱已记）。
2. **触发 ✓**：engine.rs SizingTargets gate 含 min_width 关键字，pass 进入。
3. **测量 ✗（根因①）**：`kw_aware_max_width` 对 min_w=MaxContent 盒返回 `full` 口径——而 `full`（include_own_width=true）被 `box_content_max_width_inner` 尾部 walk 的 **definite-width 守卫**（own_explicit > 0.5 时不运行 atom/inline 段核算 walk，children_inner 塌 0 → 叶回退 = 声明宽 10）压制。两个 max_w 关键字分支（R5375 content-only、R4151 cap 走 child_cap）都有正确口径，唯独 min_w 纯关键字路径直接返回 full。守卫本意（line-break normal-014：定值盒内容折行不外延）与 min 关键字语义（内容基下限）在该交叉域互斥。
4. **写入 ✗（根因②）**：R4149 写入臂 kw_min 分支把 `size.width` 清成 `auto`——taffy 块级 auto 拉伸到 CB 宽（converter 语义），宽 CB（body 800px）下 min 地板盒被过填充到 800 应 100。

## 三、修复（两刀）

- **intrinsic_sizing.rs `kw_aware_max_width`**：新增 min 关键字 × definite width 臂（置于两 max_w 分支之后，max_w 关键字案不触达）——`content = box_content_max_width_inner(content_only)` 直测（不经本包装防递归重入；子树子盒各自经递归消费自己的关键字），返回 `content.max(own_def + frame)`（css-sizing-3 §5.2 min-width 胜 width 取大；width:200px + min-width:min-content 内容 50 → 贡献 200 保持）。`content_only` 语境（R5037 W_max 公式口径）经新参数短路不进本臂——fit-content() 公式的 W_max 不得被 own 抬升（R5037 语义保持）。无 definite width 时 full 已是内容基（守卫门不触发），不重测。
- **engine/sizing.rs R4149 写入臂**：kw_min 分支只设 `min_size.width`，不再清 `size.width`——taffy 约束求解 min ≥ size 自动取大（§5.3）；kw_max 分支维持原样（size=auto 清法未动，max 关键字宽 CB 域独立，未见驱动案）。

## 四、验证（本轮 2026-11-12 续轮补全——上一 session 429 中断，修复+锚+归因已留档）

| 门 | 结果 |
|---|---|
| 常驻锚 r5507_min_kw_content_floor_tests 4 test | 全绿（恰集门 4 passed/2479 filtered） |
| 锚 stash 证伪（只 stash 两修复文件、保留锚） | 基态 3/4 红（主形宽 CB 精确/链式外层跟随/min-content 域 parity 全塌声明宽）；own_def 取大守卫锚两态绿＝守卫性质锚（防修复收缩声明宽），非判别锚 |
| 临时探针三变体 | 修复前 10/10/10 → 修复后 100/100/100 |
| 包测试 cargo test -p zero-layout-engine --lib | P=2481 F=0（含锚） |
| make test 全量 | workspace 腿中止于 zero-engine js_dom_bridge **10F＝对端在册族**（空载复跑 16.57s 确定性同 10；非本流改动面——fmt 代修已证 token 流一致）；layout-engine 域零回归，renderer/browser 腿因对端失败未跑（对端门禁债） |
| make reftest 725 | **725/725 全绿**（真通过 559＋可疑 112＋近似 54） |
| product-smoke | welcome desktop diff **17.23%**<20%（R5501 基线同值零漂移）＋struct-check 全 PASS（card:4/shortcut:6/link-tile:4/footer:1/title:1/tagline:2）＋@320 article 结构门 PASS |
| corpus 全量双态 A/B（16594 案） | base 689F → new 685F；单采样 FIXED=4（**驱动案 fit-content-min-inline-size**＋tao-128/151＋intrinsic-percent-replaced-dynamic-008）REGRESSED=∅；3 个非驱动 mover solo×3 双态 18/18 全 PASS＝**全噪声**（tao 属在册互翻族邻位、dynamic-008 属在册 intrinsic-percent 族）——**真态 FIXED={fit-content-min-inline-size}（solo 双态定谳：new ✓100%/base ✗100%）REGRESSED=∅、真值≈688F 等效** |
| bench-gate 定向 layout-engine | GATE PASS 25 指标 NEW=0（静窗） |
| clippy -p zero-layout-engine --all-targets -D warnings | 无告警 |
| rustfmt | clean |

**双二进制行为判别**（#28 纪律）：base release runner（全 stash 源回退态构建，cp 至 /tmp 独立保存）vs new（修复态构建）；驱动案 solo `reftest-upstream fit-content-min-inline-size`：new 真通过-可信 1/1、base 不一致 1/1。

**证伪方法勘误**（#39 恰集门自证）：首轮证伪误将锚文件随 stash 一起藏（untracked `-u`），过滤器空转 0 passed 假绿——重做：部分 stash 只藏 `sizing.rs`+`intrinsic_sizing.rs`、保留 mod.rs 注册与锚文件，基态 3/4 红成立。

## 五、残留与挂账

- kw_max 写入臂 size=auto 清法在宽 CB 域的对称过填充风险（dynamic-012 形态于窄 float 语境绿；宽 CB 未见驱动案，挂账观察）。
- kw_aware `max_w=MinContent` 分支的 `full` 同受 definite-width 守卫影响（dynamic-012 经 child_cap 语义正确，未动）。
- fit-content-block-size-fixedpos（fixed 容器 height:fit-content）特征缺口、uri-013 tokenizer 错误恢复多子案——本轮 triage 已定性，挂账深域。
- margin-bottom-applies-to-012/013/014/015 记 upstream-red 排除（控制面本轮同步）。

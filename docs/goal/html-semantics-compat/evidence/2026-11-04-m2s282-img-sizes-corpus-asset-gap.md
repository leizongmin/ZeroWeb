# M2 s282 — the-img-element sizes 族语料资产补缺（sizes-auto 族 6F→6P，全树 FIXED=6/REGRESSED=0）

日期：2026-11-04。切片号 s282（承 s281 下一步候选——红群普查选
the-img-element sizes 族）。**纯语料资产补缺，零引擎/shim/runner 改
动**：3 资产 + 2 脚本注记（fetch 白名单 + 升级注记）。

## 勘根

1. **s281 候选①过时承接销案**：s280/s281 账本「below/in-viewport-
   dynamic 候选」为复制粘贴漂移——s277 已收口该面（once-lazy inflight
   双页收口销账，账本 2026-11-03-m2s277-once-lazy-inflight-fetch.md
   第 92 行）；s281-fulltree-b.log 两页双绿实证。候选作废。
2. **红群普查**（s281-fulltree-b.log 413F/62T）：the-img-element
   sizes 族集中（sizes-auto 6F+1T、implicit 1F、resize-observer 1T、
   dynamic 1T）+ current-pixel-density 10F + error.html 7F +
   adoption 1F+1T。
3. **资产引用普查**（grep 三资产）：green-1x1/green-16x16 → 12 页
   （sizes-auto 三页 + adoption + environment-changes/iframed +
   image-compositing-change±ref + non-active-document +
   srcset-w-reselect-on-resize + area crash + customizable-select
   ±ref）；srcset/resources/image.png → 仅
   implicit-sizes-ignores-width.html。缺资产生效面 = testharness 消
   费页（reftest 对/iframed 子资源/域外页不在本轮 runner 路径，零位
   移实证）。
4. **面分型**（三资产补入 + solo/family 实跑定性）：
   - **资产缺口面 7**：sizes-auto.html 6F（断言 ref 面引用两绿块资
     产，404 → onerror → 断言永假）+ resize-observer-removes-auto
     1T（404 → onerror reject → harness 永不完成 → 文件级
     Timeout）。
   - **渲染域面 10**（移交 rendering-compat 边界）：
     current-pixel-density/basic.html 3F（clientWidth 亚像素舍入
     got 1 期望 0）+ error.html 7F（broken img 应渲染 alt 文本盒，
     expected_width got 0——错误图占位盒渲染缺席）。
   - **真语义面 3**（候选）：①implicit-sizes-ignores-width（width
     attr=400 + srcset 100w 无 sizes → 期望按选择源 100vw=800 渲
     染，引擎给 400；补资产后失败信息从加载错误翻为真断言
     "expected 800 but got 400" 实证资产分量剥离）；②
     resize-observer-removes-auto（ResizeObserver 触发 lazy→eager
     重评，资产补入解 harness 阻塞后真面首报）；③adoption
     （DOMParser parseFromString + adoptNode 机具，补资产后仍
     1F+1T solo 实证推翻资产嫌疑）。
   - **sizes=auto 机具面 ~28 + 2 邻页**：sizes-auto.html 1T +
     dynamic-attributes 1T + lazy/sizes=auto 子测群（s281 已挂账
     F5 简化挂起项 part03:2632「忽略 auto 项继续」待解除）——未来
     切片主体。

## 施工（零代码，3 语料资产 + 2 脚本注记）

- `tests/wpt-runner/wpt-data/images/green-1x1.png`（82B PNG 1x1，
  ZeroWeb-2 姊妹 clone 同件复制）
- `tests/wpt-runner/wpt-data/images/green-16x16.png`（92B PNG
  16x16，同上）
- `tests/wpt-runner/wpt-data/html/semantics/embedded-content/
  the-img-element/srcset/resources/image.png`（100x100 PNG 268B，
  gh api contents 按 lib.sh WPT_REV pin
  315976933870b34d6ea30e3f6643403edae678ba 上游原件取件——jsdelivr/
  raw 当日超时走 api.github.com 替代通路）
- `scripts/update-wpt-data.sh` 已知缺口清单追加注记（s227 格式，
  升级新 ref 后需重新补）
- `tests/wpt-runner/scripts/goals/25-html-semantics-compat.sh` ③e
  白名单三行（fetch_raw pin 版幂等自愈——缺席自动补齐优于纯注记，
  s225/s264 先例）+ 基准变更注记（下轮全量 join 位移按语料变更归
  因）

## 验证

- **family A/B**（FILTER=the-img-element，三资产移位 stash 法，
  make 目标 test-guard+RAF 同机；日志 s282-imgfamily-{a,b}.log）：
  A 896P/51F/10T（957 键）→ B 902P/46F/9T（957 键），键级
  **FIXED=6 / REGRESSED=0 / T-changed=0**，LOST+GAINED 各 1 =
  resize-observer 文件级 Timeout 键失 + 真面子测键得（解蔽对）。
  6 FIXED 恰靶 = sizes-auto.html 六子测（全数引用补入资产）。
- **全树 A/B**（baseline = s281-fulltree-b.log——语料 gitignored，
  该跑早于资产补入，runner 同版本同机构；日志 s282-fulltree-b.log，
  对比脚本 /tmp/s281-ab-compare.sh 复用）：行级 7866P/413F/64T →
  7875P/408F/62T（**+9P/−5F/−2T**），键级 **FIXED=6 /
  REGRESSED=0 / T-changed=0**：
  - 行级增量全数对账零悬置：+9P = 6 资产修复 + 3 link 噪声子测 +
    0（resize T→F 净零）；−5F = −6 资产 + 1 resize 解蔽；−2T =
    resize + link 文件级。
  - LOST/GAINED 各 2 键非回归：①resize-observer T→F 解蔽对（上
    述）；②link-multiple-load-events 文件级 Timeout 键失 + 3 子测
    Pass 键得 = **T→6/6P 有利翻转**——s157 贴边页侧摆（s281 solo
    ×3 6/6P 同向先例）；本片零代码改动，该页任何位移定义性为环境
    噪声，s157 账面维持不归因。
- 其余引用页零位移：non-active-document 3P 双侧同、adoption
  1F+1T 双侧同（真面定性）、iframed/compositing/area-crash 不在
  runner 结果面（非 testharness 消费页）。
- **make test 不涉**：零 Rust/shim/runner 改动（仓库面仅 shell 脚
  本注记，非测试路径）；runner 二进制自 s281 提交无重建需求（family
  make 目标依赖链 no-op 实证）。零代码改动 clippy/fmt 不涉（s279
  口径）。

## 挂账更新

- **s281 候选①销案**：过时承接（s277 已收口，s280/s281 账面文案
  漂移核销——候选文案核对源账本为新增纪律）。
- **渲染域挂账（移交 rendering-compat，勘根已定性）**：error.html
  alt 文本盒 7F + current-pixel-density clientWidth 亚像素舍入 3F
  （basic.html；同页 15 img 中 11 绿 3 红 1 骨架）。
- **真语义候选 3**：implicit width-attr（映射点已定位
  style-system/src/lib.rs img 臂 ~3168 无条件 width/height
  attr→hint；规范形态——responsive images（srcset/sizes）场景
  dimension attrs 只喂 aspect-ratio 不映射 width/height present-
  ational hints，letter-exact 规范引用下片勘根补）；resize-observer
  lazy→eager 重评；adoption DOMParser+adoptNode。
- **sizes=auto 机具切片**（未来主体）：~28 面 + 2 邻页，F5 简化挂
  起项解除 + 渲染宽度源尺寸 + lazy script-created 通道三件套。
- 承前不动：root margin 净速分档（s271 挂账①）；dynamic import()
  referrerpolicy 穿参；decode() 兜底元组细化（s276 注记）；poster
  资产 2048x1360-random.jpg 缺席注记（s281，非断言消费型）。

## 下一步候选

1. **implicit width-attr 规范勘根 + 实施**——单点真语义面（img 臂
   加 srcset/sizes 存在判定分流 dimension attrs 去向），letter-
   exact 规范引用 + corpus 面 1F。
2. sizes=auto 机具（~28 面，三件套：F5 解除 + 渲染宽度源尺寸 +
   lazy script-created 通道）。
3. adoption/resize-observer 真面勘根（机具面，视 1/2 进度穿插）。

## 证据

- `evidence/logs/s282-imgfamily-a.log`（family A，资产缺席态）
- `evidence/logs/s282-imgfamily-b.log`（family B，资产补入态）
- `evidence/logs/s282-fulltree-b.log`（全树 B，A = s281-fulltree-b.log）
- 资产 md5：green-1x1 82B add96bd2…、green-16x16 92B 46d671a1…、
  srcset/resources/image.png 268B 61d6acb5…

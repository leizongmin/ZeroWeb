# WAAPI 动画值进 gCS 刀——shim 注册表 + 端点合成 + 形状守卫（jsanim knife v1→v3）

2026-11-15。接 [WAAPI 门刀收口](2026-11-15-waapi-gate-knife.md) 地图校正后的「动画值进 gCS」刀：
同步 `getComputedStyle`（gCS）对 harness Web Animations 方法（`animate(...)` → `pause()` →
`currentTime = t`）恒读 underlying 值。本刀在 shim `Animation` 与引擎 gCS 序列化点之间建
注册表通道：shim 经 `__zw_js_anim_register/seek/playpause/cancel` 把 keyframes/timing/播放态
登记到引擎侧 [`js_anim_registry`]（thread_local ENTRIES + 独立 `AnimationClock` 实例），
gCS selector/handle 两通道序列化前咨询，命中属性返回插值串（apply 后过
`serialize_computed_property_with_sibling` 规范化）。

## 机制要点

- **零干扰**：表空查询 O(1) 返 None，行为与刀前逐字节一致；独立时钟实例，渲染管线
  时钟/事件派发零触及；页面换代 `clear_all()` 清表（同 `clear_generation_cache` 调用点）。
- **端点原文表**：注册时直读合成 `@keyframes` 规则的帧声明串（不走 tick 探针——tick
  推进状态到 finished 后 fill forwards 恒返回终帧，探针翻红污染后续求值）。
- **fallback 检测器**：开区间进度（0 < prog < 1）内 tick 输出恒等端点原文 = 引擎未真
  插值（关键字/calc/url/翻转等离散 fallback）→ 不采纳，回落既有 B0 通道；端点钳制形态
  保留采纳。
- **canonicalize None**：apply 后序列化恒等（applier 未覆盖该属性或动画值 ≡ 基值）返
  None 回落 gCS 通道——闭合「期望元素 gCS 为空串对 raw 透传裸串」的 REGRESSED 伪源
  （font-variant-ligatures 实证）。
- **端点合成（v3）**：neutral 单帧臂（harness `isNeutralKeyframe` 跳过中性端点，spec kf
  只剩 o=1 帧）缺端点帧时时钟填 initial 值而非 underlying → at(0)/at(-0.3) 臂返回
  initial 串。JS 侧在注册前读 underlying（`__zw_get_computed_style(sel, prop)` /
  `__zw_get_computed_style_by_handle`）补齐缺失 0%/100% 帧——注册前本动画不在表内，
  无自递归。spec：Web Animations §5.4.15 computed keyframes 缺失端点用 underlying 合成。
- **混合单位端点门（v3）**：translate px↔% 对、transform px↔em 对——引擎跨单位长度
  插值退化为 px 数值插值，输出值形与端点原文不符（fallback 检测不触发）且与 harness
  probe 期望恒异。端点单位序列不一致的属性不登记，回落 B0 通道。

## 三代 A/B 定谳（五组 corpus，B0 = clean eba29cc85 快照二进制）

| 代 | 形态 | gained | lost | regressed | 备注 |
|---|---|---:|---:|---:|---|
| v1 | stash 刀（注册表 + gCS 咨询裸奔） | 586 | 134 | 245 | 全量分类见下 |
| v2 | +形状守卫（composite/calc-size/random/裸关键字端点）+fallback 检测器 +canonicalize None | 183 | 0 | 9 | 零回归方向，残 9 条 |
| v3 | +端点合成 +混合单位端点门 | 198 | 0 | 0 | 五组全零回归；主增益 transforms +185（全账见验证节） |

v1 的 245 条 REGRESSED 全量归因：composite add/accumulate 107（border-*-width-composition
80 + font-size-adjust-composition 14 + transform 8 + rotate 2 + background-size 1 +
translate/scale 等 2）；calc-size/定位关键词族 60；math-fn calc 混合 16；url 族 12；
neutral 缺端点族 ~28（got '' / initial 填充形）；font 离散关键词 ~13；alpha 形式 3；
random/flip 1；杂散 ~5。

v2 权威账（bin-jsanim-b2 快照，15:38:56 构建，五腿全库存）逐组：
css-transforms +170/0/9、motion 0/0/0、css-backgrounds +13/0/0、css-values 与 B0 逐字节
持平（4807P/5033F）、css-fonts 与 B0 逐字节持平（2856P/5394F）。fonts 的 v1 增益
（180：font-size-adjust-composition 122 + font-variation-settings-composition 8 +
离散面 50）系 composite blanket 守卫与离散翻转回落的全折——守卫按零回归优先，增益
恢复归下一刀（composite-add 建模 + easing 感知离散翻转采纳）。

v2 残 9 条 REGRESSED 与 v3 两刀逐一对应：neutral initial-fill 3（rotate got '0deg' /
scale got '1' / translate got '0px'——apply 成功故 canonicalize None 兜不住）+ 混合单位
6（translate [480px 400px 320px]→[240% 160%] ×5、transform translateX px↔em ×1）。

## 三伪账/事故记录

1. **corpus 库存瞬态缺席**：15:36 b1v2 链 css-transforms 腿跑时
   `css-transforms/animation/`（31 entry-页）与 15:50 b1v2 css-values 腿跑时
   `css-values/calc-size/`（14 页）目录瞬态不在树（peer 流施工目录在共享树出没），
   页级 A/B 把缺席全记 REGRESSED（伪 183 条 transforms REG）。修复：fix 链以同快照
   runner + `CSS_CORPUS_ONLY_SUBSTR='css-transforms/animation'` 补齐 42 案（31 entry-页），
   与 v2 JSON 合并成完整账；教训——**corpus A/B 前必核双腿 page 集合相等**（B0/v1 页数
   差 = 缺席污染，不采信）。`cases=` 汇总是 merge 侧有产出页计数非尝试数（B0 transforms
   transcript 实跑 1243 案 vs cases=112），库存核对以 transcript `==== ` 行为准。
2. **B0 伪通过池**：v1 的 245 REG 中相当部分 B0 侧为回声/互垃圾伪通过（echo 恒等
   underlying、期望本身是引擎 probe 垃圾串如 font-stretch 全面期望 '100%'）——B0 通过
   ≠ 语义正确，刀的真值化会翻红；零回归_norm 下以守卫保位，真值增益留给后续语义刀。
3. **binary provenance**：全程快照二进制（bin-jsanim-b0 / runners/zw-jsanim-knife-v2 /
   bin-jsanim-b2 / bin-jsanim-b3）防共享树中途重建混源；b3 provenance 以二进制
   `_zwPv` 0→5、`_zwMs` +5 行 delta 复核。

## 验证

- v3 首批（batch2，15:55）：单位门首版把无单位端点侧（none/无单位数值）误判为
  与有单位侧失配，none→X 合法插值臂被陪葬——transforms 探针 666P 对 b2 687P
  **−21 伪回归**。停批修门（`_zwUm.length === 0` 侧跳过比较，仅两侧均非空且
  序列不同才门控），重建（provenance：二进制 `_zwUm.length === 0` 标记 1 处）。
- v3 采纳批（batch3，bin-jsanim-b3 快照，五腿全库存，results-jsanim-b3）：
  transforms 113 案 702P、motion 45 案 105P、backgrounds 124 案 1357P、values
  256 案 4807P、fonts 163 案 2856P。全账 A/B（`ab-compare.py` B0 vs b3 页级）：
  **css-transforms +185/0/0（13 页全双边显式转移）、css-backgrounds +13/0/0、
  motion/values/fonts 0/0/0（values/fonts 与 B0 汇总逐字节持平）——五组合计
  +198/0/0 零回归**。对 b2（+183/0/9）：9 条 REG 清零 + neutral-mid 新增益。
  transforms transcript 尝试数 B3 1244 vs B0 1243（+1 案树增长伪差，页级双边
  转移已证无孤儿页污染）。values/fonts 页集 256/163 双边相等、motion 207=207、
  backgrounds transcript 1251→1254（同为树增长）；B0 values/fonts transcript
  尝试数偏大系旧批重试协议差，库存以页级 JSON 为准。
- 提交门禁 make test（mkt3 16:47 / mkt6 16:57 双跑一致）：fmt/clippy 绿；workspace 腿
  135P+**1F**——`service_worker_manager::module_update_compares_static_dependency_bytes`
  真实代码红（11-15 纠类，非本刀引入非回归）：es_module 重写器把具名导入绑定为 **globalThis getter**
  （es_module.rs:1732 设计注释「具名/默认导入绑定安装 globalThis getter」），
  84a48833d（8-20）老钉主脚本 `globalThis.value = value` 赋值撞 getter-only
  抛 TypeError（脚本 :23）→ ScriptFailed(Runtime) → 永不 Activated 超时。
  SW 沙箱独立 prelude 不装 DOM shim，与 engine 动画刀零代码路径；归
  script-sandbox 模块波（s345/s418/module-import-meta 面），修复归
  script-sandbox 流（导入绑定应模块局部作用域化，非全局面）。红窗 11-08
  （f9cb4db3a s345 引入 getter 安装）打开：老钉 08-20 入树后在 make test 面
  从未执行过——test-guard compile-first 对首败 artifact 截断循环 + 缺件
  伪红遮蔽 page-runtime 套件（近轮 /tmp 全量日志 grep 无 zero_page_runtime
  Running 行），17:03 mkt6 首暴露。
- 剩余腿九腿补跑（zw-jsanim-legs 单元，脚本逐字镜像 Makefile test 目标
  L262-296，companion 构建前置防 disk-guard 部分清库缺件伪红——jsanim5 腿
  已实证 disk-guard rm 与并发 cargo 竞争「Directory not empty」即 make
  Error 1；LEGS-DONE 17:31:44）：绿四腿 renderer bin / GPU renderer /
  GPU consistency（GPU-PROBE=available）/ quickjs renderer bin；红五腿逐名
  对号**零新增真实代码红**：
  - renderer lib 371P/1F：`js_worker_fetch_without_handler_resolves_error`
    ——c6dcbce0a reject 语义过时镜像在册。
  - browser bin 741P/2F：tab_js_worker 同根镜像 + default_actions（均在册
    三红账）。
  - browser tests 740P/3F：前两者重跑镜像 +
    bookmark_context_menu_open_in_new_tab_loads_url **回填定类=负载相位
    flake**——`tests.rs:1682` 断言 left:1/right:2（期望 2 标签实得 1）；同测
    17:20:49（browser bin 腿）
    过、17:23:45（本腿）挂跨腿翻转，曾落 editing WIP 70cb4d2f5 共树在途
    落地 17:23:27 后 18s；发现流 solo ×3 全绿回填（17:35，`-p zero-browser
    --tests` 面 xvfb 前缀 filter 单测，EXIT=0×3=0.67s/0.06s/0.06s，
    /tmp/zw-jsanim-solo-b2.log）——solo 二进制按 HEAD 3fdbc3c53 增量重编
    （含 70cb4d2f5 刀码），混码嫌疑同证排除，多链负载窗并发唯一余因。
  - quickjs crates 775P/1F：`image_batch_e2e` 42 fetch/36 图 6 重复 fetch
    （image_batch_e2e.rs:317 "every image must be fetched exactly once"）
    = 在册伪红负载峰值形态第三现形（Makefile 补建 zero-browser 已根修
    缺件形态，负载形态仍存）；发现流 solo ×3 全绿佐证（quickjs 面
    filter 单测 EXIT=0×3，1.52s，/tmp/zw-jsanim-solo-s2c.log；方法论注记：
    solo 须带 --compile-first，缺它 rustc 编译段四并行 job 11.3G 撞 8G 总门
    EXIT=124 杀树，测试未跑即报红）。
  - quickjs renderer lib 369P/1F：worker-fetch quickjs 镜像，同 c6dcbce0a
    根。
  全账唯一真实代码红仍是 SW 具名导入 getter 一处（script-sandbox 面）。
- spec 链接：https://drafts.csswg.org/web-animations-1/ （§3.6 KeyframeEffect /
  §5.4.15 computed keyframes / §5.5 Element.animate）。

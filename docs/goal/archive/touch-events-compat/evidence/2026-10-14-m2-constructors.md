# touch-events-compat M2 — Touch/TouchList/TouchEvent 构造器面收敛（2026-10-14）

语料可执行面 4 案 32 子测试：**30 Pass + 2 中性 PreconditionFailed**（基线 20P + 10F
+ 2 PF，同日两轮 +10P / -10F，非中性通过率 66.7% → **100%**）。全量见
[2026-10-14-m2-constructors.json](2026-10-14-m2-constructors.json)。

| 用例 | M1 基线 | M2 后 |
|---|---|---|
| touch-touchevent-constructor.html | 0P / 5F | **5P** |
| historical.html | 3P / 5F | **8P**（双向门保持） |
| touch-globaleventhandler-interface.html | 1P / 2 PF | 1P / 2 PF（不变，按桌面 Chrome） |
| expose-legacy-touch-event-apis.html | 16P | 16P（不变） |

## 实现（纯 engine shim，`crates/engine/src/js_dom_shim/part05.js`）

按 spec https://w3c.github.io/touch-events/ 落三个接口：

1. **TouchList**：无 `[Constructor]`（new → Illegal constructor TypeError）；
   length/item（越界 → null）；内部工厂 `_zwMakeTouchList`（supported indices 以
   own 属性承载——`touches[0]` 与传入 Touch 对象恒等）。不实现 @@iterator（spec
   接口未声明 iterable）。
2. **Touch**：TouchInit 必填 identifier（long）+ target（EventTarget）；8 个
   double/float 坐标/半径/压力缺省 0（保小数——assert_approx_equals 20.2/35.34 面）；
   altitudeAngle/azimuthAngle/touchType（缺省 'direct'）按 spec 补齐。TypeError
   矩阵：非对象 dict / 必填成员缺席 / target 非 EventTarget（null/undefined/
   Location）。**品牌判据 = listener API 可达性**——shim 元素/document/window 各走
   独立 listener 路径无公共 EventTarget 原型链（part05 EventTarget 构造器注释），
   instanceof 判定结构性不可行；Location 为 exotic proxy 无 listener 面，同样被拒。
   readonly attribute 走原型 getter（`name in t` + typeof 断言面）。
3. **TouchEvent**：`_defineEventSubclass('TouchEvent','UIEvent', …)` 空壳改全 props
   ——touches/targetTouches/changedTouches 经第 4 元 converter TouchList 化（缺省
   `_zwEmptyTouchList` 共享空单例；`!= null` 缺省语义沿 shim R24 约定）+ 4 修饰键
   布尔缺省 false（TouchEventInit : EventModifierInit）。getModifierState 未实现
   （FIXME：语料未度量）。

`ontouch*` GlobalEventHandlers 曝出面与构造器解耦——保持缺席（'ontouchstart' in
document === false），GEH 2 PreconditionFailed 与 expose-legacy 16P 一致性面按
桌面 Chrome 形态维持。

## 门禁

- make test 71 suites 全绿（0 failed）
- make reftest 710/710（100%，零回归）
- cargo fmt 零 diff、clippy --workspace --all-targets -D warnings 零 warning
  （M1 轮同跑，本轮仅 .js 变更后 fmt 复核）
- product-smoke 未跑：DOM/JS API 面变更非渲染/布局（run-rules product-smoke 触发
  条件不符）；reftest 710 页覆盖页面语料。M3 收口轮按 DC-3 补齐。

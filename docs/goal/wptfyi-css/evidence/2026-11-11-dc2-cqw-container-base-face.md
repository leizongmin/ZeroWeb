# DC-2 cqw 容器基座刀（css-conditional-5 §container-lengths 计算面）

- 日期：2026-11-11 晚
- 流：zeroweb-2-ba（color 长尾桶 1）
- 代码：9d243bc67（part11-color.js +48/-3）
- 验证链：g8b（unit zw-g8b-cqw-2145；g8 首跑含旧 helper 版本被本流中途
  helper 修正作废，杀单元后 g8b 名重起——教训：链起跑与源码编辑不得交叠）

## 根因（g6b 轮定谳，两面）

语料页容器是真容器（`#container { container-type: inline-size }` + JS 动态
改宽 1000px→100px，期望 sign ±1 翻转）。fd2 账恒 +1（1000px 案碰对、100px
案错），供路为 shim 计算面 `_zwCParse(text, evalAtoms=true)` sign eval 的
cqw 视口×1000 近似（part11-color.js 原注「无容器上下文时按视口派生大正值」）。
host 面同期缺陷：computed.rs:253 `LengthValue::Calc` 求值 CalcContext 用
`..Default::default()`，container_width 字段未喂（container 参数在手未接；
裸 cq 单位臂有容器感知、calc() 内 cq 单位没有）——本轮修 shim 面；host 面
无独立语料驱动，随 TODO 挂账。

## 实现（四件，单文件）

1. `_zwCCqwBase` 模块基座变量 + `_zwCContainerInlineSize(elt)`：祖先链
   走查，gCS 宿主 query 通道（getComputedStyle().getPropertyValue）读
   container-type ≠ normal 命中容器，再读 width 显式 px 声明；命中但尺寸
   探不明 fail-closed 返 null（不虚构）。FIXME：auto 容器宽（需布局 used
   值）、cqh 高度轴、具名容器（container-name）未建模。
2. sign eval cq 分臂：基座在手 → cqw/cqi 按 N·W/100（css-conditional-5
   §container-lengths，cqi 水平书写 ≡ cqw）；无基座保 ×1000 兜底零漂移。
3. 入口 set/clear：`__zwComputedColorValue` 仅在 prop='color' 计算面置入
   （try/finally 清场）；declared 面与 opacity 臂不设基座，行为不变。
4. 通路核验：gCS(part01c:2352 `__zwComputedColor(raw, node, prop)`) →
   part11-computed-color sign/abs 前置臂 `_zwCCPrevComputed` 回退 →
   part11-color `__zwComputedColor(raw, node)` → `__zwComputedColorValue`
   （elt 透传）——装配序 part01 → part11-color → part11-computed-color
   保证全局名捕获正确。

## g8b 账（css-color 语料 66 页，g7 基线 10,962）

- **10,962 → 10,970 净 +8、零回归**：
  - color-computed-lab.html **4F→0F 全清**（页账 120P/0F——lab/oklab/lch/
    oklch 100px 腿 sign −1 正确翻转）
  - color-computed-color-function / hsl / hwb 各 +1（同型 cqw 案）
  - color-computed-rgb 94p/5f → 95p/4f（+1；残 4F 属桶 4 转义/var 域）
- calc-container 页 0P/2F 未动：该页 prop=background-color，
  `__zwComputedColorValue` 恒 null 返（只接 'color'），走
  part11-computed-color 通用解析通道（自带 chanexpr calc eval，无 cq
  基座）——预期内遗留，另刀接通用通道。

## 残余挂账（桶 1 余量）

- calc-container 2F：part11-computed-color 通用通道 cq 基座（含 contrast
  内参 + 写面 canonicalization 1F 独立面）。
- color-mix 100em 案：em 元素字号面（evalAtoms em 硬编码 16px，未读元素
  font-size 上下文）。
- host 面：computed.rs CalcContext.container_width 未喂（样式表槽位 calc
  内 cq 单位）——无独立语料驱动，TODO。

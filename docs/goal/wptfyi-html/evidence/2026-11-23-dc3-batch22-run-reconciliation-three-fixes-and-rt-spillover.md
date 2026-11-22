# DC-3 · 批二十二：批二十一 run 45028417732 收账——三修复兑现核验＋RT 切片 2 溢出面

2026-11-23。run 45028417732（cf288ecd2，批二十一语料修复＋对端 engine 切片
2/3/4/5 同 run）产物与批二十一 run 45017894086 同分片逐键 A/B；快照
c22c3c0de（15:49Z 落 main）。

## 目标级账

html 面 112,102 → **109,404（−2,698）＝净 +38,308/基线 71,096，验收
≥152,096 余 42,692**。过滤纠偏是记账回吐非增益，面缩为预期内。

## 三修复核验（批二十一承载清单逐项）

1. **过滤纠偏 −3,002 精确兑现**：三 dom 查询键回归真过滤形——
   `?include=(Document|Window)` 48P/298 注册、`?include=HTML.+` 1,003P/3,896、
   `?exclude=...` 426P/1,628，合计 1,477P/5,822 注册，与本地探针
   （48/1,003/426＝5,822 注册）逐字相同；**同值检测三键子测试集互异**
   （批二十一定谳的全页虚增指纹消失）。隔离律第 7 例闭环。
2. **non-object +4 精确兑现**：1P/4F → 5P/0F，4 个 json module-fetch
   Fail 消失。隔离律第 6 例闭环。
3. **html5lib dat +98/预期 +118**：url 驱动 60 键兑现 98P/442F/38TO；
   **write 与 write_single 双驱动 110 键全键级 Timeout（0P）**＝
   document.write 管道挂起已知 engine 域（批八刀位/批十二战役图同域，
   CI 实跑首证），预期差 −20 挂账该刀位，不另立新账。

## 预期外正向位移 ＋204：对端 RT 切片 2 溢出

窗口内唯一引擎刀 ec4ef28dc（RT 切片 2 D 面 ref-target 三矩阵注册门解锁，
12:04Z 落 main，在批二十一 run checkout 之后）对 html 面溢出：

- popover-invoking-attribute.html 0P/1,402F → **204P/1,198F**（+204，
  popovertarget/popoverTargetAction 反射与 invoker action=toggle/garbage/
  null/undefined 形全翻绿）——popovers 战役图（2,744 F/66 键）首块非刀位
  兑现，页面余 1,198F 仍属 popover 语义域待碰头。
- commandForElement reflects、interestForElement reflects、
  button-event-dispatch 各 +1。
- **interestfor-input-invalid −3**：interestForElement 现存在于
  input types，`assert_false should not be available` 三测反转红＝
  reflection 注册范围与 spec 边界差，归 RT 流域定谳（不入本流修复面）。

净 +204。批十六曾预登记「同 run 观察 dom/ 面对端切片正向位移」——本批
html 面内同样出现，照实入账并标注对端归因。

## 邻账（不入 html face）

- dom/ +222＝Range-mutations-dataChange 930→1,152（5e217bdba
  CharacterData 切片 3，批二十一预判兑现；wptfyi-dom 账本）。
- web-components RT 三键 30/30/31 → 2,910/2,910/2,911 注册全开
  （+8,640P，tentative 形）＝ec4ef28dc 三矩阵注册门解锁同源，CE·SD 邻账。
- sanitizer/trusted-types 通道首批 CI 入账 39P/31P（829c56fb1 兑现）。

## 持位与残差扫描

webappapis 1,537／infra 674／rendering 2,795／speculative 43／
import-maps 37／xhtml 25,606 全持位零漂移；onblur flaky 4→3 带内回摆
（两 run 净 0）。合成 Fail 例行扫描：html 面 fetch-module 形 3 键→2 键
（non-object 销账），余 blob-url.any.js 等 module-map 域 2 键挂账不变；
os error 2／subsetTestByKey 形两 run 同形持位（fetch/api 两键同律异道，
不入 html face）。

## 面差对平

raw 逐套件合计 −3,002（dom 键）+208（semantics 补簇＝popover +204＋
non-object +4＋interestfor −3＋command/interest +2）+98（syntax dat）−1
（semantics 套件 flaky）＝−2,697；快照 face −2,698，记账余差 −1（快照
去重/对齐规则粒度），不再追。

## 下轮 run 观察清单

write/write_single 110 键 TO 形持续（engine 刀位落地前恒定）；
interestfor −3 是否随对端 RT 后续切片变化；dom/ 面对端切片 5（TT）
无 html 面溢出（本 run 已证：TT 键全在 trusted-types 通道）。

[逐文件 JSON](2026-11-23-dc3-batch22-run-reconciliation-three-fixes-and-rt-spillover.json) ·
[批二十一证据](2026-11-22-dc3-batch21-run-reconciliation-and-subset-key-shard-gap.md) ·
[前置账](2026-11-22-dc3-goal-accounting-and-corpus-runway.md)

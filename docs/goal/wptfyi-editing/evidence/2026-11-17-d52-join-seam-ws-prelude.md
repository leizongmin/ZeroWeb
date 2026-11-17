# d52 join 缝空白序言刀（引擎 removeChild 前导空白塌方修复）A/B 证据（2026-11-17）

## 根因链（zz-probe-d52a 探针 + shim trace 一轮，patch 已全量回退）

d51 停车场项「FD ws 形块界 join 内容塌方（content `<div>ghi</div>`）」本轮收口：

1. **形状隔离**：九面变体探针（全 ws 形/plain 控制/li 尾 ws/li 头 ws/源前导 ws/
   abc 尾 ws/caret-at-len/repeat/BS 镜像）——塌方的唯一判别面 = **提取源行首
   文本节点带前导折叠空白**（T5 `<li>def[]</li>` + `  ghi` 单独复现
   `<div>ghi</div>`；li 尾/头 ws、abc 尾 ws 全部正确）。
2. **trace 轮**（J0-J5 五点入 JoinApply/JoinMutate，backup 比对零残回退）：
   shim 侧全程正确——J1 takeLine n=1 提取正确、J3 shellDst=LI/needsShell=false
   正确、J4 ser sel=li pos=beforeend 正确；J5 exit 时尚未 drain。塌方发生在
   **引擎侧 drain**：removeChild(前导空白文本子) + insertAdjacent(beforeend)
   的 wire 序在引擎侧解释为「前序内容整体摘除 + 文本落宿主级」（`<div>ghi</
   div>`）；同 wire 序去前导空白（T2 形）恒正确。T1（同形首位）bi=0 但
   content 正确——派发代际差异再证 ③ 域。
3. **修复（JoinMutate 提取序言，共享路径）**：源行首文本节点在 takeLineFrom
   **之前**摘前导折叠空白——守卫 `srcParent !== dst` + 双侧 normal 域
   （preserve/pre-line 不入）+ 落位块元素；先摘后提后 C-strip 头部支自然
   no-op，wire batch 变为 T2 形（remove("ghi") + insert("ghi")，实证恒正确）。
   BS/FD walk join 与 select-delete 键入替换面共用 JoinMutate，一处全覆盖。

## 权威账（专项 A/B，base=61ebaf369 d51 收口）

| 腿 | d51 base | d52 wip | 共键 F2P | P2F |
|---|---|---|---|---|
| backspace 主文件 | 79P/84F (163) | 81P/82F (163) | 2 | 0 |
| forwarddelete 主文件 | 98P/78F (176) | 100P/76F (176) | 2 | 0 |
| non-collapsed 三 variant | 101P (207) | 106P (207) | 5 | 0 |
| joining-dl 两文件 | 751P (1174) | 751P (1174) | 0 | 0 |

**净 +9（F2P 9 / P2F 0）**。F2P 明细：FD `def[]` ws 形 content 子测 ×2（d51
promise 键已 P、内容残余清偿）；BS caret-after-ul ws 形 `[]ghi`/`[] ghi`
content 子测 ×2（bi=2 双派 promise 键持位 ③ 不动，内容键兑现）；NC 列表
选区面 Backspace/Delete/TypingA ×3 + `<div><p>abc   [</p>]   def</div>`
块界选区面 ×2。

## 哨兵

- enginelib：**4134P/0F**（test-guard --compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 1500）
- selection CE 过滤（23 案）：对 /tmp/d49-selce-base.json 共键 **414 全同
  F2P=0/P2F=0**
- input-events 全套件：对 /tmp/d51-suite-wip.json 净差 **F2P=20/P2F=0**
  （核心 +9 + zz-probe-b3p/b3q/b3r/d46b 镜像面自增 11）；对
  /tmp/d49-suite-base.json 粗核共键 F2P=289/P2F=0
- joining-dl 对 /tmp/d49-jdl-final.json：共键 1174 F2P=0/**P2F=0**（重 join
  消费面零扰动，jdwsp 反证面不受影响）

## 探针终态（zz-probe-d52a 修复后九面）

全数 content 契约形：T1/T7/T8/T9 全 ws 形 `<div>abc <ul><li>  defghi</li></
ul></div>`（语料期望数组第一形）；T5 plain 化 `<div>abc<ul><li>defghi</li></
ul></div>`；li 尾/头 ws、abc 尾 ws 形态保留正确。T1 bi=0 首位派发鬼持位
（③ 域，不本轮面）。

## 运行产物

- /tmp/d52-{bs,fd,nc,jdl}-wip.json、/tmp/d52-selce-wip.json、
  /tmp/d52-suite-wip.json、/tmp/d52-probe{,2,3}.json
- /tmp/part01.js.d52-orig（trace 轮 backup，diff 核零残）
- zz-probe-d52a.html（wpt-data 本地不入树）

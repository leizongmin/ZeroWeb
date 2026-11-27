# s10k57 刀②b：face B 收口——HTMLScriptElement-internal-slot 页级 Timeout → 9/9 全 Pass

日期：2026-11-27。刀体 commit `52e061cf0`（part03.js + part14-trusted-sinks.js，51 增 8 删）。guard 扫描 PASS（0 发现 0 提醒）。

## 基线签名（刀前）

`trusted-types/HTMLScriptElement-internal-slot.html`（TT 电池）＝ test1 一行 Pass + 页级 Timeout。原始日志决定性证据：

```
3 test(s) unreported (state={"loaded":true,"pending":3,"phase":4,"tests":4})
```

`pending:3` ＝ promise_test 顺序执行链卡死：test2 形（connected 空体 append 后 `textContent = policy.createScript(...)`）的 promise 永久 pending → tests 3-9 body 根本不运行 → R-diag 类定时器永不注册 → 页级 Timeout。不是 9 个独立失败，是 1 个挂点饿死全页。

## probe 链定谳（zz-s10k57-probe1..4，均已删，日志摘录存此）

1. **probe1 形状隔离**：A 形（先 append 后 textContent）置章零执行；B 形（textContent 先于 append，unconnected）执行正常。
2. **probe2 真页复刻**：R1（write-face）/R3（unconnected 形）健康；R2（A 形）no-hello——唯一真挂点＝test2 形。
3. **probe3 分步隔离**：空体 append / textContent 调用均无同步阻塞；write-face 状态后 R3 形 Pass——排「调用本身死」假说。
4. **probe2b settle-timeout 版**：全健康一致，排除资源类挂死。
5. **testharness 顺序执行机制定谳**（合并 2+3）：`pending:3` ＋ tests 3-9 unreported ＝ promise 链饿死，非多点失败。
6. **刀面 v1 后 probe4v2**：hook 已生效（`ranKeys` 出现 `__n0`＝prepare 真被调）但仍无执行、`msgs=[]`——拦截点在 prepare 内部更深一层的置章臂。
7. **probe4v4 直调双门**：写完后页域直调 `__zwTtScriptExecGate` 判 `undefined`（盖章快照查中）、`__zwCspInlineScriptAllowed` 判 `1`——两门在写后状态都放行。
8. **probe5 包装门**：setter 期间包装 `__zwTtScriptExecGate` 记录——**门在写窗内被调、同文本判 `null`**。

**根因三重**（叠加缺一不复）：

- **R-a（置章错误域）**：连接空体 script 插入 → prepare consult TT 门；页面 default policy 只有 createHTML（无 createScript）→ sink 转换抛 TypeError → 门返 null → 旧代码无条件置 run-once 章。而 spec 的 HTML prepare 空体早退**不置 already-started**、TT prepare-script-text 步 2 在空体（slot==child 初值）根本不进转换——置章是超出 spec 域的误记账，后续任何 text 写被章门短路永不执行。
- **R-b（零触发）**：accessor 写树后无再 prepare 触发点——spec 的「script child text content changes」步骤缺位。
- **R-c（写窗时序）**：修 R-a/R-b 后仍挂——probe5 实证 `Reflect.set` 落树经 s111 `_mo_notify` 路由在**写窗内同步**进 prepare consult 门，彼时 setter 尾部的快照盖章尚未落 → 快照查恒失配 → 管道转换后的文本被判违规阻断 + 置章（probe4v4 写后直调同文本 `undefined` 为对照）。

## 三修（commit 52e061cf0）

1. **part03（R-a）**：TT 门 null 臂改条件置章——`_ttGate109In.trim() !== ''` 才置（非空 TypeError＝真实违规阻断，照旧）；空体不置章。救援返空置章臂不动（script-enforcement-005 变空面 [C,S] corpus 钉面）。
2. **part14 setter（R-b）**：text/textContent/innerText 写后触发 `_zwScriptMaybeStartDynamic`（globalThis 路由，part03:6016 导出；章门/connected/type 门内置于该函数，已执行件短路零效应）。
3. **part14 门首查（R-c）**：`__zwTtScriptSinkWrite` 落树计数旗（仅 6.4 三名 script accessor 打旗；Node/HTMLElement 通用 accessor 不打旗——002 descriptor 直调救援语义保真）＞0 且文本非空 → 判可信直过。空体不判可信（005 变空面不增 RUN）。accessor 期管道 TypeError 在窗开前抛出，窗外无此 consult 路径。

## 电池判读（chain58，13:15:05Z→13:48:14Z，bin `d637b6cedade`＝commit 字节）

| 臂 | 锚（chain13） | 实跑 | 真实语料归因 |
|---|---|---|---|
| tt58 | 2282P/14F/19T | 2294P/18F/18T | ΔP=+8（internal-slot 页 1→9 子测）；ΔF=0（14 真实行逐字节恒等）；ΔT=−1（internal-slot 页退出 Timeout）。+4P/+4F 全为 zz-s10k57 探针页噪声（gitignored，DIAG-by-reject 设计如此，电池后已删）：2282+8+3+1=2294 闭合 |
| dom58 | 57463P/361F/12T | 57399P/361F/12T | F/T 四象限 comm 全空（361=361、12=12）；ΔP=−64 全落 Range-mutations 双页（−35/−29，已知截断抖动带 ±150 内）＝截断落位非回归 |
| ref58 | 725/725 | 725/725 | Layout 521/521、Text 204/204、不一致 0 |

**TT 真实池净变化**：internal-slot 1F+页T（ Timeout 列）→ 9P。script-enforcement 家族零漂移。

## chain13 bin 账面订正

chain13 发射时断言 `e0cd206f5f55`，但臂内 make 重编穿透断言（实跑 `72ba716f4a5b`）。查明：make 配方 `cargo build --release --bin zero-wpt-runner` vs 直跑 `-p zero-wpt-runner` 调用形状差触发重建；12:17Z 重建时树内恰为刀② 两文件未提交态＝语义同源零字节漂移，chain13 判读仍有效。本刀②b 起走「先 commit 刀体→再发射电池链」协议（本轮已执行：bin＝commit 字节 md5 断言在发射行内）。

## 残相挂账

- 探针 selector 包装面 tagName 大写（上轮注记，未动）。
- SVG textContent 通用 accessor 无盖章（本轮 scope 外；SVGScript 池 0F 现状保持）。
- census 84,324（刀② 后专跑值）——本刀本地 ΔP=+8，census 下次专跑刷新时并入。

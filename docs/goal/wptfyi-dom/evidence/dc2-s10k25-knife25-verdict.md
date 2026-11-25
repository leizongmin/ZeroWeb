# DC-2 切片 10 刀㉕收账——DOMParser 产物文档 write/writeln 隐式 open 物化（TT 133→131＝刀域 6F 全清，五腿键集净 -1＝环境敏感红面 8 键抵消）

**日期** 2026-11-26　**判定** PASS-域（刀域 Document-write-appending-line-feed 6F→0F 全清＝debug/solo/电池/钉测四证；五腿键集净 -1＝HTMLElement-generic 与 Range-mutations-replaceChild 两页环境敏感红面 8 键非刀因暴露〔刀前刀后 solo 恒现实证〕− deleteData 抖动页转绿；WC/DP/SAN SET-IDENTICAL；Timeout TT 65 恒）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 刀㉕ `0fe7a0b2c`〔谱系 0fe7a0b2c＝发射锚→收账前 pull 携入兄弟流 s763 docs 重写＝`d90962c18` 推送终态；patch-id 恒等验证〕（part02 `_zwParsedDoc.prototype` 补 write/writeln＋part174 钉测新文件＋bridge_tests 注册行） | parsed-doc 写面主刀 | `bf23c9948c0f8d1db42d61ab08a89230` | v10 |

发射器 `.acceptance/dps/s10k25-battery-v10.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；16:52–17:26Z（≈34min，WC 5.5m/DP 13s/SAN 17s/TT 16m46s/DOM ≈11m）。发射后收账前 `git pull --rebase` 携入 3f5216a08（html-semantics s763 docs）重写刀提交（0fe7a0b2c→d90962c18，diff 逐字节恒等）；判读二进制 md5 不受影响（build 在 rebase 前，源码相同）。

发射器 `.acceptance/dps/s10k25-battery-v10.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；16:52:43Z 起跑。

## 根因链（归因→机制→刀）

1. **缺位面**：Document-write-appending-line-feed 6F 全形态 `doc.write/writeln is not a function`——语料在 `DOMParser().parseFromString('<body></body>', "text/html")` 产物上调 write/writeln 断言 body.innerHTML 精确值；产物 `_zwParsedDoc` 轻量对象无该方法。
2. **机制考古（真 Chrome 决定性证据）**：headless Chrome 探针（本地语料 server 跑上游页＋内联探针页）实证——TT enforced 页 parseFromString 源串过 default policy 管道（'[<body></body>]'），**产物 body 初始 "[]"**；但 face1 write 后精确 "(1)(2)(3)(4)"＝初始内容被清非叠加。机制＝spec write steps 的 write-writable 判定（要求 active parser 未中止）：parsed 文档解析完即无 active parser，恒 not write-writable → **每次 write 先隐式 open（清空）再写入**。同模型解释 block-string-assignment-to-DOMParser-parseFromString 页共存（只 parse 不 write → 管道产物留存）。
3. **刀（part02）**：`_zwParsedDoc.prototype.write/writeln`——独立 inert 文档不走主文档写流（part06 buffer/Base 全局单例），物化进本产物 body（_zwParseEl innerHTML 数据面）；每次调用替换语义（隐式 open）；TT 管道镜像主文档 write 口径（逐参 String 拼接、含 plain 分量→拼接串整体过一次 sink 管道、全 Trusted→直通；writeln '\n' 管道后置，sink 名 'Document writeln'；非 enforcement 页 plain null → '' 直入）。
4. **保信**：debug 单页 6F→0F（Document-write 族 34 Pass 零 Fail）；part174 钉测 enforced harness 七断言（方法面/Trusted-only 直通/plain default-once/混合参/writeln 尾 \n/parse 管道初始 "[]"/隐式 open 替换核心钉面）；s10k2x 全家 13/13。

## v10 判读（锚＝k24 v9 终态 TT F 133 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364；T 键集 TT 65 / WC 20 / DP 1 / SAN 1 / DOM 13）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **133→131（Pass 1949→1951，+2）** | **GONE 6＝appending-line-feed 全部 / NEW 4＝HTMLElement-generic 竞态** | 65=65 键集恒 |
| WC | 1116→1116 | 0/0 | 20=20 恒 |
| DP | 222→222 | 0/0 | 1=1 恒 |
| SAN | 50→50 | 0/0 | 1=1 恒 |
| DOM | 364→368 | GONE 1＝deleteData 抖动页转绿（solo 564P）/ NEW 4＝Range-mutations-replaceChild 环境敏感红 | 13→12（deleteData 键） |

### HTMLElement-generic NEW 4 归因（非刀因实证链）

1. NEW 4＝`TT enabled: script.src` 四变体（Trusted/String × connected/non-connected）——动态 meta 注入 enforcement 页，plain/错型 Trusted 赋 script.src 应抛 TypeError。
2. **刀前刀后 solo 对照**：`git show 8485db7a7:...part02.js`（刀前）临时覆盖构建 debug solo＝4 Fail；恢复刀版 debug solo＝4 Fail；release solo 连跑 3 次＝恒 77P/4F——回归与刀无关（刀 patch 单 hunk part02.js:6202 纯插入 83 行，域完全隔离于 part14 script.src sink 与 enforcement 判定）。
3. **电池 worker 分裂**：同 face 同电池 4 Pass＋4 Fail 并存（v7/v8/v9 三锚电池全 Pass）——enforcement 脏位/重扫路径存在时序竞态，solo 跑法稳定落 Fail 窗、电池多 worker 非确定分布；appending-line-feed 页转绿改变页序/执行时序可能提高窗口暴露率。
4. **挂账次刀**：part14 `__zwTtCspDirty` 动态注入生效窗口审查（返修② 013 face「同块内生效」语义在 solo 跑法下的盲区）；TT 残 131F 按簇另记。

### Range-mutations-replaceChild NEW 4 归因（非刀因，同族环境敏感）

solo（刀前 debug＋刀后 debug/release 三形态）恒 56P/4F（`paras[0].replaceChild(paras[1], paras[0].firstChild)` 4 变体）；v9 电池该页 60 face 全 Pass、v10 电池 56P＋6F+2T 分裂——非确定红面与 HTMLElement-generic 同族（多 worker Pass/solo Fail 分歧）。同电池 deleteData 抖动页转绿（solo 564P 全绿，刀㉔ 收账已证其电池 Timeout＝争抢抖动）。两红面 runner 跑法×引擎时序分歧审查挂账下刀（runner 域）。

## 质量门禁

fmt 零 diff；clippy `-p zero-engine --lib --tests -D warnings` 0 warning；提交 lei-pre-commit-guard PASS（3 文件 170 行 0 发现）；release 重建 44s（test-guard 包裹）。

## 累计

DC-2 累计 83,814→**83,813**（五腿键集口径：TT +2〔刀域 6 face 全清〕＋DOM -3〔replaceChild 4 face 环境敏感红 − deleteData 抖动转绿 +1〕＋WC/DP/SAN 0＝净 -1）；距 ≥99,947 差 16,134。刀域收益实打实（debug 单页 6F→0F＋solo＋电池＋part174 钉测四证），环境敏感红面 8 键（两页各 4）为跨刀非确定暴露，其净额波动不掩刀域收敛。

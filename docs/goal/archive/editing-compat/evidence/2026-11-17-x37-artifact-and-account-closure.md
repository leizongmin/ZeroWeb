# X37 双键归因普查收口＋fb32/fontSizeT1 挂账清账（2026-11-17）

本轮三项：① X37 双键（mt subscript/formatblock 残红）归因普查＝**窗界伪影定谳，非引擎缺陷**；② fb apply 侧 ~32 族挂账清账（逐族消费归宿）；③ fontSize T1 ×3 划转账清账（evt-reject 刀兑现）。另注：本轮 rawtext 壳刀 scoping 发现该刀域已被 ihe5/ihe6 消费（iht 0F），挂账时序勘误入档。

## 一、X37 双键＝语料窗界伪影定谳

X37＝`editing/run/multitest.html?2001-3000 :: [["subscript",""],["formatblock","<div>"],["inserttext","a"]] "foo[]bar"` 双读键：

- `queryCommandState("subscript") after` 期望 `true` 实得 `false`
- `queryCommandValue("formatblock") after` 期望 `"div"` 实得 `""`

恒一在册多轮、历刀零位移（ihe1-ihe6/img/R13 各批 mt 腿残红均恰此双键）。

### 机制（四证链）

1. **子测创建序**（tests.js `runConformanceTest`）：每变体 23 子测按创建序＝3 execCommand return → non-editable check → compare innerHTML → **18 读断言（最后创建）**。`subset-tests.js` 的 `subsetTest` 按 location.search 窗界（?2001-3000＝[2001,3000]）单调计数器决定子测创建与否，**窗外子测不创建＝函数体不执行＝命令根本不跑**。
2. **索引算术**：该变体 23 子测块恰骑跨 2000|2001 窗界——前 5 子测（命令×3＋检查×2）索引 ≤2000 落 ?1001-2000 窗，18 读断言索引 ≥2001 落 ?2001-3000 窗。单窗实测：?2001-3000 页读断言 18 键全在册（Pass 16＋Fail 2）而 execCommand/compare 子测零在册（grep 全 log 无 subscript 变体 return-value/compare 行）。
3. **窗内页实跑取证**（temp 页 hook：execCommand 包装器＋sync/queued 双 dump 测试，探针后 multitest.html 已还原备份、md5 `d0aceab6b2f33dc44da87e3c17c750c8` 复核＋variant meta 10 行＋hook 串 0 残留）：命令被跳过 → **全程树滞留初始 `foobar`**（sync dump 与 queued async dump 双读恒 initial、caret text(foobar)@3）→ 读采样（runConformanceTest 内同步代码，无条件执行）得初始值 false/"" → 双红。日志 `.acceptance/dps/x37-probe-win2.log`（单窗 998P/3F＝恰 X37 双键＋2 dump 探针测试，其余全绿）。
4. **引擎无罪双证**：①全量 mt 轮（各窗合跑）中该变体 `?1001-2000 :: … compare innerHTML` 绿（命令+compare 同窗执行），且全量残红仅 ?2001-3000 两读键；②沙箱 pin 探针（bridge 同步 flush，temp 测试读数后删除）逐步快照全绿：subscript（树不变＋qS 翻 true）→ formatblock（`<div>foobar</div>`＋qV "div"）→ inserttext（**`<div>foo<sub>a</sub>bar</div>`＝语料期望字节恒等**，caret text(foo)@3，双读 true/"div"）——apply 面与读面对该变体全绿。

Chrome 同窗同红（窗界页命令同样被跳过、读采样同得初始值）——非录音差异、非引擎缺陷，**语料上游窗切伪影**。

### 处置

X37 **除名转正挂账**：mt 门期望 10090/2 维持，残红恒 X37 双键＝窗界伪影锚，后续刀不再为它立项。真实引擎残红域修正：mt 2F 伪影出册后，editing 语料真残红＝fb 3（dl qI ×2＋dPS qV ×1，除名挂账）＋indent 21＋outdent 4（dPS-qV ×2＋ws-bq ×2）＋ip 59＋fs 7＋event 4（Chrome 同红）。

### 方法注记（复用价值）

- 本 runner 的 corpus 页 FILTER 是**文件路径子串**匹配（`testharness_editing_corpus.rs` :123 对 `relative` 匹配，variant query 不参与）——单窗跑法须临时删页内多余 `<meta name=variant>` 行（已还原）。
- mt 页脚本对 testharness 已处 run-phase：`test()` 同步执行子测函数体；窗外子测不创建是**行为等价的跳过**，非延后。
- 骑界变体的红键面 = 骑界点之后创建的子测种类：骑界落在读断言段→读键红；落在命令段→compare/check 红。普查新残红键时先查该键变体的子测索引是否骑窗界。

## 二、fb apply 侧 ~32 族挂账清账（2026-11-10 立账，全数消费定谳）

立账原文（11-10 fb qV/qI 读面刀 evidence）：「apply 侧包裹形族 ~32（color:blue 样式携带/xmp-listing 包裹/table tr 跨胞/div>p/blockquote 嵌套拆分/hgroup/h1<br> 合并/del/p→h5 dps-div 交互及关联 qV-after 读）」。逐族归宿：

| 族 | 归宿刀 | 证据 |
|---|---|---|
| xmp/listing 包裹 | ⑪ d599989af（fb-xmp-listing-wraponly） | F2P=8 恰 xmp/listing compare，fb 5015/31 |
| color:blue 样式携带（compare+qV-after 双形） | ⑫ 5bd748fea（fb-slc-dissolve-carry）＋⑬ 交叉注记 | F2P=16 全中刀域双形；⑬ 记「color:blue ×8 已修翻绿（4 compare＋4 qV-after）」 |
| table tr 跨胞 ×4 | ⑯ tq v7 终绑 13535525b | fb 5033/13 F2P=4 恰 table qV-after 白名单 |
| div>p 摆位 ×2 | ⑭ 3c622f21a（fbwrap-divp-placement） | F2P=2 恰双 dps 行 |
| blockquote 嵌套拆分/hgroup/h1<br>/del/p→h5 | ⑮ b3417f88b（fbpara-per-paragraph）＋残账重组 | ⑮ F2P=4 恰族签名；残 17F 随后分解：table ×4→⑯、9 键窗界伪影定谳除名（修复页翻绿实证）、真差异 1（cE=false span wrap）→R13 |
| 关联 qV-after 读 | ⑫/⑯ 覆盖（上两行 qV-after 面） | 同上 |

终态对账：fb 现 5043/3，残 3＝dl qI ×2（R3/R4 不可满足互斥对伪影）＋dPS qV ×1（R7 录音复位伪影）——**均不在 ~32 族清单内**（属同轮另立「dps-qV 5＋qI 泄漏 2」账），且已由 11-11 残账重组除名挂账。**~32 族挂账清账（消费完毕）**。

## 三、fontSize T1 ×3 划转账清账（selection-compat-v2 划转 → evt-reject 刀兑现）

划转账原文：`editing/event.html :: Command fontSize value ""/"quasit"/"15px" input event ×3——期望 input 事件 1 次实得 0`。evt-reject 刀（49d8e758f，part06 值拒绝路径按形补派 input）收口：event.html 173P/7F→**176P/4F，三真差异键翻绿（恰 fontSize T1 ×3 域）**，残 4 键＝Chrome 同红除名挂账（批 i 后继轮独立复核逐键在册）。**划转账清账**。

## 四、时序勘误：rawtext 壳刀域已消费（本轮 scoping 发现）

R13 收口轮把「下一步＝fb32＋T1 普查归域」写入 Active 时，ihtml 轴已在 11-16 完成 ihe4（A 族 14→9F）→ihe5（rawtext 手术臂 7668cf495，9→5F，**即「rawtext 壳刀」本体**）→ihe6（E/F/CEF 三臂 96fa8a10a，5→0F，iht 页面全绿）。本轮按 ihe4 记录的「下一步＝rawtext 壳刀」开工 scoping 时发现目标已消费——ihe4 声明的下一步被 ihe5 在同日兑现，挂账时序以本节勘误归档；rawtext 根因分析（拼接臂字面重建串经宿主 SetInnerHtml→rawtext 解析吃标记）以 ihe5 归因档 85d5d8c94 为准。

## 五、下一步（刀序重锚）

真实残红簇按规模/清晰度排序：**fs 7F 分类普查定刀序**（`<p>[foo<p><br><p>bar]` 同形 ×4〔swcss T/F×compare/qI-after〕单族主导＋table qV-after ×1＋font qI/qV-after ×2＝3 族）→ indent 21F 八微族（census 在档）→ ip 59F（M3 census 在档）→ outdent ws-bq Blink 偏离臂 ×2（小而清晰，dPS-qV ×2 挂账维持）。

# DC-2 切片 10 刀㉓收账——TT fromLiteral 静态方法（from-literal 21F 清零，21 throws 假绿面真 TypeError 保信）

**日期** 2026-11-25　**判定** PASS（TT 173→152＝净 +21 face，GONE 21 全 from-literal 页/NEW 0；WC/DP/SAN/DOM Fail/Timeout 集逐键 SET-IDENTICAL）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 刀㉓ `e29837c1f` | part12 `fromLiteral` 静态方法三接口（IsTemplateStringObject 近似＋substitutions 门＋TrustedHTML 归一化）＋part172 钉测五向＋负面页 | `23f1b443c37cf201045e7216fa1dfe7d` | v7 |

发射器 `.acceptance/dps/s10k23-battery-v7.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；墙钟 WC 7m41s/DP 13s/SAN 17s/TT 16m43s/DOM 11m04s（13:34–14:10Z）。

## 根因链（归因→刀→保信设计）

1. **缺位面**：`trusted-types-from-literal.tentative` 21F＝三接口 ×（supported＋Basic/Empty/emoji/Multi-line/Bound 5 works＋TrustedHTML Normalization）；21 throws 面（每接口 7：string/null/undefined/Object/数组/frozen 数组/表达式/non-literal with param 中 7 面）旧缺方法 TypeError **意外绿**——落刀后须以真 TypeError 保信防翻红。
2. **刀**：接口对象 own 静态 `fromLiteral`（writable/non-enum/configurable）——`IsTemplateStringObject` 近似（模板对象内部槽 JS 层不可见，spec 以 frozen 数组＋frozen raw＋长度一致近似：`Object.freeze(["abc"])` 无 raw 抛＝frozen non-literal 面）＋`arguments.length > 1` 抛（`` `a${x}b` `` 表达式面）＋TrustedHTML 臂 parse-then-serialize 归一化（bypass 单次透传＋createHTMLDocument 产物 div innerHTML roundtrip——fromLiteral 是 Trusted 构造面不经 enforcement；`<b>`→`<b></b>`），Script/ScriptURL 臂不归一化。产物经 `_zwTtCreate`（Object.create(iface.prototype)）＝instanceof 天然成立。
3. **保信**：21 throws 假绿面全部以真 TypeError 保持（判读 NEW=0 证实）；敏感页抽查 default-policy 域 7 页 Fail 分布与 v6 锚逐页恒等（9/4/3/2/2/2/1）＝零翻红。

## v7 判读（锚＝k22 v6 终态 TT F 173 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **173→152（Pass 1909→1930，+21）** | **GONE 21 全 trusted-types-from-literal.tentative / NEW 0** | 65=65 恒 |
| WC | 1116→1116 | 0/0 | 20=20 恒 |
| DP | 222→222 | 0/0 | 1=1 恒 |
| SAN | 50→50 | 0/0 | 1=1 恒 |
| DOM | 364→364 | 0/0 | 12=12 恒（v6 的 Range-mutations-deleteData 截断抖动回稳） |

## 常驻钉测

part172 两测：V1 works 基形（in/instanceof/toString 精确值 basic/empty/multi-line/emoji cooked 转义）＋V2 归一化域（HTML parse-then-serialize、Script/ScriptURL 直通）＋V3 throws 保信（非模板形态六种＋表达式＋with param 全 TypeError）＋V4 bound method＋V5 品牌保真（产物入 enforced sink 不抛＋isHTML）；N 负面页（无 meta 构造与强制无关＋throws 形不变）。门禁批内 2/2 绿；s10k 全家 27/27。

## 质量门禁

fmt 零 diff（css-parser fmt 连带照例回退）；clippy `-p zero-engine --lib --tests -D warnings` 0 warning；提交 lei-pre-commit-guard PASS（3 文件 200 行 0 发现）；release 重建（test-guard 包裹 46s）。

## 残面挂账（次刀输入）

TT 残 152F：block-string-assignment Document-write 10／default-policy-report-only 9／default-policy 页 3（script.src/div.innerHTML/script.text null）／eval-csp-tt-no-default-policy 4／createHTMLDocument 残 5／script-enforcement-008/009 各 5 等；from-literal 全清（45/0）、secondary-document 全清（97/0）、set-attributes-require-*default-policy 全清（264/0）。

## 累计

DC-2 累计 83,774→**83,795**（刀㉓净 +21，五腿电池口径）；距 ≥99,947 差 16,152。

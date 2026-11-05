# M2 Range-mutations dataChange 簇修复（2026-11-05）

## 簇规模与归因

M1 基线 `Range-mutations-dataChange` 369F@10s 截断（sanity 96 + wrong
offset 273）。探针页（testharness-dom 通道直跑，已删）证认三处根因，跨
shim 三域：

1. **part04 handle/proxy 域**：R3034 `.data`/`.nodeValue` setter 写前漏
   live-range 边界调整（R260 注释声称「_write 内统一调整」但从未实现）；
   textContent set 块无 Text/Comment 臂（落通用元素「string replace all」
   形态，写不达 `.data`）。
2. **part06 `_zwRegisterTextEl`（textEl 域）**：`textContent=` 建的本地
   文本节点——common.js `paras[0].firstChild` 的主体域（探针
   `dataDesc=accessor` + `tcDesc=value` + insertData 臂 adjust 触发而
   data-set 臂 `calls=[]` 实证）。data/nodeValue setter 漏调整；
   textContent 为构造字面 plain 字段，赋值不同步 `.data`。
3. **part05 `_wrapNodeEntry`（parsed sel 域）**：同 1/2 形态（本族
   accessor 此前仅方法臂有调整）。

修复前失败行分布（285F@截断）：`paras[0].firstChild` 188 +
`paras[1].firstChild` 97——**全部** textEl 域，与探针证认一致
（foreign/detached/xml/comment 行经 part04 修复已绿）。

## 修复内容（spec：`concept-cd-replace`；R129 LegacyNullToEmptyString）

- `part04.js`：R3034 setter 写前补 `__zwAdjustRangesForData`；textContent
  set 块补 Text/Comment 臂（adjust + handle 持久化 + characterData record）。
- `part06.js`：textEl 工厂 data/nodeValue/textContent 三 accessor 统一
  （adjust(0, 旧长, 新长) + R129 转换 + `_regWrite`）；`_regWrite` 去掉
  `textContent` 回写（防 accessor→setter→_regWrite 互递归）。
- `part05.js`：data/nodeValue/textContent accessor 补 adjust；`_write`
  同款互递归防护。

## 验证（均 test-guard 包裹）

| 轮 | 结果 |
|---|---|
| 探针复跑 | `adj(node===true,o=0,c=17,i=0)`、offset 1→0、`tcDesc=accessor`、无 THREW |
| 定向套件（Range-mutations 10 案） | TOTAL 2226P/8F/5T——dataChange **400P/0F**/1T（修复前 56P/369F/1T） |
| 全 dom 通道 A/B（384 案，HEAD 二进制 vs 修复二进制逐子测试对拍） | **0 回归**；316 改善全部落在 dataChange；head_only 670 / fixed_only 24 行全位于含 Timeout 案截断尾 |
| 全 html-syntax 通道 A/B（273 案） | 两二进制输出**字节级相同**（61,671P/697F/7T/1U）——修改对本通道零影响 |

- 修复后 dom 通道参照：[2026-11-05-m2-dom-after-datachange-fix.json.gz](2026-11-05-m2-dom-after-datachange-fix.json.gz)
- 定向套件 JSON：[2026-11-05-m2-mutations-targeted-after-fix.json.gz](2026-11-05-m2-mutations-targeted-after-fix.json.gz)

## 残余

- mutations 尾簇 8F（appendChild 4 + insertBefore 4）——M3 候选，与
  dataChange 无关（HEAD 二进制同值）。
- dataChange 每轮 1T：CASE_TIMEOUT=10s 截断（矩阵 ~441 行跑不完，修复后
  全绿行加速使截断点后移 372P→400P 轮间漂移）；harness 自带
  `<meta name=timeout content=long>`，提额属 runner 通道行为变更，不在
  本簇范围。

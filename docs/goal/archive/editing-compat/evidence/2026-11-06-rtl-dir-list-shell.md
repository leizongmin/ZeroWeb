# rtl compare 族 ×6 收口：freshShell 出口 dir 特判（rtl 刀）

日期：2026-11-06。基线 = br 刀收口面（iol 1750P/11F / iul 2050P/15F / mt 10,092 键 9,676P/416F / enginelib 3,773P/5F 五红名册）。

## 结果

iol 1750P/11F → **1753P/8F**：judge-dps **F2P 3 / P2F 0 / postF 0 / lost 0**，rows 1761 恒等，VERDICT OK；fail 行集 diff 恰除名 rtl ×3（`?1001-last` 三行：`<div dir="rtl">a[]bc</div>` 折叠选区 / `<div dir="rtl">a[bc` × `<div dir="rtl">de]f` 跨块同 rtl / × `<div dir="ltr">de]f` 跨块混合），余 8 行（nested ③ ×3 + blockquote ×2 + styled ×1 + noneditable ×1 + qVb 伪影 ×1）逐名恒一。

iul 2050P/15F → **2053P/12F**：**F2P 3 / P2F 0 / postF 0 / lost 0**，rows 2065 恒等，VERDICT OK；fail 行集 diff 同恰除名 rtl ×3（`?2001-last` 三行镜像），余 12 行（nested ×4 + blockquote ×2 + divbr straddle ×2 + div 边界 ×3 + noneditable ×1）逐名恒一。

mt 全文件腿：rows 10,092→10,092（P 9,676→9,676 / F 416→416），**F2P 0 / P2F 0 / post-only 0 / lost 0**，键集逐位恒等，VERDICT OK（br 后基线直比，无切片阴影）。

enginelib：**3,773P/5F 五红名册恒一**（ROSTER OK，与 dps/br 轮 canonical 名单逐字一致；本基线对齐轮间无新增测试）。

探针：zz-probe-rtl post-patch 6/6 got 与备选树 2 逐字合（差异仅选区标记位，compare 面语义剥除；`rtl-probe-round2.log`）；零扰动 zz-probe-br 6/6 got==exp（`br-probe-round4.log`）、zz-probe-qv13 26/26、zz-probe-dps 11/11（post-patch 复跑）。

## 归因

探针 zz-probe-rtl（dump 式 6 案 = iol/iul × 3 形，期望树逐字取语料 `data/insertorderedlist.js` :726/:732/:741 与 `data/insertunorderedlist.js` :818/:824/:833 双备选树）pre-patch got 全捕获（`rtl-probe-round1.log`）：

- 折叠形 `<div dir="rtl">a[]bc</div><div dir="rtl">def</div>` → got `<ol><li dir="rtl">a[]bc</li></ol><div dir="rtl">def</div>`（首 div 被列表壳替换 = 本实现提升形）
- 跨块同 rtl `a[bc / de]f` → got `<ol><li dir="rtl">a[bc</li><li dir="rtl">de]f</li></ol>`
- 跨块 rtl+ltr `a[bc / de]f` → got `<ol><li dir="rtl">a[bc</li><li dir="ltr">de]f</li></ol>`（iul 三案镜像，壳为 ul）

机制：跨块 straddle enable 走 p/div→li retag（setTagName 全属性携带），源块 dir 落 li。本实现产出提升形（div 被壳替换），与备选树 2（`<ol dir="rtl"><li>abc</li>…` 壳携 dir、li 恒裸）唯一差 = dir 位置。语料明注（iol :738-740 注释、iul :830-832 同文）："do not copy `dir` attributes to corresponding `<li>`s because different one from the value of the parent list element causes odd looks and anyway the `dir` attribute does not affect the text direction"。双备选树 dir 恒在列表壳（提升形）或留包裹 div（内嵌形），li 恒裸；全 editing/data 语料无任何期望树含 `<li dir=…>`（grep 证），strip 臂不可能破坏期望树。

## 修复（part01e 单笔 +34 行，唯 part01e 改动）

freshShell 出口 dir 特判，置于 br 刀 `_zwListFixDisallowedAncestors` 之后、dps 归并出口之前，同一 `freshShell && newShell && _zwListNodeInDocument(newShell)` 门：直子 li 携 dir 时，首个 dir 值迁壳（壳已有 dir 不覆盖；读/写异常保守跳过），随后全部直子 li 移除 dir。缩进合并臂（creator 归并分支，freshShell=false）不触发；id/class 行已绿不受影响——"other attributes should be cloned to each list item" 契约 id/class 留 li，唯 dir 特判。

## 门禁

- 探针：见结果节；执行体证据 `grep -a -c "726/:732/:741"` `target/release/zero-wpt-runner` = 1（批内 cargo build 0.70s no-op 复用 15:30 代二进制——探针 round2 行为证 + marker 串证双验，marker 门先核后跑）。
- 全腿批：`batch-rtl-legs.sh`（flock 单实例 + marker 门 + judge 三段 + 五红名册核对；systemd `zw-rtl-legs.service` 脱离，18:09:57 起，18:47:03 ALL-DONE，BATCH-VERDICT OK），基线 = br 后 `batch-{iol,iul,mt}-post.log` + enginelib 名册。
- 判读器：`judge-dps.py`（行级元组 F2P/P2F/postF/lost，dps 轮同款复用）。
- 日志：`.acceptance/dps/batch-rtl-legs.log`（批摘要）、`batch-rtl-{iol,iul,mt}-post.log`（全腿原文）、`batch-rtl-enginelib.log`（五红原文）、`rtl-probe-round{1,2}.log`（修复前/后 got；dump 式恒假断言 rc=1 属预期）。

## 衔接账

rtl 归因、修复与探针验证由孤儿会话（PID 3713306；驱动轮 14:35 退出后脱管，15:17 完成上一轮 br 刀证据修订提交）15:22-15:34 完成：probe round1（pre-patch got，15:24）→ patch 落树 → 二进制重建（15:30）→ probe round2（post-patch got，15:30）→ 零扰动三探针（15:32-15:34）。验证席 session 2b56d583 独立核对探针期望树与语料逐字一致（含 iul :818/:824 镜像）及 mt 补审声明无误，15:36 429 死亡，控制面在飞标记未落。续轮（18:04 限额重置后）接手：全腿门禁批 + 判读 + 落账收口。

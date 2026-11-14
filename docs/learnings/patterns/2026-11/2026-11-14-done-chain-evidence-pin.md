---
date: 2026-11-14
modules: engineering, workflow
---

# done 链 evidence pin：task→done 同 rev 必带 {path,sha256}，漏项重建走存证+零漂移断言

## 问题

workflow 账本把任务迁到 done 时，`task.evidence` 必须同 rev 绑定
`{path, sha256}`；漏绑后回补容易犯两类错：直接改历史 rev（账本不可变语义
被破坏），或凭记忆重建证据内容（sha 与当时产物不符，伪造证据）。

## 根因分析

done CP 的 `workflow_ref` 用弱引用 `{revision, snapshot}`——弱引用允许跨
rev 追溯，但 `task.evidence` 是强 pin：路径+内容哈希绑定具体文件。证据
文件在 done 之后可能被轮转清理，漏 pin 的任务在事后审计时既找不到文件、
也没有哈希可对，只能按当时运行产物重建。

## 解决方案

1. **pin 契约**：task→done 同 rev 必带 `task.evidence = {path, sha256}`；
   path 用运行目录相对路径，sha256 对当时文件实算。done CP 的
   `workflow_ref` 保持弱引用 `{revision, snapshot}`，不复制证据内容。
2. **漏项重建四步**（T64 实证形制）：
   a. 找到漏 pin 时刻的运行产物（快照/运行目录留存文件）；没有就如实标
      REBUILD，不许拿后补文件冒充当时产物；
   b. 先存证 `*-pre-evidence.json`（重建前账本状态快照，含被改 rev 与
      漏项字段）；
   c. 文本级重建 evidence 文件 → 实算 sha256 → 与运行目录留存件（若有）
      逐字节比对零漂移断言；
   d. 过 validator 全量校验后重放迁移；失败态快照存证修正重放，不覆盖
      失败历史。
3. **时间戳纪律**：重建件的时间戳写重建时刻并标注 rebuilt，不伪造为原始
   时刻；finished_at ≤ 快照 mtime、updated_at ≤ 墙钟不变量自查。

**Why**：evidence pin 是账本审计的锚。哈希绑定让「这条 done 当时看过什么」
可复核；存证+零漂移断言让修复动作本身也可审计，两条缺一不可。

**How to apply**：迁移到 done 前先检查 evidence 文件存在且 sha 实算入 pin；
发现历史漏项时按四步重建并全程留证；校验器报的字段缺失逐条对照本条目
补齐，不绕过校验器直接改 JSON。


### t9 双首轮与返修待办（2026-10-15 15:10，交接）

- 账本 b7 rev1 已装：t9 pr_review、双首轮齐（op-29 测试有效性 CHANGES_REQUIRED 2 findings / op-30 缺陷 PASS 零阻断）、门禁全绑（target+engineering PASS）、校验通过。
- F1（返修待做，medium）：补"长臂降级不阻塞"回归钉——把 6 点回退直呼后全量测试仍绿（无保护）。建议：worker 忙臂占住时断言有界调用在超时内返回 Err 而非阻塞（参考首轮报告 review/t9-first-test-validity.md 的建议与既有 pin 风格）。
- F2（已修）：完整 SHA 短前缀错误延展——manifest/gates/delivery 已更正为真实值（head 9d56a90672d5e41b13c86beb5cb24125d1d8ea66 / base e7e5b8c8d29b1a5f6fc4f00db6eae17007f4101e / subject d5870cd9…）。
- 返修序列：补忙臂钉（commit）→ 推分支 → 非作者复核（test_validity 角色，covers=[op-28-t9-implement 及返修 op]）→ 汇总（复用 op-30 缺陷角色 agent_4a2b65b3）→ 正文 render（截图 not_applicable 已在 body）→ ready → 合并意图 → merge → 集成 → t9 done。
- 分支 t8-input-bounded @ 9d56a9067 已推送；PR #180 draft。双首轮 agent：①=agent_758c6e8c（过期可重派）②=agent_4a2b65b3。
- 教训：完整 SHA 必须从 git rev-parse 实取，禁止短前缀人工延展。

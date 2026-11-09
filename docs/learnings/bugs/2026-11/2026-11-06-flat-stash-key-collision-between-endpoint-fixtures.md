---
date: 2026-11-06
modules: wpt-runner
---

# wpt-runner flat stash 单键与上游多键 dict 语义冲突——多端点共用 token 的 fixture 须键命名空间化

## 问题

net-api-compat-v3 M1b 把 `fetch/api/resources/redirect.py` fixture 从全局
原子计数升级为上游 per-token stash 语义后，`fetch/api/cors/cors-redirect.any.js`
20 个子测试 Pass→Fail（`No preflight request has been made expected "0" but
got "1"`）——改动的 fixture 与回归的用例不在同一目录，最初以为 redirect
批次语料导入引入，实际是**既有 fixture 的 stash 值格式冲突**。

## 根因

上游 wptserve 的 server stash 是 **per-token 多键 dict**：redirect.py 写
`{count, preflight}`，preflight.py 写/读 `{did-preflight, ...}`，同 token
下各端点字段互不可见。runner 的 `WPT_TOKEN_STASH` 是
`HashMap<String, String>`——**一个 token 只有一个字符串值**。redirect.py
fixture 写 `"count\x1fpreflight"`（如 `"1\x1f0"`）进裸 token 键后，同一
cors-redirect 链上的 preflight.py fixture 读同键、按自己的四字段定序格式
解析，把首字段 count `"1"` 误读为 did-preflight。

改前 redirect.py 从不写 stash（全局原子计数），冲突被掩盖；语义升级后
"写 stash" 成为新路径，冲突显影。A/B 探针收敛条件：**redirect.py 请求带
token 即污染**（同 token 直连/无 token 中转均干净）。

## 解决

redirect.py fixture 的 stash 键加端点命名空间：`redirect\x1f{token}`；
`clean-stash.py` fixture 同步清两键（上游 clean 按 token 抹全部端点状态，
`had` 取任一键存在）。preflight.py 继续用裸 token 键，互不可见，等价还原
上游多键 dict 语义。修复后 cors-redirect 20 对回到逐字节一致，全通道 A/B
零 P→F。

## 避免方式

runner fixture 引入**新的 stash 写入方**时：不要往裸 token 键写与既有
fixture（preflight.py 四字段、stash-count 等）不同构的值；用
`<端点>\x1f{token}` 命名空间键，并让 `clean-stash.py` 同步清理。反之，
既有 fixture 的解析器若依赖字段序，对"外来"值应按格式判别默认兜底而非
按位硬解。

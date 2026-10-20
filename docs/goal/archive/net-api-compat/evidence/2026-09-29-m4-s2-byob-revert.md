# M4-S2 — byob read(view) 甄别与回退（零净变化收口）

**日期**: 2026-09-29
**切片**: M4-S2（byob 真读取面首版尝试 → 回退甄别）
**原始数据**: streams 全量复核 = M4-S1 基线逐案一致（[streams](2026-09-29-m4-s1-streams-streams.json) 复用；
本轮回退后复核 271/598 = 45.3% 与 M4-S1 完全一致）

## 结果

**零净变化**：streams 271/598（45.3%）与 M4-S1 逐案一致（复核跑 exit=1 系 read-min
挂 settle 案的整跑墙钟超时，与 M4-S1 相同形态）；其余五 corpus 未触碰。

## 甄别过程（byob read(view) 首版）

1. **首版实现**：getReader({mode:'byob'}) 返回 byob reader——read(view) 填充 view、
   剩余字节回队（最小填充）、waiting 路径同步填充。queue 路径探针单读通过
   （guard=1 offset=25）。
2. **waiting 路径爆涨**：`_bodyToStream` 单 chunk 源 × waiting 残余 wait 的
   close 填充（`_RS_DONE` 对象经 String() 退化 '[object Object]' 填充）× view 形态
   组合 → **4.2GB 内存爆涨**（test-guard 拦截，proces 树单杀不连累）。
3. **修复尝试**（`chunk.done` → done 分支）后仍挂（exit=124）——waiting 路径的
   flushPull × pull 重入组合未甄别完整。
4. **回退**：read(view) 移除，恢复默认读取形态（closed-check 一并修复——首版回退
   splice 曾丢 `state === 'closed'` 检查，已补回并复核）。

## 回归基线核对

回退后 streams 271/598 与 M4-S1 **逐案一致**（复核跑无 REGRESS 输出）；fetch
response-consume-stream 14/15 ✓。

## 残余（M4-S3 记账）

- **BYOB reader 专设切片**：view 跟踪/最小填充契约/close-done 分支/waiting × pull
  重入语义——需独立设计 + 内存画像（首版 4.2GB 面的根因分解），不并入混合切片。
- values() 完整语义（async-iterator 41 腿 + 挂 settle 甄别）——同上待专设。
- 构造 dictionary 转换序（queuingStrategy 先于 underlyingSource）。

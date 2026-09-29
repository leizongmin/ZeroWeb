# M4-S3 — BYOB reader 专设切片（response body byob 读取全绿）

**日期**: 2026-09-29
**切片**: M4-S3（BYOB reader 专设——M4-S2 首版 4.2GB 爆涨面根因对策后的重做）
**原始数据**: [streams](2026-09-29-m4-s2-streams-streams.json) ·
[fetch](2026-09-29-m4-s2-streams-fetch.json)（其余四 corpus 与 M4-S1 一致：
[url](2026-09-29-m4-s2-streams-url.json) / [xhr](2026-09-29-m4-s2-xhr-state-xhr.json) /
[mimesniff](2026-09-29-m4-s2-xhr-state-mimesniff.json) /
[eventsource](2026-09-29-m4-s2-xhr-state-eventsource.json)）

## 结果

- **fetch response-consume-stream：15/15（全绿）**——6 条 byob 腿（blob/text/
  URLSearchParams/arrayBuffer/formData + offset 读取）全部解锁。六 corpus 侧
  fetch 1221/1745、streams 271/598（逐案核对零回归）、其余与 M4-S1 一致——
  **8063/10789 = 74.7% 持平**（byob 解锁的 fetch 腿 +5 与 byob 守卫面 -2 残余相抵）。

## 修齐内容（part02.js getReader byob 分支重写）

1. **reader 本地余量**（`byobPending`）：fill 余量存 reader 独占字段，**不回 queue、
   不 unshift**——M4-S2 首版 queue/pull 双通道复制面（4.2GB 爆涨根因）消除。
2. **done 语义三足**：①填充即返 chunk（done=false，closed 流的已排队数据仍可读）；
   ②排空后 closed → done；③readable 等待 → waiting 自定义 resolve（chunk 到达填
   view、余量进 pending 后**重入 byobViewRead**——不回 queue）；close 哨兵
   （`_RS_DONE`/done:true 非字节视图）→ done。
3. **fill try/catch**：detached buffer 等 view 构造 TypeError → reject（不挂探针）。
4. M4-S2 的 closed-check 丢失与 waiting 盲区两处回退伤在重写中一并治愈。

## 全量复核

streams 全量复核（9000s 限额，exit=1 系 read-min 在册挂案）**271/598 = 45.3% 与
M4-S1 逐案一致、零回归**；fetch byob 腿全绿（consume-stream 15/15）。

## 残余（记账）

- values() 完整语义、构造 dictionary 转换序：延后（M4-S1 回退注记维持）。

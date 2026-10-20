---
date: 2026-10-20
modules: engine
---

# WHATWG prescan 共享 position 游标：skip 落点必须回写再进属性循环

## 问题

`_zwFramePrescanCharset`（iframe 子文档编码嗅探）在 baseline.htm 首个 `<p>`
标签处死循环，单次 `execute_script` 超 90s V8 watchdog 报 `Execution
timeout`。同套件其他 fixture 正常，死循环点随输入内容漂移。

## 根因

spec 的 prescan 算法只有一个 position 指针：进入 tag 分支后先「sliding to
the first whitespace or `>`」定位属性区起点，再循环 get-an-attribute——两步
共享同一游标。首版实现用局部 `i2` 做 skip、闭包 `_paPos` 做属性扫描，两游标
失步：`_paPos` 残留上一 tag 的陈位时，getAttr 从陈位起扫可能把后续区间当属
性吃完并停在早于 `i2` 的落点；循环末尾 `i2 = _paPos` 回跳，外层 `for(;;)`
永无进展。

失步是否触发取决于字节内容（陈位与 `i2` 的相对落点），因此死循环点随输入
漂移、部分 fixture 仍通过。

## 解决方案

skip 落点回写共享游标后再进属性循环，且仅在窗界内进入：

```js
_paPos = i2;            // 游标同步：spec 只有一个 position 指针
if (i2 < end) {
  for (;;) { if (!getAttr()) break; }
  i2 = _paPos;          // 属性循环的最终落点即新游标
}
```

通用规则：**移植逐字节状态机类 spec 算法时，凡 spec 文本中同一「position」
被多个步骤读写，实现必须对应单一变量**；引入局部镜像游标前先证明两游标单
调同向，否则失步回跳即死循环。

## 排查路径备忘

死循环只有 90s 超时一个症状，定位靠最小 probe 矩阵二分（控制组 → 逐条件
增删），缩到「仅 prescan 禁用即完成」后代码级复推。注意：并发 rally 共享
`/tmp` 心跳文件会互染，进程级归因要用独占文件名。

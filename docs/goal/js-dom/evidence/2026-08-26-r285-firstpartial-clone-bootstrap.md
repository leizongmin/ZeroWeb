# R285 Evidence — firstPartial clone 引导（extract 100% / clone 185P）

**日期**: 2026-08-26
**切片**: M4——R285(a) comparator walk 域簇 → 实为 spec 缺口
**改动面**: `part06.js`（extractContents + cloneContents 的 firstPartial clone 引导）+ `part23.rs`（r283 f53 期望更新）
**commit**: `3c3fe2014`

## 一、归因链（「comparator 域」假设推翻——实为引擎 spec 缺口）

R284 把 53,x 残余归因为「comparator walk 域」——本轮 dual-walk probe
（A/E 双侧 nextNode 同步 dump）实证：

```
A: [frag, P#e(text), P(CDATA...), comment-head]
E: [frag, P(empty), P(text), P(CDATA...), comment-head]
```

**E（oracle）是正确的 spec 输出**：sc=P#d 是 cac(DIV) 的直接子且
partially-contained → **frag 以 P#d 的空 clone 开头**（spec
`dom-range-extract-contents` 的 first partially contained child 分支：
clone + 子区间提取；so=1 越过 P#d 唯一 text 子 → 空壳）。引擎 A 侧缺
引导使后续节点全部错位一位——walk 里读成「expected Element got Text」。

R284 的「引擎 frag 结构正确」结论没错（那个 probe 是 DOM 子测试轮的
提取）；frag 子测试是**独立一轮 restoreIframe + 提取**——两轮形态不同。
教训：**同一 subtest 的 DOM/frag 断言是两次独立执行**，probe 须注入在
目标断言的同一轮。

## 二、实现（双侧引导 + 首版回归教训）

- **extract**：sc !== cac 且 sc.parentNode === cac 时，frag 以 sc 的
  shallow clone 开头（内承载 [so, scEcPath) 子 move）。
- **clone**：同形态的 deep-clone 版。
- **首版教训**：全形态引导（cac===sc 也 bootstrapped）使 48,x -3 回归
  ——sc 是 cac 时 sc 是**容器自身**非 firstPartial，frag 无 wrapper。
  引导限定 `sc.parentNode === cac`。

## 三、验证（A/B vs R284 基线，全 ranges sweep）

| 项 | R284 | R285 | Δ |
|---|---|---|---|
| Range-extractContents | 186P/1F | **187P/0F（100%）** | +1（53,x 全解） |
| Range-cloneContents | 184P/3F | **185P/2F** | +1（53,x 解） |
| Range-deleteContents / insertNode / surround | 125P / 1840P / 1840P 全 0F | 同 | 持平（100%） |
| ranges 全量 | 37853P | **37855P** | +2，set-diff 0 新 fail |
| engine 单测 | 2418 | 2418（f53 期望更新） | 持平全绿 |
| fmt / clippy | 干净 | 干净 | — |

**ranges 域现状**：deleteContents 100% + extractContents 100% +
insertNode 100% + surroundContents 100%；cloneContents 185P/2F
（29/31,x docEl clone 域——handle vs plain 克隆形态，低 ROI 小簇）。

## 四、教训

- **「expected X got Y」的 walk 错位先查缺失的引导节点**：一侧多一个
  首节点（空壳 clone）会使后续全部错位——dual-walk probe 直接暴露。
- **DOM/frag 两断言是两轮独立提取**：probe 必须注入目标断言同一轮
  （本轮第一 probe 在 DOM 轮得出「frag 正确」的错误结论）。

## 五、R286 靶点

- **(a) clone 29/31,x**（docEl clone 的 handle vs plain 域——小簇）。
- **(b) Range.detach() 预存 1F**。
- **(c) deleteContents ShadowRoot 一例**（`{<span>ABC</span>}` 形态）。
- **(d) mutations 超时族**（环境慢，低 ROI 备档）。

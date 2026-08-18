---
date: 2026-08-19
modules: crates/engine/src/js_dom_shim/part02.js
---

# IndexedDB key range canonical conversion

## 问题描述

IDBKeyRange 直接保存调用方传入的 key，会保留 TypedArray/DataView 类型和可变 buffer
引用；简化的“是否合法”探测还可能吞掉 array getter 抛出的异常，或在首个参数无效后
继续读取第二个参数。

## 根因分析

Key validation、canonical copy 和 comparison 被拆成多条近似路径。仅用比较结构验证 key
无法生成规范要求的独立 key 值，也无法统一 BufferSource view 范围、Date clone、递归
array conversion 和异常传播顺序。

## 解决方案

建立单一 canonical conversion：先转为 typed key wire，再从 wire 重建 JS key。该路径
自然复制 Date、ArrayBuffer、TypedArray/DataView view 和递归 array，并保持 getter 异常
原样传播。KeyRange 构造器、includes 和需要同步校验的 API 复用此边界；多参数 API 在
每个参数转换后立即判错，禁止无效首参继续触碰后续参数。

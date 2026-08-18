# IndexedDB metadata tasks and UTF-16 wire names

**日期**: 2026-08-18

**相关模块**: `engine`、`page-runtime`、`storage`

## 问题描述

Metadata WPT 同时暴露两类跨边界问题：upgrade success 在 `upgradeneeded` 的同一 microtask checkpoint 内触发，以及包含 lone surrogate 的 object-store 名称无法解析为 Rust `String`。

## 根因分析

IndexedDB success/error 是独立 task；用 microtask 完成 upgrade 会抢在 promise test 完成前派发 success。另一方面，DOMString 是 UTF-16 code-unit 序列，Rust `String` 只接受有效 UTF-8，直接通过 JSON 传输会在 serde 边界失败。

## 解决方案

Upgrade completion 进入下一 timer task，保留当前 task 的 promise checkpoint。数据库、store 和 index 名称在 JS host 边界按需编码为可逆 UTF-16 hex，并只转换协议名称字段，不递归用户存储值。关闭连接时冻结 schema 名称快照，避免后续连接升级反向修改旧连接视图。

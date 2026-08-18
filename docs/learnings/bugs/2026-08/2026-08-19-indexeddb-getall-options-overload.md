---
date: 2026-08-19
modules: engine, wpt-runner
---
# IndexedDB get-all options overload 不能复用旧 query/count 路径

## 问题描述

IDBObjectStore/IDBIndex 的 getAll 与 getAllKeys 同时接受旧的 query/count 参数和
IDBGetAllOptions dictionary。直接把首个对象参数当 key 会让 options 全部抛 DataError；
直接用 Number(count) 截断又会把 NaN、Infinity 和负数静默接受。

## 根因分析

WebIDL 转换发生在算法状态检查前，count 是 [EnforceRange] unsigned long；options
dictionary 还按 count、direction、query 的字典成员顺序读取。dictionary 的 count 0 表示
不限制数量，而 legacy 第二参数的 0 仍是显式数量。index 的 unique/reverse/count 也必须在
同一有序 entries 视图上依次应用，不能让 host 提前截断。

## 解决方案

先按 IDB key 类型与普通 dictionary 对象区分 overload，再用共享 helper 完成 count、
direction、query 转换。host 返回完整有序 entries 后，在 JS 侧统一执行 unique、reverse、
count；getAll/getAllKeys/getAllRecords 只在最终结果 shape 上分叉。回归覆盖 count 0、
第二参数忽略、prevunique 和 IDBRecord 只读反射。

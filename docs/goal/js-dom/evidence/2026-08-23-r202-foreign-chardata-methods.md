# R202 Evidence — foreign/detached doc 文本节点的 CharacterData 方法面（M4）

**日期**: 2026-08-23
**切片**: M4 轻量——R201 解锁的 24kF 中 foreign-doc CharacterData 簇的能力面修复（方法从无到有）；subtest 计数持平（30144P/24395F/20T 双路径，fail 集逐字节相同、零新增文件）
**改动面**: `part03.js`（`_zwAttachCharacterDataMethods` 新 helper + 三接线点）+ `part21.rs`（单测）

## 一、能力面

`_zwMText`/`_zwMComment` 产物（foreignDoc/detachedDoc 的 createTextNode/createComment、
`new Text()`/`new Comment()` 构造器、innerHTML 解析树子）+ detached doc 工厂的内联
createComment/createCDATASection 对象——补
appendData/insertData/deleteData/replaceData/substringData 五方法 + data/nodeValue
可写 accessor。**本地变更语义**（R48 no-parentSel 快照分支同款——foreign 树 JS 侧
持有，无 host SetChildText）；null→''（LegacyNullToEmptyString）。

探针全绿：foreignDoc.createTextNode 上五方法 + setter 链（insertData→"fooxyz" /
appendData+len / deleteData / replaceData+substringData / data= 联动 nodeValue）+
comment 方法面。

## 二、过程两坑（单测/探针当场抓回）

1. **setter 自递归**：首版 `_write` 赋 `n.data`/`n.nodeValue`——两字段刚转为
   accessor（setter 调 `_write`）→ 无限递归 Maximum call stack。修：`_write` 只写
   `__nv`/textContent（accessor getter 读 `__nv`，无需写字段）。
2. **detached doc 的 createComment 是独立内联对象字面量**（非 `_zwMComment` 工厂）
   ——单测 `cm.appendData is not a function` 抓回，同款 attach 补上
   （+ createCDATASection 同块）。

## 三、subtest 计数持平的语义（诚实记录）

解锁的 foreign 子测试从 "insertData is not a function" 立即死亡 → 现在跑到**下一
断言**（identity/offset 形态）再失败——计数不变、失败位置前移。mega 簇的主阻塞
仍是 identity 域（R202 下一步 (a)）。本切片是它的前置能力面。

## 四、验证

- 全量 polyfill **30144P/24395F/20T** / native **30144P/24395F**（fail 集与 polyfill
  **逐行相同**）；vs R201 基线 fail 文件集 **零新增零修复**（计数持平）
- zero-engine 2342 单测全绿（含新
  `test_foreign_doc_text_characterdata_methods_r202`）；fmt/clippy 干净
- `make test` 全绿除 `window_surface_present_smoke`（XOpenDisplayFailed 环境，
  clean HEAD 复现，run-rules §10）

## 五、commit

`d7b6c4967`

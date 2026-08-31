# Evidence: R1 — WPT dom/nodes 上游通过率基线（M4 / DC-3 首切片）

**日期**: 2026-08-13
**轮次**: R1
**Commit**: 本切片 land commit（见归档）
**分类**: `dom/nodes`（首批导入，上游 WPT `31597693`）

## 测试命令

```bash
make testharness-dom                    # 全量（text 输出，exit 1 = 有失败）
make testharness-dom FILTER=Document-   # 按路径子串过滤
cargo run --release --bin zero-wpt-runner -- testharness-dom --format json   # JSON 输出
```

## 通过率基线（首跑数字即基线，后续持续提升）

| 指标 | 值 |
|------|-----|
| 用例文件数 | 141 |
| subtest 总数 | 2696 |
| Pass | 1112（41.25%） |
| Fail | 1572 |
| Timeout | 12 |
| Unsupported | 0 |
| 全部 subtest Pass 的用例 | 14 / 141 |

## 失败聚类（top 缺口类型，按错误消息归并）

| 失败次数 | 类型 | 根因方向 |
|----------|------|----------|
| 414 | `assert_throws_dom`（非法操作未抛 DOMException） | createElement 非法标签 / appendChild 闭环 / Document fragment 边界——**最大缺口**，DOMException 抛出语义未实现（M1/M2 候选） |
| 98 | `Cannot read 'documentElement' of undefined` | XML/XHTML document 模型缺失（`document.implementation.createDocument` 返的 doc 无 documentElement） |
| 80 | `Cannot read 'name' of null` | Attr 节点 / namedNodeMap 在 XML 上下文缺值 |
| 60 | `invalid string (token must not be...)` | DOMTokenList/classList token 校验（空/重复/非法字符）应抛 InvalidCharacterError |
| 49 | `instanceof HTMLElement` expected true | native 对象原型链（customElements/HTMLElement 构造器链） |
| 44 | `document.createProcessingInstruction is not a function` | ProcessingInstruction API 未实现 |
| 39 | `instanceof Element` expected true | native Element 原型链（部分路径返 generic object 非 Element 实例） |
| ~100 | `wrong class after modification`（多条） | classList 修改后 class 属性序列化顺序/去重 |
| 12 | `page script threw`（Runtime error） | 少量用例脚本执行异常（OOM/超时类） |

## 全 Pass 用例（14 个，native + polyfill 在此面已达 spec 一致）

`dom/nodes/` 下 14 个用例全部 subtest 通过——这些是已达成 spec 一致的面，后续迁移不得打破。
（完整清单见 commit 同附的 JSON 报告；样本：CharacterData-appendData / Comment-constructor / Document-URL / DocumentFragment-constructor 等基础构造与读语义。）

## 结论与下一步

- **M4 基线建立**：dom/nodes 通过率 **41.25%** 作为首个锚点。DC-3「上游用例导入 + 通过率基线」首切片达成（dom/nodes 子集）。
- **基线价值**：立即暴露 native/polyfill 真实规范差距——`assert_throws_dom`（414 次）是单一最大缺口，DOMException 抛出语义是最高 ROI 修复方向。
- **下一步 M4 切片候选**（按 ROI）：
  1. **DOMException 抛出语义**（createElement 非法标签 + classList token 校验）——单类 ~474 失败，最高 ROI
  2. `document.createProcessingInstruction` API 实现（44 失败）
  3. XML/XHTML document 模型（98 失败，但跨面较大）
  4. 扩展 `DOM_TEST_SUBDIRS`：导入 `dom/events` / `dom/collections` 扩面

## 注意

- 用例 gitignored（`fetch-dom-subset.sh` 按需拉取，不入库）——基线复现须先 `make fetch-wpt-dom`。
- 当前生产走 polyfill 桥（kill-switch 关），本基线反映 polyfill 路径真实通过率；native 路径（`ZW_NATIVE_DOM=1`）通过率对照待 M1 L2/default-on 后单独建。

# M4 R37 — 导入 dom/collections 子目录建立通过率基线

**日期**: 2026-08-14
**轮次**: R37
**里程碑**: M4（WPT dom 上游基线 + 按聚类驱动修复 / DC-3 扩展）
**前置**: R36（dom_bindings coverage 提升）
**状态**: ✅ 已 land（纯资产导入，零源码逻辑改动，双路径对等）

---

## 背景

R36 后剩余 ROI 重排，`扩 DOM_TEST_SUBDIRS`（dom/collections 等）是零源码改动纯资产切片（同 R21 dom/events 模式）。探测候选子目录用例规模：collections 10 / traversal 17 / lists 5 / ranges 44。选 **dom/collections**（10 用例，纯 DOM API：HTMLCollection/NodeList/NamedAttributeMap/DOMStringMap，不依赖 document/window listener 深结构，根因清楚可按聚类驱动修复）。

## 实现

### 配置扩展（2 处同步）

- `tests/wpt-runner/src/testharness.rs:62` `DOM_TEST_SUBDIRS` 加 `"dom/collections"`
- `tests/wpt-runner/scripts/fetch-dom-subset.sh` `SUBDIRS` 加 `"dom/collections"`

### 用例 fetch

10 个 .html（jsdelivr CDN `@master`，0 失败）：HTMLCollection-as-prototype/delete/empty-name/iterator/own-props/supported-property-indices/supported-property-names + childnodes-messagechannel-crash + domstringmap-supported-property-names + namednodemap-supported-property-names。wpt-data gitignored（不入库，按需 fetch；R32 幂等快路径 cached 后秒过）。

注：首次 fetch 时 GitHub Contents API 偶发慢（`make testharness-dom`/直接脚本超时），改用 jsdelivr CDN 逐文件 fetch 成功；cached 后幂等快路径 0.004s 秒过（3 dirs 全跳过 API 列目录）。

## 基线（双路径对等）

| 路径 | pass | fail | timeout | 通过率 |
|------|------|------|---------|--------|
| polyfill | 11 | 37 | 1 | 22.9%（49 subtest，10 cases） |
| native | 11 | 37 | 1 | 22.9%（与 polyfill **完全对等**，差 0pp） |

双路径对等差 0pp——用例侧 collection 经 polyfill（document.getElementsByTagName 走 polyfill shim，未解问题 #9），native 路径对 collection 无影响，双路径同源。

## 失败聚类（按 ROI）

1. **HTMLCollection own 属性枚举**（~19 fail，主力）：`Object.getOwnPropertyNames(htmlCollection)` 应返 indexed（"0".."N"）+ named（id/name 属性值，HTML 命名空间元素，去重）+ expando。当前 polyfill proxy 返 `["length","item","namedItem"]`（仅原型方法，无 indexed/named）。影响 HTMLCollection-own-props/supported-property-names/supported-property-indices。**根因**：`_makeProxy` 缺 getOwnPropertyNames/ownKeys trap（HTMLCollection 经 polyfill proxy，未暴露 indexed/named properties 作为 own 属性）。
2. **HTMLCollection supported-property-indices/names set/delete 语义**（~5 fail）：indexed/named property 的 set（expando 不覆盖 indexed）/ delete（indexed 不可删）边界。
3. **NamedAttributeMap own 属性枚举**（3 fail）：`attributes` 的 getOwnPropertyNames 应返 indexed（"0".."N"）+ named（属性名）。
4. **HTMLCollection-empty-name/delete/iterator** 边缘（~9 fail）：空 id/name 处理、delete 语义、iterator 顺序。
5. **childnodes-messagechannel-crash**（1 timeout）：MessageChannel + childNodes 组合（postMessage 基础设施，非 collections 核心）。

下一切片（R38）候选：**HTMLCollection own 属性枚举**（聚类 1，~19 fail 主力，根因清楚——polyfill `_makeProxy` 加 getOwnPropertyNames trap 暴露 indexed + named properties）。

## 验证

| 门禁 | 命令 | 结果 |
|------|------|------|
| clippy v8 | `cargo clippy -p zero-wpt-runner --features v8 --all-targets -- -D warnings` | ✅ 零警告 |
| clippy quickjs | `cargo clippy -p zero-wpt-runner --no-default-features --features quickjs --all-targets -- -D warnings` | ✅ 零警告 |
| fmt | `cargo fmt --all -- --check` | ✅ 无 diff |
| WPT polyfill dom/collections | `make testharness-dom FILTER=dom/collections` | 11P/37F/1timeout（22.9% 基线） |
| WPT native dom/collections | `make testharness-dom-native FILTER=dom/collections` | 11P/37F/1timeout（双路径对等差 0pp） |
| fetch 幂等快路径 | `time bash fetch-dom-subset.sh` | 0.004s（3 dirs cached 全跳过 API） |

## 决策记录

- **为何选 dom/collections 而非 dom/ranges（44 用例）**：collections 10 用例规模适中、纯 DOM API（HTMLCollection/NodeList/NamedAttributeMap）、不依赖深结构，基线建立 + 聚类分析快速；ranges 44 用例规模大且 Range API 涉及更多边界。下一切片按聚类修复 collections 后可再扩 traversal（17）/ranges（44）。
- **R37 纯资产 net=0**：导入本身零源码逻辑改动，不提升现有基线，只建立新子目录基线 + 暴露真实缺口（HTMLCollection own 属性枚举主力）。后续 R38 按聚类修复提升。

## 净影响

- DC-3（WPT dom 基线扩展）：新增 dom/collections 子目录基线（polyfill/native 双路径 22.9% 对等差 0pp），WPT dom 覆盖面从 2 子目录扩到 3
- 暴露 HTMLCollection own 属性枚举缺口（~19 fail 主力），为 R38 按聚类驱动修复提供根因清楚的新工作面

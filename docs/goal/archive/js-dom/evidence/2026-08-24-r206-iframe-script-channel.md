# R206 Evidence — iframe 子文档脚本执行通道（M4）

**日期**: 2026-08-24
**切片**: M4 能力面——src iframe 的子文档 `<script>` 执行通道（旧只建 doc/win 不跑脚本）；套件计数持平 **49534P/5005F**（同文件内失败形态重分布、零新增失败行 vs R205 逐行 diff）
**改动面**: `part01.js`（`_zwRunIframeScripts` + 加载器接线）

## 一、通道设计

| 件 | 内容 |
|----|------|
| **脚本提取** | `<script src>` 外链经 `__zw_fetch_script(pageUrl, src)` 取源（相对解析）；inline 体直取 |
| **合并作用域** | 全部脚本段拼接进**单个** Function——共享变量环境（common.js 的 var 与后续 inline 的 `eval('paras[0]')` 同链）；形参 window/self/parent/top/document/location 绑 iframe win/doc |
| **per-part try/catch** | 真浏览器 per-`<script>` 错误隔离——一段抛错不杀后续段；首个错误落 `win.unexpectedException`（harness 期望的形态） |
| **顶层声明导出** | 行首锚定扫描（function/var/let/const——script_gen R147/R201 启发式家族）+ 每名 try 导出后缀：顶层声明成为 win 属性（真 iframe window 语义）——`contentWindow.setupRangeTests/.run/.testRange` 父侧可达 |
| **body onload** | 脚本后执行 `<body onload=NAME>` 处理器（若已定义） |

## 二、验证链（探针逐步实证）

1. 单元探针（stub `__zw_fetch`/`__zw_fetch_script` 读 wpt-data）：首版 `srt:undefined`
   → wire 分隔符/布局两坑修正（Rust `\x1f` 字面量非转义 + `__zwfr:` 后无空字段）
   → `st:done` 但无导出（合并 Function 顶层 throw 截断后缀）
   → per-part try/catch 后 **`srt:function` + `run:function`** ✓
2. Runner：`is not a function` ×920 簇**消除**——失败下移一层（restoreIframe 的
   contentDocument 失同步 `Cannot read null removeChild` ×919 + `testRange undefined`
   ×920 + `paras[5].append` 缺口 ×1——detached doc 元素缺 append/cloneNode 深层）

## 三、诚实记录

- 套件计数**持平**（49534P/5005F）——失败形态重分布（同文件内），逐行 diff 零新增
- 本切片是**能力面**：子文档脚本从「不执行」到「执行 + 可达」；暴露的下一层
  （restoreIframe 语义、detached doc append/cloneNode 面）是 R207 输入
- zero-engine 2345 单测全绿；fmt/clippy 干净

## 四、commit

`7f36dbbe0`

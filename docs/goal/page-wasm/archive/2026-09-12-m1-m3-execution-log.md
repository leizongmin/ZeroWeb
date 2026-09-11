# page-wasm 执行日志归档 — M1~M3 切片记录（2026-09-12 收官）

> 归档区：只追加不修改（goal 文档 Document Control）。本文件收拢 M1~M3 各切片的
> 执行记录与关键决策，控制面（master.md）只保留当前状态与尚未关闭的事项。

## 立项（2026-09-07）

从父目标 zero-web M12 拆出。立项基线假设：页面 WASM = dom_bridge polyfill stub 桥，
导出面/类型为瓶颈。（该假设后被 M1 基线实测修正——见下。）

## M1 — WPT 基线建立 + 类型扩展 ✅

- **切片 1**（commit 37da92898）：`fetch-wasm-subset.sh`（31 案标准 JS API 面，
  pinned 31597693，jsdelivr 首选 raw 回落）+ `testharness-wasm` runner 子命令 +
  Makefile 目标。基线 **715/719 = 99.4%**。
  - **架构级发现 1**：页面路径（run_page_scripts/js_dom_shim）跑 **V8 原生
    WebAssembly**——polyfill 只装 execute_script_with_dom 面。立项基线假设修正：
    后续切片针对 embedder 面（DC-2/DC-3 按其字母语义对 polyfill 路径执行）。
  - **架构级发现 2**：4 案 `constructor/` 异步面 Timeout 根因 = **V8 前台消息循环
    全仓从不泵**（v8_runtime 只泵 microtask，无 PumpMessageLoop）→ 修复落
    script-sandbox（event-loop-spec 流域）→ 按 run-rules §9 记跨流卡点不硬解。
- **切片 2**（d9565dddd）：`ExportDescriptor` 签名反射（三后端）+ 桥协议类型化——
  i64 BigInt 双向（>2^53 精度）、f32/f64、多返回值 Array、零返回 undefined。
- **切片 3**（9cdae5f69/b4474cfcb）：`WebAssembly.Module.exports()` 静态 + `{name,kind}`
  描述数组，compile/instantiate 双路径注入；`WasmExternKind` 分类。

## M2 — Memory/Global/Table + 实例化语义 ✅

- **切片 1**（731de158c）：Memory.grow 真实接线——三后端 `grow_memory`（spec 增长前
  页数语义）+ 桥队列 grow 线调用 + JS buffer 按新字节数重建。旧假实现（返新总数）
  本就不符 spec，R3352 测试同步修正。
- **切片 2**（a8912b310）：Global 导出接 JS 面（值对象 `.value`/`valueOf`，i64 →
  BigInt）+ Table `{length}` 快照（三后端 `table_size`）。途中抓到并修复注入脚本
  缺逗号 SyntaxError（export_fns 与 global_table_fns 拼接）。
- **切片 3**（422a6645d）：错误分类面——spec 错误类构造器
  （CompileError/LinkError/RuntimeError，Error 子类 + instanceof 判别）+ host 分类
  注入（compile → CompileError、链接失败 → LinkError 实例）。
  - **根因修复**：WA 桥状态此前每次 `execute_script_with_dom` 被全新对象覆写
    （_nextId/_callResults/_moduleExports 跨 execute 重置 → host 按 id 寻址注入
    错位，实测第二个 instantiate 的错误覆写第一个的）。改幂等安装（`_modules`
    标记守卫，与 document 守卫同型）。

## M3 — host function + 流式 + 收尾 ✅（切片 1/2）+ 收尾核账（切片 3）

- **设计**（fa3fc7bc3）：`docs/specs/page-wasm-host-function-spec-rfc.md`（lei-spec-rfc
  标准模式，Lint 23P/2W/0F）——方案 A 同步重入（host 排空点嵌套 sandbox.execute）；
  方案 B 队列挂起因 wasmi 无栈暂停被否；假设 1 证伪时降级方案 C。
- **切片 1a**（同 commit）：`ImportDescriptor` + 三后端 `import_signatures()` +
  polyfill `_importFns`/`_invokeImport`/载荷 `imports` 字段（零行为变化）。
- **切片 1b/1c**（37efb24b0）：instantiate 桥接线——LinkerConfig 组装（签名权威在
  模块）+ HostFn 线程局部作用域指针重入（Send+Sync 约束下 Box<dyn Sandbox> 不可
  捕获；SAFETY 注释覆盖 set→call→clear 作用域不变式）+ 严格编组（类型不符 → trap）。
  - e2e 挖出并修复两处协议 bug：嵌套脚本对 `_invokeImport` 返回值再包
    `JSON.stringify`（双重编码 → `["v"]` 取 Null）；HostFn 结果索引赋值 vs
    空 Vec push 约定（被静默吞掉 → wasm 读 0）。
- **切片 2**（4c1750110）：`WasmSandbox::validate()` 三后端（全量校验）+
  `compileStreaming`（新面）+ `instantiateStreaming` Content-Type 校验（spec
  TypeError）；JS validate 魔术字节快速检查按 DC-3「或明确记录」条款记账。
- **切片 3 收尾核账**（本归档同日）：DC-1~4 全满足，基线复跑零回归（99.4%），
  workspace build/clippy 全过 → **DONE 判定**。核账依据：
  [evidence/2026-09-12-m3-dc-closeout-audit.md](../evidence/2026-09-12-m3-dc-closeout-audit.md)。

## 移交后续的事项

1. **跨流卡点（P2）**：V8 前台消息循环泵——修复落 script-sandbox（event-loop-spec
   流域）或经用户授权跨域。落地后 `make testharness-wasm` 复跑预期 99.4% → 100%
   （4 案 constructor/ 异步面 Timeout 翻绿）。
2. **协议性已知限制**（master.md「已知限制」节）：Global 快照值 / Table 仅 length /
   `__wasm_errors__` 查询面 / import 回调 NaN → trap / JS validate 魔术字节快速检查 /
   streaming 聚合缓冲。
3. **TBD**（spec-rfc §10）：QuickJS 后端 importObject 重入可行性（TBD-1）；
   iterator-protocol 多返回值 import（TBD-2）。

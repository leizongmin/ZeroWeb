# M3 切片一：py 路由底座模块迁出（testharness_py_routes）

日期：2026-11-08。蓝图 = py 普查档 §三；基线锚 = 同日归因档
（[m3-baseline-reattribution](2026-11-08-m3-baseline-reattribution.md)，
日志 `/tmp/m3-baseline-164121.log`）。本切片为**纯迁出**（行为逐字节保持），
注册表化自切片二起按族分批。

## 一、交付面

- 新模块 `tests/wpt-runner/src/testharness_py_routes.rs`：fetch handler
  闭包体整体迁出（84 条臂 + 路径归一前置 + 静态回退尾，先命中先返回
  臂序不变）；案级 12 项 stash/状态收敛 `PyEndpointCtx`；分派入口
  `dispatch(ctx, req)`。testharness.rs 原位只剩 ctx 构建 + 调用
  （19,236 → 15,630 行）。
- testharness.rs 对被迁出体引用的 12 个 helper fn / 3 个 type alias /
  1 个 static / 4 个 PNG const / 5 个 ServeCustomResponseState 字段
  升 `pub(crate)`（可见性对齐，无行为面）。
- `wpt_cache_py_wrapped_handler` 外层包裹器与 `case_cache` 共享 Arc
  关系不变；ctx 内 `case_cache` 为切片对齐保留（dispatch 内 cache.py
  臂直调 fixture，缓存语义由外层包裹器承载，`#[allow(dead_code)]`
  注记）。

## 二、等价性证据

- **move 完整性**：迁出体与 HEAD 原体逐行比对（dedent 8 空格归一），
  差异仅 cargo fmt 换行重排（33 diff 行——长 if 条件并行 / GIF 字节
  数组换行点移动），空格归一后零内容差。
- **全量 A/B**：`/tmp/m3s1-ab-173322.log`（17:33–17:41，systemd 单元
  m3s1-ab-173322，同 test-guard 门 `--per-proc-mem 4 --total-mem 8
  --time-limit 900`，run 内 cargo Finished 19.88s）对照基线
  `/tmp/m3-baseline-164121.log`：头条全等 **13,582P/1,337F/8T**
  （14,927 subtest）；逐 subtest multiset（status+案+名，14,915 键含
  重复计数）对称差**空集**；七域分布逐项全等（fetch 2,533 /
  xhr 1,248 / websockets 741 / streams 1,019 / url 7,412 / mimesniff
  1,898 / eventsource 76——与基线归因档 §一逐项同）。
- 方法注记：两日志均含 UTF-8 高位字节（subtest 名带 emoji/非 Latin
  字符），locale 未定时 GNU grep 判 binary 静默吞 stdout——对照脚本须
  `LC_ALL=C grep -a`；首轮对照曾伪报 27 行「只增不减」（与
  `grep -c` 头条全等矛盾），以文本模式 + 计数保留 multiset 为准。

## 三、切片序列（后续批次）

- 切片二起：`PyRoute` 保序注册表（`when` 谓词 + `handle`，fallback
  None 语义），按族把 if 链逐段转注册项（迁移边界前缀推进，臂序恒
  保持），每批全量 A/B；四 fetcher 闭包臂（script/worker/service-worker，
  testharness.rs 5943-7354 段）随后同框架分表迁移。
- S12 cors-preflight-redirect 内联补丁收编 + update 脚本缺口清偿
  （语料 pin 形态核对 → 销账或补条目）随注册表化批次落。

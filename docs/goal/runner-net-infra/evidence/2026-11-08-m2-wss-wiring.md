# M2：scheme/端点寻址接线（DC-2 收口）

日期：2026-11-08。实施基准 = M0 wss 审计档 D3-D5；码面提交 853e3c572
（D3 token 映射 + D4 变体枚举 + D5 页锚 + M1 clippy 遗留清偿），通道 A 侧
采集经 systemd service 链跨 session 完成终验。

## 一、交付面

| 项 | 落点 | 内容 |
|---|---|---|
| D3 token 映射 | testharness.rs `apply_wpt_substitutions` | `{{ports[wss][0]}}` → `fixture_wss_port()`（TLS 端点）；h2 维持明文映射（无 HTTP/2 WS 栈，token 只落排除分支） |
| D4 html 枚举 | 同上 共享 runner `run_any_js_corpus_subdirs_with_helpers_variants` | net-api 通道对 websockets 子树开 wss 变体枚举（`CorpusRunOptions.ws_wss_variants`，其余通道分母不动）；**追加式**——基础 URL 单跑保留为 default 账，仅增 wss 变体键；含 wss 变体过滤天然排除 `?default`/`?wpt_flags=h2` |
| D4 any.js 解析臂 | 同上 新增 `case_meta_variants` | `// META: variant=` 注释解析（与 `wpt_meta_scripts` 同款精确前缀），单测钉三形 + 过滤面 |
| D4 变体页 substitution | html 变体臂 | 与基础页同 `apply_wpt_substitutions` 面，query 是唯一差异（any.js 走 `any_js_window_wrapper` 既有 substitution 面） |
| D5 页锚 | `net_api_case_uses_http_page` | `wpt_flags=https` 查串 → 页锚 `https://wpt.test`（secure-context 面，cookies/001-007 上游语义）；其余 net-api 维持 http://wpt.test:8000 |

## 二、实施发现（记档）

1. **枚举池实跑实数 = 101 案，非 M0 普查的 102**：离线普查（41 html + 61
   any.js）未过 skip 面。实跑 = 普查 − cookies/003/004/005（源含 `<iframe`，
   既有 iframe skip 规则命中——43 号记桩类「语料 helper 缺口」，枚举继承
   skip）+ constructor/016 与 interfaces/CloseEvent/clean-close（普查 grep
   形态欠采样，实跑枚举为准）。**实跑枚举是池真值**，门裁决按 101。
2. **共享 runner 8 参收编 `CorpusRunOptions`**（clippy too_many_arguments
   清偿）：expand_variants / extra_forms / ws_wss_variants 三开关结构化，
   五调用点显式声明。
3. **M1 遗留 clippy 三项清偿**：`FixtureStream::Tls` 装箱（变体尺寸差）、
   `let_unit_value`（send_close_notify）、zero-net 孤立 `NoDelay` import
   （`TcpStream` 固有 `set_nodelay` 优先于 trait 方法，dev profile 孤立）。
4. **A/B 工具面教训**：比对脚手架 awk 两处 bug（keys 文件误套状态正则 →
   shared 集空 → 空文件 diff 假阳性「全等」；修时又失 `-v rx` → 空正则全
   匹配混入日志尾标记行）。终裁改 python 多重集口径（逐案 (状态, 子测试名)
   排序比对），不受行序与日志尾噪声影响。

## 三、验证（门裁决）

1. **单测**：ws_fixture e2e 2/2 + `case_meta_variants` 1/1 绿；clippy
   `-p zero-net -p zero-wpt-runner` 零告警；rustfmt 过。
2. **通道 A/B**（`make testharness-net-api FILTER=websockets/`，make 内置
   test-guard 4G/8G/2400s，A 侧经 systemd service 跨 session 完成）：
   - B 侧 = M1 基线（干净 HEAD 70960967a，110 案 / 381 Pass / 0 非 Pass）；
   - A 侧 = M2 树 853e3c572（211 案 / 741 行 / **make_exit=0**）；
   - removed keys = 0（既有键零移动）；
   - shared 110 案逐案 (状态, 子测试名) 多重集**全等**（2 案仅同案内连接
     完成序漂移：Create-blocked-port.any.js 等，内容集相同）——既有面零回归；
   - added 101 案 = wss 变体池（`?wss` / `?wss&wpt_flags=https`），子测试
     360/360 全 Pass，**批次门 100% ≥ 98.5% ✓**，无挂账归因项。
3. **探针**（先于全量）：cookies 子集 `?wss&wpt_flags=https` 4 案全绿
   （https 页锚 + constants.sub.js wss 分支 + TLS fixture 全链）。

## 四、DC-2 落账与 M3 交底

- **DC-2 全项达成**：TLS 端点 fixture（M1）+ wss 端到端单测（M1）+ 通道
  scheme/端点寻址接线（本档）+ 既有通道零回归（本档 §三.2）。
- DC-4 wss 池前半：covered = 101 案 > 0，批次门 100%——终账随 M4 收口
  （五面 A/B + 头条不回落）。
- **M3 待做**（py 普查档 §三蓝图）：路由表框架（注册项 = 匹配谓词 + 方法
  豁免 + handler(&req,&EndpointCtx)；ctx = query 解析复用 + stash 表 +
  头构造助手）；既有 153 臂迁入（**逐字节等价硬门**——迁移前固化 net-api
  全通道 13,275P 面基线）；S12 cors-preflight-redirect 补丁收编 + 语料仓
  update 脚本缺口清偿；P-A/P-B 注册面 + 单测各 ≥1 例。

# M2-S347：内联 module 求值错误报告 lineno——evaluation-error-5 F→P

日期：2026-11-08。控制文档：`docs/goal/html-semantics-compat.md`（头部状态链 s347 段）。
本切片收口 s345/s346 承继挂账「内联 module 错误行号」：语料单钉面 evaluation-error-5
（文档 16 行 `<script type="module">a</script>` 求值抛错，window error 事件 lineno
须 16，红态 0）。域口径 the-script-element **24F→23F**，det F→P=1、P→F=∅、T=8 同集
零位移。spec 依据：
- report the exception / ErrorEvent lineno
  https://html.spec.whatwg.org/multipage/webappapis.html#report-the-error
- run a module script（求值失败同 fetch 成功后的失败类）
  https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script

## 定谳

1. **报告链缺 lineno 实参**（s255 哨兵报告面遗漏项）：内联 module 体求值抛错经
   `__zwModEvalErrObj` 哨兵 → post-check `script_module_eval_error_report` →
   `reportError(__zwErr)`——二三四参缺省，shim `reportError` lineno 落 0
   （part01.js:11870 `lineno == null ? 0`）。修复 = 报告器加 `lineno` 形参发射
   `reportError(__zwErr, '', LINE)`（colno/filename 语料无钉面维持空/0）。
2. **行号源 = M2-s2 CSP 位置表复用，非快照再扫描**：宿主侧原始源行号已有现成机制
   ——`config.csp_script_positions`（runner 经 `csp_script_positions_prepared` 注入
   prepared 序号空间原始源位置，**避开 harness 内联扭曲**——runner 装配页把外链
   harness 内联，快照行号漂移实测 16→5316；prepared 空间随 testharnessreport 等
   「整标签移除」剔除重映射，与 run 循环 script_index 同空间，M2-s2 原注）。首轮
   自建文本扫描方案（es_module 行对齐发射 + shim stack 解析 + `__zw_script_source_line`
   回调三件套）实现后 solo 实测即翻车（got 5316）——机制上输给既有位置表，全量
   回退（es_module padding、webview 编译回调第 3 参、扫描 helper 均撤净），改
   `script_positions.get(script_index).0` 三行接入。
3. **适用面收窄为内联模块**：`is_module && module_dep_base.is_none() &&
   !inlined_from_external`——外链模块体行号属模块自身 URL 内坐标（位置表是 case
   源坐标，两空间不可混）；runner 投递内联面（inlined_src）同此，维持 0（FIXME：
   语料无钉面）；csp_enforcement 关闭面表空回落 0 = 旧行为（浏览器车道本面 unpinned）。
4. **动态插入内联模块的求值错误行号同面未建模**（shim `__zw_compile_inline_module`
   求值臂 `_zwReportListenerError` lineno 0）——语料无钉面，挂账不硬解（勘根过程
   中证实其发射链与静态面独立，需 shim 侧行号通道，等语料钉面出现再立切片）。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| 报告器 lineno 形参 | engine `js_dom_bridge/script_gen.rs` `script_module_eval_error_report` | `(script_ordinal, lineno: u32)`；发射 `reportError(__zwErr, '', {lineno})`；doc 注明适用面与降级语义 |
| 行号接入 | webview `webview.rs` run_page_scripts | `eval_err_lineno` = 内联模块取 `script_positions[script_index].0`（M2-s2 prepared 位置表），其余 0；传入 post-check |

## 验证

- solo（`s347-solo-eval-error.log`）：evaluation-error-1..5 全 Pass——5 由 Fail 翻绿
  （红态证据 = s346 域跑 `s346-b-script-element.log` Fail 行），1-4 无回归。
- 首轮方案翻车实录（solo 首跑）：`expected 16 but got 5316`——快照行号扭曲实证，
  弃自建扫描改位置表（定谳 2）。
- 域 join（the-script-element 子域、键级 stable 集语义——`LC_ALL=C`、`grep -a`、
  (key,state) `sort -u`、双态键剔除）：A = `s346-b-script-element.log`
  **1095P/24F/8T**（raw 1096P 含 1 重复 Pass 键，s346 实证）；B =
  `s347-b-script-element.log` **1097P/23F/8T** raw。det F→P=**1**
  （evaluation-error-5 逐键吻合）；det P→F=**∅**；残余 F 集逐键相同；T=8 同集零位移
  （execution-timing 063/fetch-src empty 双案/load-error-events-3/credentials
  setTimeout 变体/code-cache-base-url/instantiation-error-8/script-crossorigin-network
  在册长尾）。语料口径 **24F→23F**。
- 单测（test-guard 包裹）：engine **3817P/6F**——6 案逐名同 s345 挂账⑦既有集
  （t8k websocket 对 + img/svg/fetch/iframe SW 四案），script_gen 签名变更零新增红；
  runner **298P/0F**（s346 基线持平）；webview **844P/1F**——1 案
  `image_decoder::tests::proxy_mode_decodes_png_correctly`「无法启动 image-decoder
  进程」**HEAD stash 后同红实证**（目标二进制缺席的环境依赖面，非本切片引入，
  挂账 3）。
- clippy（zero-engine + zero-webview --all-targets）**0 告警**；rustfmt 净。
- 本切片无既有测试断言变更；`script_module_eval_error_report` 唯一消费方 webview
  同步改签名（engine 内无其他调用方，grep 实证）。

## 挂账（承继 + 更新）

1. **外链模块求值错误行号**（模块自身 URL 内坐标）与 **runner 投递内联面行号**：
   维持 0，语料无钉面（FIXME 在码）。
2. **动态插入内联模块求值错误行号**（shim 求值臂 lineno 0）：语料无钉面，勘根记录
   在案（定谳 4），钉面出现再立切片。
3. **webview 既有 1 案环境依赖红**（image-decoder 进程二进制缺席，HEAD 同红）：
   与 runner 定向 clippy 29 既有 M3 债同性质碰头账——目标二进制构建面由其归属流
   或环境整备处理，非 html-semantics 切片内。
4. s345 既有承继原样：包装级 parse 原子性（choice-of-error-1/compilation-error-2
   双案，需 compile-only API 或宿主取回面）、getter 安装失败回落、同名多星号首见
   FIXME（均语料无钉面）、engine 6 案 M3 回归修复（挂账⑦）、s341-s346 承继、
   域长尾 22F+1（evaluation-error-5 翻绿后 23F 在册面：22 长尾 + 首轮方案撤净后
   无新增）。
5. 下一步候选：parse 原子性双案（勘 compile-only API 可行性优先）或域残余 F 面
   逐案勘根——下轮勘定；engine 6 案回归修复按 46号 M3 收敛插队。

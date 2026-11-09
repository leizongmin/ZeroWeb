# M3 切片五：残链清空批——末 21 臂迁入注册表 + dispatch 收口为「循环 + 静态尾」+ 文件族拆分

日期：2026-11-08。基线锚 = 合流后 HEAD `1212d5d38` 重录
（`/tmp/m3s5-base-214840.log`，systemd 单元 m3s5-base-214840，口径同切片四档）；
前置 = [切片四档](2026-11-08-m3-slice4-fetch-es-40arms.md) + [普查档]
(2026-11-08-slice4-residual-census.md)。本批落库后 **dispatch 残链清空**
（注册表循环 + 静态回退尾两段），注册表 85 行 = 原链 84 臂 + s346 合流行
（切片四 §五），DC-3 的 dispatch 主表迁移面收口。

## 一、交付面

- **残链 21 臂全量迁入**（普查档六a尾 + 六b + 六c 三批合并为一批执行——
  切片四合并批先例，省两轮全量 A/B 墙钟；普查「19 臂」概数与实提 21 之差
  = 405 门 + OPTIONS blanket 两臂的口径归属，臂清单以内容为准）：
  get-set-cookie（Exact）/ xhr .asis（DirSuffix）/ eventsource cors.py
  （DirSuffix）/ clean-stash、preflight（Exact ×2）/ **405 兜底门 +
  OPTIONS blanket（Custom ×2——req.method 面谓词首入表）** /
  inspect-headers（Suffix）/ fetch trickle（Exact）/ bad-chunk-encoding
  （Suffix）/ common/redirect、dom encoding、sw vary/fetch-status/
  cors-approved/cors-denied/cache-storage redirect、blank.html、
  websockets security check、xmldecl trickle、load-error-events（Exact ×11）。
- **405 门/OPTIONS blanket 必须在表内**（非表尾代码）：两臂谓词 method 面
  先于其后 inspect-headers/trickle 等臂与静态回退尾——移到循环后会先被
  method 无关的 Exact/Suffix 行截答（POST trickle 200 化、OPTIONS 静态文件
  落盘读），行为变。以 Custom 行原位承载，None=落穿语义不变。
- dispatch-top 局部绑定收敛：`get_set_cookie_stash`/`es_cookie_stash`/
  `vary_value_override` 三绑定随臂迁 handler 内 `&ctx.` 重绑定，
  dispatch 只剩 `root`（静态尾用）。
- **文件族拆分**（CLAUDE.md 2000 行守卫恢复——迁移后单文件 4,822 行）：
  `testharness_py_routes.rs` → 目录模块 `testharness_py_routes/{mod,site,
  xhr,fetch,es,misc}.rs`——mod.rs 只留底座（ctx 类型/谓词变体/注册表/
  dispatch），89 项 handler/谓词按端点族五分（site 15 / xhr 37 / fetch 17 /
  es 9 / misc 11）。纯移动：item 体逐字节保持，仅 `fn` → `pub(crate) fn`
  （表在 mod.rs 经 glob use 引用），共享 import 升 `pub(crate) use`
  re-export（re-export 不触发 unused 警告）。表序零改动（表文本不变）。

## 二、等价性证据

- **机械双件套**（/tmp/m3s5_verify.py，rustfmt 折行后按 token 归一复跑
  ALL PASS）：A——21 handler 体 vs HEAD 原臂体经 dedent 4 + Some 包装两
  变换后 token 序全等（21/21）；B——注册表谓词逐臂反推条件式 vs 原 if
  条件 token 序全等（21/21，链序一一配对）；B2——Custom 谓词 fn 体 =
  原条件式（2/2）；C——表行 85、dispatch 残链顶层 if 清零。
- **编译与单测**：cargo check 零警告（生成器 usage 检测一处误报修正：
  `clean-stash` 注释文本命中 `\bclean\b` 词边界——route_clean_stash 参数
  改 `_clean`）；`cargo fmt --check` 净；runner 单测 298P/0F
  （test-guard 包裹，与切片四合流后同数）。
- **全量 A/B**：基线锚重录（合流后 HEAD，见 §三）+ 迁移树 B 跑
  （`/tmp/m3s5-ab-220242.log`，systemd 单元 m3s5-ab-220242，22:02 起）——
  **multiset 空集（计数敏感口径）**：双侧总账 13,614P/1,305F/8T 全等
  （分母 14,927 一致，均跑至末案收尾），逐键计数 diff 空集；四域锚全等：
  websockets 面 Pass 741=741、S3b constructor/016 2=2、S12
  cors-preflight-redirect 10=10、inspect-headers/header-values 面 70=70。
- **拆分往返校验**：/tmp/m3s5_split.py 逐 item 子文件连续在场断言 +
  族清单 vs route/pred 清单完备性互斥断言（漏项/多项即中止）。

## 三、基线锚重录与漂移归因（合流扰动收账）

- 切片四 A/B 锚为合流前 16:41 态（该档 §五末注记）；本批按 master 下一步
  指示对合流后 HEAD 重录：`/tmp/m3s5-base-214840.log`（21:48 起，单元
  m3s5-base-214840，MAKE_EXIT=2 同形态，14,927 条分母一致）。
- 新锚 vs 16:41 锚 = **32 子测试 F→P 单向漂移，全部落 fetch/api/request
  request-cache 族 + mode-same-origin redirect 两案**——c6dcbce0a
  （http-cache M2 wire 传播）语义域，账归该提交与切片四档 §五合流记录，
  本批零回归判定不受影响（两侧同锚）。
- A/B 口径升级：对照脚本本批起保留 uniq -c 计数（切片四脚本剥计数后
  实为集合比较）——计数敏感 multiset 更强，分母 + 总账 + 逐键计数全等。
- **对照脚本 binary 陷阱**：runner 日志含原始非 UTF-8 字节（编码面案例
  直写 stdout），无 `-a` 时 GNU grep 二进制检测使 `-hE` 提取提前截断
  （`-c` 总数不受影响）——首轮对比曾出 20 条假 changed（xhr/status-basic
  等 4 案），加 `-a` 后假 diff 消失、真 multiset 空集。切片四档同款脚本
  存在同一潜在截断面（其总数口径不受影响），后续批次对照一律 `-a`。

## 四、S12 update 脚本缺口销账（普查档 §三 交付项核对）

- pin 树核对：`tests/wpt-runner/wpt-data/fetch/api/cors/
  cors-preflight-redirect.any.js` 现形态含「after preflight succeeded」腿
  （上游 651ffcb03 改写面）——2026-10-29 M2-S13 全量同步已原生含修复。
- `scripts/update-wpt-data.sh` 头部缺口清单第 37 行起已记该案对齐条目
  （「新 ref 快照 ≥ 2026-09-12 则无需补」条件成立）；本批补销账注记，
  升级重补负担清偿。S12 本地补丁本体（redirect.py OPTIONS 分支）已于
  切片四随 route_fetch_redirect 迁入注册表（该档 §四），收编闭环。

## 五、过程记录

- 基线锚先于源码编辑启动（HEAD 态跑 A），源码编辑待 A 跑完——「后台构建
  期间不改源码」约束下串行；两跑均 systemd-run 单元 + test-guard
  （--per-proc-mem 4 --total-mem 8 --time-limit 900）。
- 生成器（/tmp/m3s5_convert.py）四处迭代修于落库前：f-string 花括号、
  臂间上方注释块归属（405 门/bad-chunk/redirect 三处）、return 多行开启
  形态（`return Ok(FetchResponse {`）、dispatch-top 绑定歧义（新 handler
  内重绑定行同文）——均由断言拦截，未进提交。
- verify 脚本 rustfmt 折行适配：表 Exact/DirSuffix 长路径行被 rustfmt 展开
  （与既有表 122-147 行同形态），B 段解析改空白全剥 + 尾逗号容忍。
## 六、文件族拆分落库（§一交付面第四项的执行记录，独立第二提交）

- 拆分器 /tmp/m3s5_split.py 一次通过（DeprecationWarning 一处为脚本内部
  re.sub 计数位置参数，不影响产物）；89 项五族清单与 route/pred 清单
  完备性互斥断言通过，逐 item 子文件连续在场断言（round-trip）通过。
  产物（fmt 后终值，全部 ≤2000 行）：mod.rs 809（底座：ctx/谓词变体/
  注册表/dispatch）+ site.rs 645（15）+ xhr.rs 1,749（37）+ fetch.rs
  892（17）+ es.rs 474（9）+ misc.rs 336（11），合计 4,905 行；旧单文件
  git rm（E0761 防双模块歧义）。
- 表序零改动独立复核：`fn py_routes()` 表文本 vs 拆分前提交
  （3db404d45）逐字节全等，85 行两侧一致。
- 编译与单测：cargo fmt --check 净；cargo check -p zero-wpt-runner
  零警告（子模块 `use super::*` 反供 + `pub(crate) use` re-export 可见性
  接线一次成形）；runner 单测 298P/0F（单元 m3s5-split-ut-222421，
  与拆分前同数）。外部引用仅 testharness.rs :5934 rustdoc 链接补
  `site::` 段（route_module_fetch_tester 迁 site.rs 后 glob import 不再
  re-export，路径需全）；:7407/:7423（PyEndpointCtx/dispatch）留守
  mod.rs 路径不变。
- 全量 A/B 第三跑（拆分树，单元 m3s5-split-ab-222458，22:25:51 起
  runner，22:32:56 收尾，test-guard --per-proc-mem 4 --total-mem 8
  --time-limit 900，MAKE_EXIT=2 与基线同形态）：对 §三重录基线
  /tmp/m3s5-base-214840.log——**multiset 空集（计数敏感口径）**——总账
  13,614P/1,305F/8T 双侧全等（分母 14,927 一致），四域锚全等：wss
  741=741、S3b 016 2=2、S12 10=10、inspect-headers 70=70。拆分
  纯移动零行为变化落证。
- 过程注记：A/B 等待窗内本 session 五次实例中断/重启，各自落了 §六
  草稿（数值一致、两份含 fmt 前旧行数）——续轮合并为本节单稿落证；
  systemd 单元跨 session 存活是长跑未废的原因。

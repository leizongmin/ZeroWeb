# M2 s442 — 小尾巴分诊 + color.window.js 勘根：资源补齐揭 196 键真面全 Pass + 默认 30s completion-teardown hang 定谳（资源回收，零语料净变更）

日期：2026-11-15。承前：s441 下一步「小尾巴逐件勘根（combobox tentative
12 / tabular 4 / tails 10）或候门决项」。本轮机器载入他流 reftest-upstream
重腿在跑（loadavg ~6），全程只跑 FILTER solo 轻验证，未开全量双腿。

## 小尾巴分诊结论（s441 join 206F 内非门非跨域面逐件过）

| 面 | 键 | 归属判定 |
|---|---|---|
| tabular col-span-limits | 2 | **渲染域**——两失败键均断言 `offsetWidth`（1000 列布局宽度）；clamp 语义三键（script set / attribute parse / column span）**已 Pass**，引擎 colspan [1,1000] 钳制健全。非本域可动 |
| tabular rowspan-0(-quirks) | 2 | 渲染域（`clientHeight` 布局盒断言） |
| input checkable-active-onblur | 2 | s209 在册 onblur flake 族 |
| input color.window.js | 1 | **本轮勘根主体**（见下） |
| label forwarded-click ×2 | 2 | 键 2（label.click() 不可信转发）疑引擎事件面——留勘；键 1 依赖 testdriver 真输入管道 |
| details/embed/textarea/audio 余 | 7 | content-visibility / intrinsic sizing / scroll 容器——渲染域或 runner 能力门 |
| combobox tentative | 12 | 未入本轮（popover/键盘指针合成面，疑能力门族，留续分诊） |

分诊后 in-domain 直接可动面进一步收窄；本轮在 color.window.js 上勘出高
价值真缺口（下）。

## color.window.js 勘根链（"load support" 假 Fail → 真面 → hang）

1. **假 Fail 归因**：META `script=/css/support/color-testcommon.js` 缺席
   ——css/support/ 目录由他 goal 部分拉取（1x1-green.png 等）未覆盖此文
   件，无任何 goals/*.sh fetch 行。上游 pin 315976933 **在档**（jsdelivr
   实测 167 行 self-contained，`set_up_fuzzy_color_test` 模糊比较器，零
   二级依赖）。s227 先验判据同型：缺资产 → load support 假 Fail 遮蔽全
   部子测真面。
2. **真面（资源就位后 solo FILTER，四次独立复跑逐键一致）**：
   **196/196 全 Pass**（24 色值条目 × 2 colorSpace × 2 alpha × 2 setter
   形态 = 192 生成 + 4 standalone），零 Fail 零 Timeout——引擎 color
   input value 解析/序列化/limited-srgb/display-p3/alpha 重解析面健全，
   假 Fail 背后无真红。
3. **定谳：默认 30s deadline → completion-teardown hang**：
   - 默认 deadline（30s，无 env 覆写）solo ×3：末子测
     `Adding alpha reparses the original value attribute for a non-dirty
     value` 打印后**进程树静默僵死**——输出截断于行中、无 Timeout 行、无
     make 错误行、无 rc；活体采样：runner 进程 `State S`、wchan
     `do_wait`、CPU time 0、RSS ~5.5MB 恒定（非内存炸弹，非 JS 自旋）。
     Rust 侧 30s deadline 轮询（`Instant::now() >= deadline`）不可达 =
     单个 `execute_script` 调用同步不返回。
   - **`ZW_CORPUS_CASE_TIMEOUT_SECS=15` solo：干净 rc=0**，196 Pass 零
     Timeout——runner deadline 先于 harness 页预算到期 → 走 force-collect
     路径（"completion callback was not called" 收集臂）收干净结果退出。
   - **机制判定（假设，未逐行钉死）**：注入 harness 页预算与 case
     deadline 同源同值（30s）；二者同时到期时 harness timeout handler 在
     shim 内同步跑 completion/teardown → 无限循环 → execute_script 阻
     死 → Rust deadline 检查永不可达。15s 时 runner 先到 → 收集退出。
   - **全量波及判定**：全量 sweep 同为默认 30s → 该案 worker 挂死 →
     parallel_map join 永不归 → 5400s guard 杀全 run，sweep 失败。资源
     补齐在 hang 修复前**不可落地**。
4. **处置**：25 脚本 ③l fetch_raw 行撤销（回 s441 态）+ 本地资源文件删
   除，**零语料净变更**，sweep 完整性优先。假 Fail 键留在账面（在册资源
   遮蔽面，账本留修复路线）。

## 修复路线（下一轮）

勘 shim completion/timeout teardown 挂死根因（testharness.js timeout
handler → shim completion 路径的同步无限循环；活体证据链已足：15s/30s
对照 + do_wait 活体采样），修复后重落 ③l `fetch_raw
"css/support/color-testcommon.js"` → 预期键面位移 F−1（load support 假
Fail 消失）+ P+196（真面全 Pass 按本轮四跑实证），全量 join 位移按语料
变更归因非回归。

## 验证工件

/tmp/s442/{color-solo,color-solo2,color-solo3,color-solo4}.log（默认
30s hang ×4，196 Pass 后静默截断）、color-solo5.log（15s 覆写，rc=0
完整收口）、/tmp/s442-color-testcommon.js（上游字节，已回删）。清场核
对：runner 进程零残留、工作树零 diff（脚本 git checkout 回 s441 态 + 语
料文件已删）。

## 结论

s442 收口：小尾巴分诊收窄 in-domain 面（col-span/rowspan/onblur 等
9 键归渲染域与在册族）；color.window.js 勘出「资源假 Fail 遮蔽 +
196 真面全 Pass + 默认 30s completion-teardown hang」三段定谳，资源回
收保 sweep 完整性，hang 勘根为下一轮主攻。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137。

## 组合树复验（push 后他流入树双腿复验）

s442 提交 push 时 rebase 入他流三笔：f6a70ef86（ip 左臂携集精化，
part118 pins）、74c1159c3（g45 image-set 文法位，css-parser/engine
shim part01f/part06 + computed_style）、78cb6723d（R5579 账本），
组合树 HEAD 4d3c92fcd。按并行归因纪律双腿复验本语料基带（runner 随
g45 重编译）：

- 键集口径注记：s441 fixr 集与本次原始提取存在尾 tab/双空格归一化伪
  影（149.html `for='  WINdow\t`、wrapping `exact  len`），原始日志字
  节恒等；按统一规范化（strip 尾空白 + uniq）对照。
- **键全集 8320 逐键恒等，零 GONE/NEW**；leg1 与 s441 stable 全集逐键
  同状态恒等（8107P/206F/6T/1U）。
- 双腿同状态 join **8319 键（8106P/206F/6T/1U，uniq 口径）**：恰 1 键
  置换——`dialog-overlay.html :: dialog computed overlay initially
  'none'` leg2 Fail 单腿振荡（同文件姊妹键 Opening-and-closing 全轮恒
  Pass；该键为 computed overlay 初值断言，渲染邻接；单腿 = 负载 flake
  签名，非代码位移签名——g45 类变更会双腿确定性翻转）。按 s441 在册
  负载族振荡记账，**基带 8113P/206F/6T/1U 维持**。
- 附注：全量 sweep `make 错误 1` + rc=2 为该 runner 正常收尾形态
  （s441 leg1 同型），非异常。
- 结论：g45 + f6a70ef86 对 html-semantics 语料零位移，与 s441 对
  fb81665dd 复验结论同型；下一步（shim hang 勘根）基带归因面干净。

工件：/tmp/s442/{leg1,leg2}.log、leg{1,2}.ks、{leg1,leg2,s441}.norm、
gone.txt/new.txt（规范化前伪影样本）。

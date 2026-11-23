# DC-2 切片 6 证据：domparsing 簇 DP-① runner xhtml 入扫 + DP-② CustomElementRegistry 可构造（2026-11-23）

## 定位

- **DP-①（runner 扩展名清单）**：`run_domparsing_cases` 走 `run_corpus_subdirs_ext`
  只收 `.html`——6 个 `.xhtml` 变体页（insert_adjacent_html-xhtml、
  innerhtml-01/03/05、createContextualFragment-xhtml、xml-serialization）整簇
  0 出扫。runner 已有 `exts` 参数（刀③④ 引入），补清单即入扫。
- **DP-②（CustomElementRegistry 构造器）**：part03 接口构造器 throw TypeError
  守卫是 scoped-registries 之前代的旧行为。现行 HTML spec 已增设
  [CustomElementRegistry() constructor](https://html.spec.whatwg.org/multipage/custom-elements.html#dom-customelementregistry-constructor)
  （scoped registries 前置面）。corpus ground truth：custom-elements/registries/
  全簇、sanitizer-api、domparsing/shadow-dom tentative 页均 `new
  CustomElementRegistry()`，**零页面断言其抛**——旧守卫误杀整页
  （registries 簇页级 Fail 死标记＋declared tests 塌零）。

## 交付面

- **tests/wpt-runner/src/testharness.rs（DP-①）**：`run_domparsing_cases` 扩展名
  清单 `&["html", "xhtml"]`，其余套件构造性零变化（各自清单不变）。DOMPARSING_
  SUBDIRS 注释声明：.xhtml 页内容走 HTML 解析管线近似，XML 文档形态断言面残 F
  记对账（FIXME 解除）。
- **crates/engine/src/js_dom_shim/part03.js（DP-②）**：
  - `_zwMakeCERegistry` 工厂补 `upgrade` no-op（子实例诚实 stub——spec upgrade
    按本 registry 状态，`_ceUpgradeNode` 读主实例状态未参数化，升级路由随
    scoped registry 后续片，FIXME 在注）。
  - 接口构造器 throw → 工厂返回：`Object.setPrototypeOf(_zwMakeCERegistry(),
    _CERegistryIface.prototype)`——prototype 链挂接口原型（instanceof 面保持），
    `new` 表达式取返回对象。prototype 方法（define/get/getName/whenDefined/
    upgrade 委托 globalThis.customElements）不变（实例 own 方法遮蔽）。
- **crates/engine/src/js_dom_bridge_tests/part28.rs（测试契约更新，同提交）**：
  `wc_m1_ce_define_domexception_types` 末段从断言 `new` 抛 TypeError 改为断言
  构造成功（`r2 instanceof CustomElementRegistry` 且 `r2 !== customElements`）
  ——旧断言编码的正是本刀移除的过时守卫。

## 语料对账（release 重建后全量跑）

### domparsing 套件面（DP-①＋DP-② 联合）

- 基线 154P/1318F（1472 行）→ **171P/1364F/1T（1536 行，+17P/+46F/+1T）**。
- **+17P 全部落 .xhtml 页**：insert_adjacent_html-xhtml +12、innerhtml-03 +3、
  createContextualFragment-xhtml +2。既有 .html 面逐行零漂移。
- +46F：44F＝.xhtml 页 XML 文档语义残 F（XML serialization/文档形态断言 vs
  HTML 管线近似，DP-① 注释已声明该限制，后续 XML 管线深项刀位）；3F＋1 页级
  Timeout＝tentative/stream-html-custom-element-sanitizer 页（DP-② 解锁后从
  页级 Fail 死标记转为可运行：3 子测 F＋页 TO，**得分净零，如实入册**——
  该页 pre 面 1 条页级 Fail，post 面 3F+1T，Pass 零增）。

### web-components 套件面（DP-② A/B 隔离判读）

dc1-wc.log 基线过期（早于切片 1/2 ref-target +8,738P）不可直用——按字节绑定
纪律做干净 A/B：post 面（DP-② 字节）已在手（/tmp/dp1-wc-post.log，17,905P/
4107F/18T，22,030 行），pre 面 git stash part03 回 HEAD 字节（md5 复核
77e2c06a…＝HEAD ✓）全新跑（21,905 行，WC-EXIT=2），pop 恢复（md5 复核
96e69eda…＝刀字节 ✓）。

逐行 comm（LC_ALL=C awk 提取 `^(Pass|Fail|Timeout|NotRun) ` 行，双侧 sort 后
comm）：**post-only 147 行 / pre-only 22 行，diff 面 100% 限定
custom-elements/registries/ 11 页**，wc 其余面（custom-elements 主簇、
shadow-dom、template-element）逐行零漂移＝本刀隔离干净。

- **+24P**：13 键 F→P 完美翻转（Construct 1、CustomElementRegistry-define 3、
  multi-register 2、registry-define-get-etc 7）＋ 11 键全新 Pass
  （Document-createElement 2、createElementNS 2、importNode 7——这些页 pre 面
  整页死在 `new CustomElementRegistry()`，子测注册根本不存在）。
- −22F：13 键翻转为 P；5 条页级 Fail 死标记消失（页复活）；4 F
  （scoped-registry-define-upgrade-criteria 页）被新页级 Timeout 吞——**该页
  pre 4F → post 页级 TO（其 4 子测随页死），P 净零但完成度回退，如实入册**
  （页现在跑 detached frames/constructed documents 深面后 TO，与已知停摆形
  同族，随页级 TO 挂账）。
- +122F＝此前未注册的子测现在注册并失败（scoped registry 隔离语义、upgrade
  criteria、define/get 深面）——**新可见失败面**，是后续 registries 刀位的
  输入清单，非本刀破坏（pre 面无这些行）。
- 1 新页级 Timeout＝define-upgrade-criteria（上条）。

## 判读字节绑定

发射前快照（/tmp/dp1-knife-bind.md5）part03 `96e69edad2c768f7f636a6dd0478200c`
＋testharness.rs `b3ba819b60a5a647cbcd929f50ecc572`；判读后复验一致。
part28.rs 契约更新在语料跑之后（4f3d916f…），不参与 shim 装配（测试文件），
判读域零涉。

## 回归网

- **zero-engine lib unittests 4252P/0F**（test-guard `--compile-first`，
  17.45s）＝各刀记录值精确恒等（part28 契约更新后复跑确认）。
- `cargo fmt --all -- --check` 绿；`cargo clippy -p zero-wpt-runner
  --all-targets -- -D warnings` 绿（4.72s）；`cargo clippy -p zero-engine
  --all-targets -- -D warnings` 绿（18.65s，Rust 面两包全扫）。
- shim 全 28 part 拼接后 `node --check` 绿（part03 单件是装配片段不可单检）。
- 判读日志入库：[dc2-slice6-domparsing-post.log](dc2-slice6-domparsing-post.log)、
  [dc2-slice6-wc-post.log](dc2-slice6-wc-post.log)（新判读基线；pre 面字节＝
  HEAD part03 可自 git 复构，comm 清单见上文）。

## 净账与挂账

- 本刀净增 **+41P**（domparsing +17 ＋ wc registries +24）；累计
  79,567 → **≈79,608 / 107,971**，距目标 ≥99,947 余缺口 ~20,339。
- registries 新可见 122F 分域挂账：scoped registry 隔离/继承语义（主簇）、
  upgrade criteria（define-upgrade-criteria 页级 TO 停摆形在册）、
  define/get/getName/whenDefined 深面。
- .xhtml XML 文档语义残 44F 挂 XML 管线深项（DP-① 只解入扫，不解语义）。
- 下一轮主菜：domparsing tentative 位置方法族（WICG
  declarative-partial-updates，~1,050F：appendHTML/prependHTML/beforeHTML/
  afterHTML/replaceWithHTML ± Unsafe，建在既有 fragment-parse＋
  insertAdjacentHTML 机械上）。

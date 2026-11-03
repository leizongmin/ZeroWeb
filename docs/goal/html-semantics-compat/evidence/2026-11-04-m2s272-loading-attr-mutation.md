# M2 s272 — loading 属性变更触发面（to-eager 收口：deferred 图 eager 化唤醒，2 文件 3 刀）

日期：2026-11-04。切片号 s272（承 s271 下一步候选①：型 C eager 变更交互
勘根）。2 文件 3 刀（engine shim 单车道，零 runner 改动）。

## 勘根

1. **型 C 四页主干判定**（wpt.fyi API summary，2026-11-03 run，revision
   52041a16d3）：
   - image-loading-lazy-to-eager：chrome [1,1] + firefox [1,1]
     **双引擎主干绿** → 真缺口，本切施工目标；
   - below-viewport-dynamic / in-viewport-dynamic：chrome [1,1] /
     firefox [0,1] **Chrome-only 绿**，且与 spec 字面冲突（fetch 步
     「Let delay load event be true if the img's lazy loading attribute
     is in the Eager state」——eager 化后请求按 spec 应阻 window load，
     页期望不阻=Chromium 特有语义面）→ 挂账（牵 inflight 阻塞链调度，
     全树时序面，非本切风险内）；
   - multiple-times：chrome [0,1] / firefox [0,1] **双引擎主干红**
     → 非缺口，销案（solo 签名 assert_unreached「换 src 后不 eager
     装载」= restart 绕门语义，主干同红不追）。
2. **to-eager Timeout 根因**（探针 zz-s272-probe/2 双轮，验后删）：
   - 探针 v1：两 below-viewport lazy 图 eager 化后 window load **已
     派发**（load:true 排除 load 阻塞面），但 300ms 两图 complete
     false——装载未发生；
   - 触发链计数（monkey-patch __zwImgAttrMutated5277 /
     __zwImgRevalidate5277）：IDL setter `img.loading='eager'` 到达
     helper（attr 数组 `loading(+)`、revalidate 1 次、该图装载成功
     complete true）；**removeAttribute('loading') 零触发**——
     Promise.all 单腿挂死 → 文件 Timeout 全解释。
3. **三汇流点名单差**：part05 IDL set trap 汇流点（1282）无名单过滤
   （tag/名称门全在 helper）→ IDL 路径通；**part04 setAttribute 臂
   （4032）/ removeAttribute 臂（4325）名单缺 `loading`** → 内容属
   性路径断（s70 名单 src/srcset/sizes/crossorigin/referrerpolicy/
   media/type）。helper（part03 `__zwImgAttrMutated5277`）IMG 臂同
   缺 loading 分支。
4. **spec 定谳与刀形修正**（初刀无条件 revalidate 后全树 A/B 撞
   relevant-mutations-lazy 双子测 Pass→Fail）：spec relevant
   mutations 定义（HTML §4.8.4.3.2 Reacting to DOM mutations）
   **不含 loading 属性**；页 relevant-mutations-lazy 注释明标
   「should not invoke update the image data」——已装载图 loading
   变更（eager 化/LAZY 化）no-op timeout。与 to-eager（deferred 图
   eager 化须装载）统一语义 = **仅唤醒未启动图**（Chromium
   LoadDeferredImage 只唤醒 deferred）：刀形补 `!_resourceStates[
   _elKey(sel,handle)]` 条件（= `_imHadPrev` 判据同源）。
5. **checkable-active-onblur-with-click 摇摆定谳**（b1 轮 P→F 假
   回归）：基线二进制 solo 同红（following label 子测 Fail）+ B 态
   同二进制 b1/b2 重跑翻转（Fail→Pass）+ 页面零 img/loading 交集
   ——调度环境敏感噪声，非刀回归。

## 施工（2 文件 3 刀）

1. **part03 helper loading 分支**（`__zwImgAttrMutated5277` IMG 臂，
   crossorigin/referrerpolicy 分支后）：`ln === 'loading' &&
   !_resourceStates[_elKey(sel,handle)]` → `__zwImgRevalidate5277`
   ——仅未启动图唤醒，注释含 spec relevant mutations 排除引证 +
   Chromium LoadDeferredImage 语义 + 两页断言面对照。
2. **part04 setAttribute 臂名单**（4032）：加 `|| n === 'loading'`
   （s272 注记：内容属性路径 run 触发器）。
3. **part04 removeAttribute 臂名单**（4325）：同加 `|| n === 'loading'`
   （removed 臂，`_rmExisted` 门既有）。

## 验证

- **solo**：to-eager Timeout→**Pass**（日志 s272-solo-toeager.log）；
  relevant-mutations-lazy 全子测 **Pass**（回归修复，
  s272-solo-relevant-mutations-lazy.log）。
- **族**（FILTER=image-loading-lazy，37 结果行 / 27 文件）：
  **27P/7F/3T**——与 s271 族归一 diff **唯一位移 = to-eager
  Timeout→Pass**，其余（base-url ×2 / below-viewport-dynamic /
  different-crossorigin / in-viewport-dynamic / multiple-times /
  image-loading-lazy / srcset / below-viewport-load-event）双侧
  零位移（日志 s272-family.log）。
- **全树 A/B**（stash 同二进制法，test-guard + RAF）：A 8343 键 /
  B2 8343 键，(state+path) join **REGRESSED=∅**（第十二个连续完美
  A/B）：
  - FIXED 键形换代：to-eager 文件级 Timeout 键（A-only）→ 子测
    Pass 键（B-only，整页红→绿）；
  - checkable-active-onblur-with-click「following label」双子测
    三跑摇摆（a Fail / b1 Fail / b2 Pass，Checkboxes+Radios 同步
    摇摆）——同二进制 b1/b2 翻转 + 基线 solo 同红排除，非刀回归
    （该子测历史 s270/s271 四轮全 Pass，本轮偶发）。
  - b1（修正刀形前二进制）撞 relevant-mutations-lazy 双子测
    Pass→Fail → 初刀无条件 revalidate 过宽的直接证据，修正刀形
    后 b2 与 a 同绿（键集零差）。
  （日志 s272-fulltree-{a,b2}.log；b1 遗弃不提交——修正前态。）
- **make test**（test-guard）：67 test-result 全绿，唯一 FAILED =
  engine **3724 passed + 5 failed**——R-t2qq adopted-stylesheets
  既有 5 红同集（s265/s268/s270/s271 账本记档），本切零位移。
- fmt：`cargo fmt --all --check` 干净；make test 并行 clippy 臂
  （-D warnings）零 error 完成。

## 挂账更新

- 型 C **销案三分**：to-eager 收口（本切）；multiple-times 主干
  双红销案；below-viewport-dynamic / in-viewport-dynamic 转挂账
  （Chrome-only 绿口径 vs spec delay-load 字面冲突，牵 inflight
  阻塞链调度——「lazy 图请求不阻 window load + loading 变更不重开
  进行中请求」语义面，与 F5 视口模型收口联动权衡）。
- 新记录：**checkable-active-onblur-with-click 全树摇摆子测**
  （following label Checkboxes+Radios 同步摇摆，调度环境敏感，
  A/B 归一时须按 (state,path) 键集 + 重跑仲裁，本次已记仲裁证据）。
- 承前不动：型 B base-url ×2；型 D image-loading-lazy（主干
  [4,4] 全绿，我侧文件 Timeout——挂死根因优先）/ srcset /
  below-viewport-load-event；型 E different-crossorigin；runner
  全量钉内存大户二分（s270 挂账①）；root margin 净速分档未建模
  （s271 挂账①）；dynamic import() 元素级 referrerpolicy 穿参
  （零 corpus face）。

## 下一步候选

1. 型 D 文件级 Timeout 勘根（image-loading-lazy 主干全绿 [4,4]
   / srcset / below-viewport-load-event——挂死根因定位，页级
   子测可见性面）。
2. 型 B base-url ×2 勘根（parse-time base URL 冻结面——s225 延步
   重选源机制与 lazy 门交互，settle 分流后基线态）。
3. 型 E different-crossorigin（avail 元组面——list of available
   images 元组键比对）。

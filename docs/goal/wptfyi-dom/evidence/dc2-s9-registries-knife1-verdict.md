# DC-2 s9-registries 刀① 判读收账（2026-11-24 06:2x，session 403b8e7a 续轮）

## 范围

Scoped Custom Element Registries（whatwg/html#10854）基础面纯 JS shim 刀：
`new CustomElementRegistry()` + `initialize(root)`（Element/ShadowRoot/Document 三形）+
Element/Document/ShadowRoot `customElementRegistry` getters + `createElement(NS)`
`{customElementRegistry}` option + `attachShadow` option + `customelementregistry`
属性 live 读钉 null。编辑面 5 文件：part01（iframe realm doc getter）/ part03（CE 区：
解析核 `_zwCeResolveElementRegistry`/`_zwCeResolveRegistryEntry`/slot WeakMap/
pinned-null 哨兵/initialize 升级走查）/ part04（R5433 clone 臂 slot 复制 + DSR 重钉）/
part05（attachShadow option + handle 域 bind + getter IIFE）/ part06（createElement(NS)
option hook + scoped 表升级）。零 Rust。

## 主判读（testharness-web-components 全语料）

- post：**17995P / 4016F / 19T** vs 基线（s7/s8 恒等链，dc2-slice6-wc-post.log）
  17905P / 4107F / 18T = **净 +90P / −91F / +1T**。
- 证据：evidence/dc2-s9-wc-post.log（本轮全腿）。基线外漂移 = 0：页集合 comm 差
  diff = 13 页（全在 custom-elements/registries/ 簇），其余 346 页零漂移（per-key
  对账闭合：+79 新 Pass 键 + 11 flip 键 Pass 侧 = +90 ✓）。
- 逐页（registries 簇）：scoped-registry-initialize 27F→2F、
  CustomElementRegistry-initialize 13F→1F、Element-customElementRegistry 11F→4F、
  ShadowRoot-init 12F+3F→2F、Document-customElementRegistry 4F→1F、exceptions 3F→0F、
  attribute 页 ~20F→6F、Element-innerHTML 12F→2P/9F/1T（见下节）。

## Element-innerHTML Timeout 归因（8 轮探针定谳，非刀① 缺陷）

页遗留确定性 Timeout（pending:1，执行看门狗 ~1s 烧断，非墙钟）。归因链：

1. A/B stash 探针：HEAD 字节 → P=0/F=12/无 TO；刀① → P=2/F=9/T=1。
2. 禁 getter IIFE（probe3）→ TO 消失；单禁 Element getter（probe4b）→ TO 消失；
   walk 砍到 own-slot 快径（probe5）/换 no-op setter（probe6）→ TO 持续。
3. 全页复刻步进（replica2）：**空测试都不完成** → 挂点 = test 11 cleanup 段。
   去 cleanup（replica4）→ 零挂；去 `insertAdjacentHTML('afterend')`（replica3）→ 零挂。
4. marker-throw getter（probe7/8）→ TO 消失且 7 个栈标记全为测试体直接读
   （`Test.<anonymous>` 帧，零 shim 内部调用方帧）。

**因果链**：getter 缺席时测试体首个 `assert_equals(el.customElementRegistry, …)`
即抛 → 后续 DOM 操作全部不运行；getter 在场（刀① +2P 面）→ 测试首次跑到
`insertAdjacentHTML('afterend', …)` → **既有 shim 缺口暴露**：IAH 对 handle-only
元素（createElement 产物，sel=null）在 part04:12533 `if (sel && …)` 整段跳过宿主
插入（part01.js:11632 宿主通道仅 sel 形），afterend 补偿视图 prevSibling 恒 null
无反向锚 → `div.nextSibling` 恒 null（HEAD 侧同 null，从未被观测——旧基线该测试
在 getter 断言即中止）→ cleanup `div.remove()` 后 overlay/adjust 链烧看门狗 → 页
Timeout。宿主侧 `insert_nodes_at_position` afterend 臂本身正确（js_dom_bridge.rs:3296）。

**处置**：补偿锚返修试验（afterend 首节点 prevSibling=目标元）不修页（视图并入
需桶登记链，slot-only 不入 `_zwOverlayPendingChildNodes` 的 `bucket.added`）→
**已回退**，不携未验证共享面行为变化出刀。IAH handle-only 插入复杂体记入刀②
候选（归 R3254-K3 插入面刀位，非 registries 域）。终态 2P/9F/1T 为确定性常量，
主判读 +90P 含此 T。

## 守卫腿（判读 vs s8b chain6 锚，均零漂移）

- testharness-domparsing：**1264P / 271F / 1T** = 锚态逐位一致（解析产物升级路径
  + 门翻转面中性）。evidence/dc2-s9-domparsing-post.log。
- testharness-sanitizer-api：**414P / 342F / 1T** = 锚态逐位一致（该语料页首行
  `new CustomElementRegistry()` → `__zwCeScopedActive` 门激活面，gated 路径全量
  在飞下零回归——门参数化 {markActive} 收敛生效）。evidence/dc2-s9-sanitizer-post.log。

## 字节锚

- 判读绑定二进制 md5（终刀，solo 复核 P2/F9/T1 后）：`9503ec9605212d30fc6e47d81d3bf729`。
- shim 五文件与判读态逐字节恒（part03 d72a213d / part05 ede44f313 与判读时备份
  md5 恒；part01/04/06 探针全程未触碰）。探针残留字符串二进制 grep = 0
  （PROBE2/S9MARK/PROBE4b/no-op setter 四标记 -a 全零）。
- 判读方法：per-(page,status) 键 `uniq -c` 对账（页 TOTAL 对 Pass↔Fail 翻转盲——
  本轮教训两次 join-on-totals 翻车后定谱）；行提取 LC_ALL=C awk；二进制特征串
  `grep -a`。

## 刀② 候选（随档）

insertion/adopt effective-registry 矩阵（effective-global-registry 56F 残面）/
append 14F / element-mutation 15F / R90 trap scoped 原型臂（instanceof 残面）/
parse-time attr 区分（live 读恒 parse 面，set-after-parse 3 案反向）/ DSR
shadowrootcustomelementregistry 引擎 deep JSON 字段 / importNode·upgrade-order·
criteria 页 / **IAH handle-only 插入复杂体**（本刀归因产出：sel 门跳宿主 +
补偿反向锚 gap + remove 看门狗环）。

# DC-2 s10-registries 刀②续（knife2b · DSR contents 补注册）判读收账（2026-11-24）

席位链：写者 4c6091cf（v3d→v3e→v3f 绑定链，11:08:44 死于 429 未判读）→ 核验方
ae172c18 轮（11:10 接管：fmt＋重绑 725c47d5＋钉测双腿绿，11:19:02 死于 429，
clippy 中断）→ 本席（rally 新轮）接管收尾：取证钉死前两席事件、复跑钉测、
修 clippy 一处、solo 重跑被毒化 dom 腿、判读、落档。全程取证见
`.acceptance/dps/s10k2-interference-1024.md`（本地协调面）。

## 范围

DSR（declarative shadow root）空 contents 补注册 + 克隆转登——刀② 判读档挂账的
「B 域 DSR 物化」首项。html5ever 仅在模板收到首个子节点时注册 contents（惰性
语义）＝**空 contents 声明式 template 永不注册**，物化消费臂恒空转、宿主 shadow
root 缺失。

## 修复面（五件）

1. **zero-dom parser.rs**：`create_element` 空 contents 即注册 fragment（补 html5ever
   惰性缺口）。
2. **zero-dom shadow.rs**（+299）：`register_declarative_empty_contents` 补注册
   sweep（先补注册再物化）＋ `materialize_declarative_shadow_roots` 消费臂对补注册
   空 contents 的处理＋ 常驻钉测模块 `s10k2_empty_contents_tests`（3 测试）。
3. **zero-engine js_dom_bridge.rs**（+16/−2）：`template_contents_json_doc` sr 字段
   发射面（模板 contents 视图中宿主 SR 存在性出桥）。
4. **part03.js**（+25/−2 staged＋+2 unstaged）：`_zwTransferDeclarativeSR` 克隆转登
   ＋SR 视图挂载；两处 false 门复活＝clone-transfer 转登启用（v3f 面）。
5. **part148_s10k2_dsr_contents_pins.rs**（新件）：引擎常驻钉测三面＝① doc-parse
   物化面（静态 contents 视图 host.shadowRoot 非 null＋open＋clonable 位＋template
   已消费）② cloneNode(true) 转登面 ③ 入文档面（append 后 docfind＋SR
   querySelector/appendChild）。

## 钉测实测（本席复跑，test-guard 内）

- `cargo test -p zero-engine --lib s10k2` → **1 passed; 0 failed**（part148 三面
  单测试函数）。
- `cargo test -p zero-dom --lib s10k2_empty_contents` → **3 passed; 0 failed**。
- fmt `--all -- --check` CLEAN；clippy `-p zero-dom`/`-p zero-engine`
  `--all-targets -D warnings` 双绿（本席修一处：shadow.rs 钉测模块未用
  `use super::*` 删除——cfg(test) 域，lib 行为零差）。

## 字节绑定链

- 电池四腿二进制＝**`b5ff87b6`**（v3f，写者 11:03 构建）。核验方 11:15:03 重建
  `725c47d5` 为 fmt-only 重绑（11:13:18 shadow.rs mtime＝`cargo fmt --all`，transcript
  取证钉死）；本席后续唯一源码变更＝cfg(test) unused import 删除。两 delta 均
  lib 行为零差，判读携带。
- part03.js 终态 md5 `d52bf54cacb0a11b026fbd9ddab26a0a`；shadow.rs
  `2bbf004829dc59a723fca31a59a4b2ca`；parser.rs `3549cf603c0ddc99bf40caac0a37b36c`；
  part148 `e08835330775739b9e84ff8cc95d6d7a`。

## 电池判读（锚＝已推 刀② 17995P/4016F·1264P/271F·414P/342F）

| 腿 | knife2b 终态 | 对已推 刀② | 判读 |
|---|---|---|---|
| testharness-web-components | **18015P/3996F/19T** | **+20P/−20F** | 六页全限定（下表），其余 459 页零漂移 |
| testharness-domparsing | 1264P/271F/1T | 恒等 | 与 knife2b 锚**逐字节恒等**（cmp）＋SET-IDENTICAL |
| testharness-sanitizer-api | 414P/342F/1T | 恒等 | 对已推 刀② **SET-IDENTICAL**（42 键逐键相等） |
| testharness-dom | 57865P/496F/12T（r2 solo） | 无已推基线 | 对 knife2b 锚 −15P＝Range-mutations 双页噪声（下节） |

### WC +20P/−20F 逐页归因（六页全限定 registries/shadow-dom 簇）

| 页 | 刀② | 刀②续 | Δ |
|---|---|---|---|
| custom-elements/registries/scoped-registry-effective-global-registry.html | 10P/56F | 18P/48F | **+8P/−8F**（DSR contents 主病页） |
| shadow-dom/declarative/declarative-shadow-dom-opt-in.html | 105P/13F | 109P/9F | +4P/−4F（station 泄入 inert 文档修复，v3e scripting 门） |
| custom-elements/registries/scoped-registry-append.html | 2P/14F | 5P/11F | +3P/−3F |
| shadow-dom/declarative/declarative-shadow-dom-basic.html | 13P/9F | 16P/6F | +3P/−3F |
| shadow-dom/reference-target/tentative/reference-target-basics.html | 3P/3F | 6P/0F | +3P/−3F（页全绿） |
| shadow-dom/attachShadow-with-ShadowRoot.html | 0P/2F | 2P/0F | +2P/−2F（页全绿） |
| shadow-dom/shadow-root-clonable.html | 3P/3F | 4P/2F | +1P/−1F |

键面 466→465＝attachShadow-with-ShadowRoot 的 Fail 键清空（页全绿），非键消失。

### dom 腿（r2 solo 重跑）

首跑（11:10–11:19）尾段与核验方在飞 `cargo clippy`（11:15:47 起）并发＝墙钟毒化
（−362P 作废）。r2 solo（11:34–11:42）57865P/496F/12T 对 knife2b 锚 57880P/496F/11T：
`Range-mutations-dataChange` 1168→1178（+10）、`Range-mutations-replaceData`
1146→1121＋页级 TO（−25）——双页均属已录 Range-mutations 截断抖动/停摆形噪声族
（±45P 带，停摆点逐窗漂移；master 碰撞记录 2026-11-23 两则同形）。刀域（DSR
contents）与 dom/ranges 页零交集，无归因路径。其余 419 页零漂移、F 构成恒等。

### knife2b 锚豁免说明（防后续轮误读）

knife2b 锚（09:12–09:26，写者 10:04 抹除前树）已含 sweep 主增益
（effective-global-registry 18P/48F 在锚内），且自带 station 泄入形（optin
105P/13F）与 sanitizer sethtml-with-DSR 页 −1 瞬态（413P/343F）。故对已推 刀②
的增量归因以本表为准；对 knife2b 的增量仅 optin +4（泄入修复）。三码窗
（v3d `0dcd8dbb`→v3e `71bd51cc`→v3f `b5ff87b6`）逐级定位见 interference 面附录。

## 证据日志（同提交 add -f）

dc2-s10k2-wc-post-knife3.log（`fb75331587cca9c8c726ae6cc78852c9`）/
dc2-s10k2-domparsing-post-knife3.log（`492dc92343e774df4272e0e6a488e627`）/
dc2-s10k2-sanitizer-post-knife3.log（`2b319939d4ec51c4e9cc306e03b98e0b`）/
dc2-s10k2-dom-post-knife3.log（`65b400422e699ed5ca8eabbbabaae7f7`，毒化留档）/
dc2-s10k2-dom-post-knife3-r2.log（solo 判读域）/
锚系 dc2-s10k2-*-post-knife2b.log 四件（写者晨窗，同提交收档）。

## 残面（下刀挂账）

- effective-global-registry 残 48F＝adopt/append 矩阵深域（null-reg 父链 append
  断言族）——下刀首项。
- element-mutation 15F 未动；R90 trap scoped 原型臂 instanceof 残面、parse-time
  attr 区分（live 读恒 parse 面 3 案反向）、DSR shadowrootcustomelementregistry
  引擎 deep JSON、importNode/upgrade-order/criteria 页、IAH handle-only 插入复杂体
  ——沿 刀② 判读档序。

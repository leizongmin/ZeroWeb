# g47 分面定谳：`<position>` 严格文法 / `<bg-position>` 宽容文法分治 + 逻辑关键字 + canonical 序 + computed 折算

日期：2026-11-15（g48 提交 43535b6bf 之上工作树轮）
范围：crates/css-parser（parse_extended_visual.rs / serialize_position.rs / tests）、
crates/style-system（apply_advanced.rs / matcher/supports.rs）、crates/engine
（js_dom_bridge/computed_style.rs / paint/painter/effects.rs）。

## 1. 三源定谳（本轮裁决核心）

上一轮 g47 提交（88dd6b3ef）把「缘偏移对全有全无」放进了共享
`parse_background_position`，并翻转单测 oracle（`left 50px center` 判拒）——与
上游 bg 面语料和 R2478 reftest 双向冲突。本轮三源定谳分面：

1. **规范**：css-values-5 §4.2 通用 `<position>` 文法无 3 值形（spec 原文注记：
   3 值形仅 background-position 例外保留，通用文法因解析歧义移除）；缘偏移对
   全有全无；逻辑关键字 x-start/x-end/y-start/y-end 属 `<position>` 文法。
2. **语料**（wpt-data 上游同步，只读）：
   - `css/css-images/parsing/object-position-{valid,invalid,computed}.html`
     ——object-position = 严格 `<position>` 面。invalid 拒 13 形含
     `bottom 10%`（垂直缘前导偏移弃形）与 `right 3% center`（pair+裸混搭）；
     computed 钉 `right 20px bottom 10px` → `calc(100% - 20px) calc(100% - 10px)`。
   - `css/css-backgrounds/parsing/background-position-{valid,computed}.html`
     ——bg 面宽容：`left 10px center`、`right top 14%`、`bottom 16% left` 等
     混搭形 valid；computed 钉 `right 9%`→`100% 9%`、
     `top 17px right -18px`→`calc(100% + 18px) 17px`（负偏移加法形）。
   - `css/css-values/position/position-{valid,invalid,computed}.tentative.html`
     ——通用面：valid 17 逻辑关键字形（`x-start`→`x-start center`、
     `y-start 20px left 10px`→`left 10px y-start 20px`、`10px y-end` 原样）；
     invalid 钉 `bottom 10%`/`bottom 10% top 20%`；computed 钉
     `x-start 10px`→`0% 10px`、`x-end 10px top 20px`→`calc(100% - 10px) 20px`。
3. **reftest**：`css/css-backgrounds/background-position-three-four-values.html`
   （R2478）line 19 = `left 50px center` 上游原文，bg 面必须收 3 值混搭形。

裁决：**specified 层分面**（bg 宽容 / `<position>` 严格），**computed 层两面
共用同一折算模型**（关键字→%、left/top 缘偏移原样、right/bottom 缘 100−x、
负偏移加法形）；逻辑关键字仅严格面收。

## 2. 实现

- `parse_background_position` = `parse_position_impl` + 出口
  `position_has_logical` 滤除（bg 面拒逻辑关键字；EdgeOffset 的 side 域同查——
  首轮漏查致 `left 10px y-start 20%` 泄入 bg 面，单测逮住修复）。
- `parse_position_strict` = 3 值形入口拒 + all-or-nothing 出口裁 + impl 共享体
  （逻辑关键字收录）。垂直缘前导偏移（`y-end 10px` ≡ `bottom 10%`）两文法同拒
  （invalid 语料钉；`10px y-end` 偏移前置合法）。
- serialize_position.rs：canonical 化改走 strict 面；逻辑关键字 specified 保形
  （`x-start`→`x-start center`、`y-start`→`center y-start`）+ atom 8 边 side 臂。
- style-system apply：object-position 写门（apply + CSS.supports）切
  `parse_position_strict`；`position_value_to_computed` 逻辑缘物化
  XStart→Left/XEnd→Right/YStart→Top/YEnd→Bottom（FIXME(write-modes)：htb/ltr
  恒定映射，engine 无 vertical 写入模式布局——position-computed.tentative
  test_writing_modes 32 子测仅恒定映射写组可过）。
- engine computed_style.rs：`bg_position_axis_to_css` 负偏移折算修符号
  （`right -18px`→`calc(100% + 18px)`，此前工作树遗留 bug 给 `calc(100% - -18px)`）；
  单 EdgeOffset 顶层水平判定扩逻辑缘；防御臂穷举（编译期 E0004 守卫）。
- paint effects.rs：resolve_position_component EdgeOffset 逻辑缘防御臂
  （XStart/YStart→off、XEnd/YEnd→(container−image)−off；apply 期已物化正常不达）。
- 单测：tests_9 R2478 oracle 复原（`left 50px center` bg 面 Some + strict None
  pin）；advanced.rs strict 全有全无 8 拒形 + bg 面混搭 6 形 + 逻辑关键字分面
  （含 `y-end 10px` 双拒 pin、`10px y-end` strict 收 pin）；serialize 17 逻辑
  形语料 pin；engine g47 computed fold +负偏移加法形测试。

## 3. 验证（单元层）

- cargo test -p zero-css-parser -p zero-style-system -p zero-engine：
  css-parser **3237P/0F**（+R2478 oracle 复原、+strict/分面/逻辑新钉）、
  style-system **4018P/0F**、engine 全套 0F（含新负偏移 fold 测试）。
  守卫：systemd scope unit=zeroweb-g47-test-b3（Mem 24G），EXIT 0。
- cargo fmt --all --check clean；clippy 三 crate + engine(--features v8)
  `--all-targets -D warnings` 净（touch 强制复检 EXIT 0，4 Checking 行核实）。

## 4. 受控 A/B（B0 = results-g48-b1 合并树旧态；B1 = results-g47-b2 本轮分面树）

| corpus | B0 | B1 | 净 |
|---|---|---|---|
| css-images | （待填） | （待填） | （待填） |
| css-values | （待填） | （待填） | （待填） |

- 预期页族：object-position 三页维持/扩大翻绿；position tentative valid
  +8 逻辑形回归修复、invalid 维持；gradient computed 残 4F 维持（长度锚挂账）；
  css-backgrounds background-position 页组（bg 面恢复，单向账
  results-g47-b2-bgpos，无 A/B 基线）。
- 逐页归因：（corpus 后填）

## 5. reftest 面

- R2478 `background-position-three-four-values.html`：driving 文件上游原文
  未动，bg 面回宽容后应复绿（make reftest 守卫入口核）。（结果待填）

## 6. 残面挂账

- gradient computed 4F：`conic at right 10px` 需 `calc(100% - 10px)`——
  LengthValue 单值模型无缘锚表达，需锚定模型扩展（typed 通道）。
- position-computed.tentative test_writing_modes 32 子测：engine 无 vertical
  写入模式布局，逻辑缘恒 htb/ltr 映射（FIXME(write-modes)）。
- conic-calc-angle invalid/valid 8F（独立面）。

## 7. 运行记录

- 单测三轮（b1 分面漏检 1F→修 position_has_logical side 域；b2 测试断言
  `y-end 10px` 应拒修正→语料核对；b3 全绿）。守卫 systemd scope，
  /tmp/g47-unit-test-b{1,2,3}.log。
- corpus：results-g47-b2（css-images + css-values 全量）+
  results-g47-b2-bgpos（css-backgrounds 仅 background-position* 过滤面），
  systemd 单实例 flock + 同树守卫 + 唯一 transcript，test-guard 内存/墙钟限。
- 发射纪律：peer clone（ZeroWeb-3 dps C1 五腿门禁批）在飞期间等待不并发
  （伪红防护——case 外部墙钟在 CPU 竞争下会误判 Timeout）。

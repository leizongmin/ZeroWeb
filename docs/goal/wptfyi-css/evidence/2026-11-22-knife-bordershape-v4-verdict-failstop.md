# 刀 BORDERSHAPE v4 电池判定：静态恰集 32/35＋插值红线 P2F=127 fail-stop（2026-11-22）

电池＝unit `zw-bswait-bordershape4`，12:26:42 发射 → 12:38:20 EXIT rc=0。
patch v4 md5 `41e9f4c9e2ae069ece5a0c2f544fccb3`（v3 `c1aaaa2e` 配平扫描切片
panic 修复：`break i` 时 i 已停在 `)`＝close，切片 `&v[i+1..close]` 起点恒
大于终点；改捕获 `let open = i` 切 `&v[open+1..close]`）。双臂 bin：b0
`f51e6dbe41da5719d50bbbe0fda791ba`（HEAD 态，与 v3 b0 md5 恒等＝构建确定性）
/ b1 `2c0580d448c226b9583a4f0fc00301c9`。FACE-MARKER b0=0 b1=3 ✓。SMOKE 门
（新增：验证 build 后用 face 二进制全量跑 css-borders 腿）12:26:42 PASS
354421 字节零 panic——编译＋运行双验证首次入链即拦住 v3 类炸面。

## RECONCILE（键级，四语料 544 页全谱）

| 语料 | F2P | P2F | MSGDIFF | ASYM | 判读 |
|---|---|---|---|---|---|
| css-borders | 107 | **127** | 32 | 0 | 主战区，双向大位移 |
| css-masking | 0 | 0 | 0 | 0 | SET-IDENTICAL ✓ |
| css-shapes | 0 | 0 | 0 | 0 | SET-IDENTICAL ✓ |
| css-values | 1 | 0 | 0 | 0 | random-computed.tentative 1 案（random 时序域 flake 候，不入账） |

## 静态恰集（CLAIM 预注册 35＝invalid 12＋computed 22＋valid 1）

实收 **32/35**：invalid 12 ✓ 全翻；valid 1 ✓；computed 19/22，差 3 案＝
normalize **过度丢弃 geometry-box**：

- `circle() border-box` → got `circle()`（外组件 border-box 应保留）
- `circle() padding-box polygon(...)` → 内组件 padding-box 被丢
- `circle() border-box circle() border-box` → 双 shape+box 形被过度折叠

## 红线：P2F=127（全集中在插值两页）

- `border-shape-animation.html` **78**：CSS Transitions/Animations/Web
  Animations 三车道全灭——b0 Pass（HEAD 插值可用）→ b1 Fail，b1 值冻结在
  端点（expected `circle(10px at 10px 10px)` got `circle(30px at 30px 30px)`
  双读数同端点＝过渡不插值）。
- `corner-shape-interpolation.html` **49**：superellipse 插值退化（expected
  `superellipse(-2.5)` got `superellipse(0)`）。

### 机制定谳

HEAD 本就支持 border-shape/corner-shape 基础解析与插值（127 案 b0 Pass 为
证）。v4 face 在 **parse 期**将声明值 normalize 为 canonical 文本改写存储
形态；插值器按 HEAD 原文形态解析动画端点，canonical 形态失配 → 解析回退
→ 动画冻结端点/离散跳变。同页 75 F2P（插值页 b0 fail→b1 pass）＝normalize
恰好补齐 HEAD 插值器不认的形态——**75 风腾增益与 127 损失同源不可分**。

教训条目兑现：页级 gained/lost（battery 内置 RECONCILE 报 gained=59
lost=78）会掩盖页内 P2F（插值页 75F2P 净额吞 49P2F）；本判定以键级
judge（knife-bordershape-judge.py）为准，P2F>0 不问净额直接归因。

## 判定：FAIL-STOP

- **v4 face 不入账**，tally 不动；树已剥净（TRAP-NOOP state=done，face 文件
  对 HEAD 净）。电池产物 `results-knife-bordershape/` 留证；v3 panic 残迹
  在 `results-knife-bordershape-v3-panic/`。
- CLAIM 三页恰集 32/35 兑现度高（12+1 全翻＋19/22），**face 方向有效**。

## v5 处方（已布防）

normalize 收缩到 **echo 面**：gCS 桥（computed_style.rs）＋setter 回读
（shim/bridge）保留 canonical 串化；parse_extended_border.rs **只留拒收
校验＋view-box 关键词**，不再改写合法值存储文本。插值器读 HEAD 原文＝
P2F 结构性归零；静态 35 预期全兑（invalid 12 走拒收、computed 22 走 echo、
valid 1 走 setter 回读）。75 案插值增益挂独立深刀（插值器 canonical 形态
适配，涉 animation.rs 域＝D1-b/FAM-F/M4-b 三队列互斥域，需另行协调）。

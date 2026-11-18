# 残面侦查：transitions/animations/masking 三语料 ≥20F 页靶图（2026-11-18）

数据源：results-knife-a3b（刀 A3b r3c 八腿电池 b1 侧，13:02→13:26
GATE-DONE，R2 收窄形 bins）。覆盖 transitions 1250F / animations 781F /
masking 1704F，≥20F 页合计 1004+443+1540。逐族归域与占用态（对照当时
工作树 dirty 清单）：

## css-transitions（1250F）

| 族 | F | 域 | 占用态 |
|---|---|---|---|
| 事件系统（properties-value-inherit-001 560 + properties-value-003 122 + implicit-001 60 + inherit-003 60 + event-dispatch 30 + transitionevent-interface 23 + getComputedTiming 22） | ~857 | engine callbacks/css_wire + shim | 在飞件占用 |
| text-shadow-composition 55 + vertical-align-composition 24 | 79 | animation.rs composite add/accumulate | A3r 流占用 |
| transition-timing-function-invalid 25 + transition-behavior 23 | 48 | apply.rs VALIDATED_PROPERTIES 白名单门族 | apply.rs WIP 占用（同刀 R 文件） |

## css-animations（781F）

| 族 | F | 域 | 占用态 |
|---|---|---|---|
| animation-range（shorthand 111 + start-computed 30 + end-computed 29） | 170 | animation.rs + shim 解析（scroll-driven 新属性，registry 无条目） | A3r 流占用 |
| OM 族（getComputedTiming 41 + getKeyframes 32 + Element-getAnimations 21） | 94 | typed-om | 对端 64ffa88f0 侦查已认领 |
| keyframes-name-valid 39 + animation-shorthand 36 | 75 | shim 解析白名单门族 | shim 在飞件占用 |
| event-dispatch 32 + animationevent-interface 23 | 55 | 事件系统（同 transitions 鲸面） | 同上 |
| text-decoration-inset-auto 49 | 49 | 待分解 | 未占用待查 |

## css-masking（1704F）

| 族 | F | 域 | 占用态 |
|---|---|---|---|
| clip-path 插值/composition（shape 218 + geometry-box 144 + composition 129 + 002 84 + xywh-rect 84 + 001 62 + discrete 57） | ~778 | css-parser shape() 解析 + engine 插值 | fv1 占 css-parser |
| mask-border composition/插值（outset 100 + width 100 + slice 93 + width-interp 66 + source 42） | ~401 | composite 池 + mask-border 简写（刀 B1 同族残） | animation.rs 占用 |
| no-interpolation（mask 42 + clip-rule 42）+ mask-position 38 | 122 | registry/applier 离散臂 | animation.rs 占用 |
| parsing（clip-path-invalid 48 + mask-valid.sub 36） | 84 | apply.rs 白名单门族（刀 R 同族） | 同上 |

## 刀 R 靶面形态预研（apply.rs VALIDATED_PROPERTIES 59 项门，:2244/:2532）

白名单门族可执行面（树清后）：clip-path-invalid 48F（`auto`/`ray(0deg)`/
`inset()`/`inset(123)` 维度缺单位/`inset(1% 2% 3% 4% 5%)` 五值）+
shape-function-invalid 40F（shape() 尾逗号/逗号分隔参数/`hline to top`
类非法关键字）。同族随刀白拿候选：transition-timing-function-invalid 25 +
transition-behavior 23 + clip-invalid 4 + clip-rule-invalid 2 +
background-clip-invalid 5（=59F 附加）+ mask-valid.sub 36F（valid 门正
向形）。刀 R 既有口径 88F（48+40），扩面全族上限 ~183F。

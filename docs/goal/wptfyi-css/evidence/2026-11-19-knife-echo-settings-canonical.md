# 刀 echo：font settings 计算值串化规范形行为收账（2026-11-19）

## 目标

`7d119d8ef`（font settings 计算值串化规范形——隐式 1 省略 + 同 tag
右值胜出去重 + tag 字节序升序，css-fonts-4 §font-feature-settings-prop
serialization canonical 形）已随 fontcycle ffix 收口批先行落库，落库时
探针仅证 ffs/fvs computed 主靶预测。本刀为其行为面收账：A/B 电池补证
serialization 与 inheritance 顺带面，并核 animations/transitions 哨兵
零回归。

## 电池

- 脚本 `.acceptance/wptfyi-css/knife-echo-battery.sh`，log
  `knife-echo-battery-2234.log`，结果 `results-knife-echo/`。
- 22:34 发射，23:00:28 GATE-DONE，BATTERY-RC=0。
- face = `settings_computed_join` 规范形（HEAD 已含）：b0 = HEAD 剥除
  face（`git apply -R`），b1 = HEAD 重放；legs
  {fonts, animations, transitions} × {e0, e1}。
- b1 bin md5 `ef69612447d37188065d59860e6c14aa`；ride census S3
  窗内零扰动；M2 区分过。

## 账

`gained=13 lost=0 regressed=0 pages_gain=6 pages_loss=0`，全落 css-fonts：

| 页 | ΔP |
|---|---|
| parsing/font-feature-settings-computed.html | +6 |
| parsing/font-variation-settings-computed.html | +3 |
| font-feature-settings-serialization-001.html | +1 |
| font-variation-settings-serialization-001.html | +1 |
| font-variation-settings-serialization-002.html | +1 |
| inheritance.html | +1 |

- 主靶 ffs +6 / fvs +3 与落库前侦察预测精确兑现。
- serialization 三页（隐式 1 省略形）与 inheritance +1 为规范形顺带
  白拿，落库前未预测。
- 哨兵 animations/transitions 两腿逐页零差异（reconcile 输出零非
  fonts 页）＝computed 串化面变更不越界实证。

## 配套与残面

- `part17.rs` 钉测适配 39+/2− 在树（纯 echo 钉），待 part17 test 目标
  守卫跑过后 pathspec 提交。
- ffs/fvs declared 面的 settings math apply 臂残面归 fontcycle 刀
  （见 [knife-fontcycle-units](2026-11-19-knife-fontcycle-units.md)
  残面段）。
- 三性（font-kerning/font-variant-numeric/font-variant-alternates）
  离散域缺臂由刀 ffix2 接手（同窗在飞，见 NOTES-fontcycle-ffix-
  convergence.txt ffix2 段）。

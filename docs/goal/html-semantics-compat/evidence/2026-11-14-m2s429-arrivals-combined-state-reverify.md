# s429：五笔他流到货组合态整验（零源码改动轮）

日期：2026-11-14 ｜ 性质：组合态复验（零源码改动） ｜ 前序：s428 carve-out 施工 + 到货挂账

## 到货清单（sweep 窗口累计 5 笔，rebase 后本域树首验）

| commit | 车道 | 面 |
|---|---|---|
| ac2566689 | 渲染 | style-system font-weight 相对权重表（css-fonts-4 边界，font-weight-computed +8P） |
| 3df4f287a | 渲染 docs | wptfyi-css g32 归因挂账 |
| d1d418daf | editing | insertParagraph 拆分边界 inline 壳三族 P2F 修复 |
| bdbe581a6 | editing | strike REMOVE 方向判向 + u/s css-true 裸 span 吸收（engine part01d.js 等） |
| 65b06cbba | editing | insertParagraph 壳尾重开判据 v3.1 重定谳（s428 rebase 窗口新落） |

与本域四面（s428 carve-out）零文件交集；s428 sweep 树为 pre-arrival，本轮树
= HEAD 21d097d93（5 笔到货 + carve-out 组合态）。

## 方法与验证链

| 门 | 结果 |
|---|---|
| runner 重建自 HEAD（guarded） | exit 0，1m33s |
| 全域 sweep ×2（02:51–02:59） | 双轮 exit 1（套件含红正常），8458 键 ×2 满规模 |
| stable-set join vs s428 双基线 | 见下 |

## Join 结果

- stable-F **202=202，new=0 gone=0**
- stable-T **13=13，new=0 gone=0**
- stable-P 8241→8239：gone=2，两键均 `checkable-active-onblur-with-click.html`
  的 following-label 2 子测 → 转沾染桶
- 沾染：base=0 → cur=2（同上两键）；C-NEW 2 / C-GONE 0

## 判读

**5 笔到货组合态零稳定漂移**：三桶稳定集合逐键相同（F/T），唯一位移 =
在册 flake 键（s424 分诊在册）following-label 2 子测自 stable-P 移入沾染桶
——同二进制双轮异态（r1 2P / r2 2F）= 时序不稳定，非代码位移（代码级变化
对双轮确定性同翻）；失败形 `:active while the label is pressed expected
Element node but got null`（指针命中时序面）；到货 5 笔零输入/指针/激活路
径交集；parent-label 2F 在册稳定红全程未动。环境注记：sweep 窗口与
ZeroWeb-3 multitest 长跑（99.9% CPU，10800s 限）重叠，为指针时序闪变的
合理放大因子。

**口径**：稳定红 215（202F+13T）不变；沾染桶 0→2（在册 flake 家族，不新增
稳定红，红位待后续轮次复归时按桶内重排记账）。

## 基线移交与下一步

- **s430 join 基线改用 /tmp/s429/pairs-{1,2}.txt**（沾染 2 = 在册 flake 键
  单列；其若在 s430 双轮稳定复归会以 NEW-P/NEW-F 形式可见，属信息非噪音）。
- 下一步：**s430 engine act:option 臂施工**（s425 定型设计，双刀合龙
  seltouch 翻绿判据）。

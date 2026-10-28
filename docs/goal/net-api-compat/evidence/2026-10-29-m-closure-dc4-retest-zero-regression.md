# M 收口 — DC-4 coverage 复测终稿 + 一期零回归声明（2026-10-29）

**切片**: 二期收口。DC-1/DC-2/DC-3 已于 M1/M2 各轮落档，本档出 DC-4 终稿
定量与一期零回归声明，二期 Done Criteria 全部对账。

## DC-4 终稿定量（联合终态运行 `netapi-m2s3c-r4.json`，M1 归因档同口径）

| 族 | M1 基线（一期终态） | DC-4 目标 | 终态实测 | 判定 |
|---|---:|---:|---:|---|
| url | 1146F/6065 | ~360F | **169F**（5896/6065 = 97.2%）| ✓ 超额（余 13F url-setters 记账 + 28F USP BOM + 单案面）|
| mimesniff | 865F/1898 | 602F | **602F**（1296/1898 = 68.3%）| ✓ 精确命中（残余全部为已仲裁上游漂移 Blob.type 腿，不收敛记账）|
| fetch/api | 572F+1T/1928 | ~300F | **334F+1T**（1593/1928 = 82.6%）| ◑ 含 request-cache 族 98F 差 34F；剔除该 standing exclusion 为 236F 已达线——口径待决（见下） |

收敛轨迹：M1 基线通道 9354/12078 → 终态 **10843/12078 = 89.8%**（累计
**+1489**，S1 +270 / S2 +789 / S3 +160 / S3b +11 / S3c+S4a 联合 +258）。

## 一期零回归声明

- 各收敛轮 A/B（逐 subtest 对照前基线）**零真回归**：S1 +270 / S2 +789 /
  S3 +160 / S3b +11 / S3c+S4a 联合 +258（`abdiff-m2s3c-r4.txt`）。
- 过程轮暴露的真回归全部当轮清偿：S3 r1 28 legs、S3b 真回归 3 legs
  （xhr json UTF-16 / ws 016 / xhr CORS 同源重定向）、S4a 过程轮 ws 构造器
  fragment presence 1 leg（hash getter 空串面连带，`part06.js` href 判定
  修复）。
- 唯一 Pass→Fail 存留项：integrity「CORS invalid integrity」——**语义显影**
  非行为回归（基线靠 `Failed to fetch` 假绿，S3c `top.txt` 解锁后显影为 SRI
  真缺口，机械证据落 S3c 档记账 1，SRI 切片重排收敛）。
- 8 legs MISSING（S3b 页锚改名）逐条核对 Pass→Pass；联合终态 MISSING 0。

## 待决项（用户）

**request-cache 族 98F 口径**：DC-4 fetch/api ~300 目标按「HTTP 栈本体
standing exclusion」剔除口径已达标（236F）；含族口径差 34F，收敛需立项
HTTP 缓存层方案。挂用户拍板，不阻塞其余 Done Criteria 判定。

## 记账（承前汇总，详见各切片档）

- mimesniff 602F：上游语料互斥（FileAPI vs mime-types Blob.type 腿），一期
  已仲裁不收敛。
- url 169F：url-setters 13（xn-- ×2 / non-spec `/.` 序列化 ×2 / pathname 族
  ×9，见 S4a 档）+ USP BOM 28 + 单案面。
- fetch/api 334F+1T：request-cache 98F（待决）+ SRI 显影 1 + abort/general
  4 + request-upload 4 + h2 流块 3 + 余单案面。
- 全仓 coverage 矩阵重扫（`docs/compat/wptfyi/coverage-2026-10-27.md` 47
  套件干跑口径快照刷新）不在本 goal DC-4 口径（DC-4 按 M1 归因档通道分母
  定量），挂 runner 域排期。

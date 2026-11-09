# M0 甄别三：xhr 残余失败簇归因 + 枚举缺口分解

日期：2026-11-06。数据源：`make testharness-net-api FILTER=xhr/`（test-guard 包裹）
+ pin rev 全量树（xhr/ 444 blob）× 本地语料比对 + 缺失 case 内容甄别。

## 一、通道现状（M0 实测）

**91 文件实跑：586P / 7F / 1T**（98.7% 子测试通过率）。上游 den 2,333（wpt.fyi
aligned），通道实跑分母 ~594——缺口主体不是引擎失败，而是**枚举器形态缺口 + 语料
资源缺口 + 变体膨胀**三件套（下节分解）。

## 二、在账失败簇（7F + 1T 逐条归因）

| # | 案 | 症状 | 归因 | 批次 |
|---|---|---|---|---|
| F1 | formdata.html | FormDataEvent.formData 条目断言 expected "v1" got null | 引擎缺口：**FormDataEvent.formData 关联面**（formdata event interop） | M3 收敛 |
| F2 | formdata/constructor-formelement.html | unpaired surrogate 文件名 `U+dc01` 被 replaced character 形式序列化 | 引擎缺口：**表单 dataset 构造的文件名 surrogate 保真**（form nametochar 语义） | M3 收敛 |
| F3 | formdata/constructor-submitter-coordinate.html | Image Button submitter 坐标 expected "23" got "0" | 引擎缺口：**submitter 坐标 entry 构造**（image button .x/.y） | M3 收敛 |
| F4 | formdata/submitter-coordinate-value.html | dispatched event 坐标 expected "14" got "0" | 同上（submitter 坐标族） | M3 收敛 |
| F5 | overridemimetype-unsent-state-force-shiftjis.any.js | Shift-JIS 解码 expected "テスト" got "�e�X�g" | 引擎缺口：**overrideMimeType 强制编码在 unsent 态生效**（xhr 响应解码接 encoding 底座——encoding 37 号划界面，此处只收 xhr 侧接线） | M3 收敛 |
| F6 | sync-xhr-supported-by-permissions-policy.tentative.html | `document.permissionsPolicy.features` undefined | **Permissions-Policy API 面**（引擎未实现 permissionsPolicy） | **排除**（策略面，security-hardening 域；tentative 案） |
| F7 | xmlhttprequest-sync-default-permissions-policy.tentative.sub.html | `run_all_fp_tests_allow_all is not defined`（permissions-policy 支撑 helper 未拉） | 同上 + helper 依赖 | **排除**（同上） |
| T1 | cookies.http.html（Timeout） | harness 完成 0/1 报告（pending 1） | xhr Set-Cookie 时序面挂起——cookies 通道有同域在账，此处单案挂起归 channel 侧再查 | M3 收敛（单案） |

真实引擎缺口簇收敛对象 = **formdata/submitter 簇（F1-F4）+ overrideMimeType 编码接线
（F5）+ cookies 单案挂起（T1）**，共 6 项；F6/F7 策略面排除。

## 三、枚举缺口（未实跑面分解，328 本地 case 文件中 237 未跑）

| 形态 | 数量 | 例子 | 根因 | 批次 |
|---|---:|---|---|---|
| `.htm` 案 | ~175 | access-control-* preflight 族、事件序 .htm 老式案 | **通道枚举器只认 `.html`**（`run_any_js_corpus_subdirs_with_helpers_variants` 的 `file_name.ends_with(".html")`）——xhr corpus 190 个 `.htm` 是上游老式命名 | **M1a 批次（枚举器 .htm 解锁，最大杠杆）** |
| `.window.js` 案 | 9 | abort-after-stop / abort-during-* / open-after-stop | **枚举器不认 `.window.js`**（webmessaging/wasm-core 通道已有 `.window.js` 先例，net-api 共享枚举器缺此形态） | **M1a 批次（同上）** |
| manual 案 | 3 | send-authentication-prompt-* | skip 规则既有（用户交互面） | 排除（既定口径） |
| idlharness | 1 | idlharness.html | skip 规则既有（WebIDLParser build 期资产） | 排除（既定口径） |
| iframe 面 | 19 | sync-xhr-iframe-* 等 | skip 规则既有（runner 无多 frame 管道） | 排除（既定口径） |
| worker 面 | 13 | sync-xhr-worker.js 等 | skip 规则既有（worker 面排除） | 排除（既定口径） |

`.htm` 解锁面的内容预判：access-control-* preflight 族引用 corsca.py / preflight
端点（xhr fixture 已有 access-control-basic-allow-star.py、access-control-origin-header.py
等 v1 先例），预计**部分案直接可跑、部分需按 fixture 缺口分批补 shim**——批次实测后
按失败簇归因再定（不预先承诺通过率）。

## 四、语料资源缺口

| 资源 | 引用案数 | 甄别 | 批次 |
|---|---:|---|---|
| `resources/folder.txt` | 23 | 纯静态文件（目录列表测试载体）；**现 23 案在缺文件状态下跑且通过**（多数断言 error-state 语义，缺文件 = 网络错误同样满足），补拉后需 A/B 观察翻转 | M1b（pull + A/B） |
| `resources/auth1-11`（11 目录 ~14 文件 .py/.txt） | 8 实案 + 2 manual | **HTTP Basic auth 401 challenge 端点**（send-authentication-* 族）——fixture 需 401 + WWW-Authenticate + Authorization 回显语义，可行性中（ws_fixture /basic_auth 先例在 WS 侧） | M2（py shim 分批） |
| `resources/nocors/*` | 1（getresponseheader-error-state.htm） | nocors 跨源 error-state 面，单案 | 随 M1b 顺带 |

## 五、coverage 口径记账

上游 den 2,333 与通道实跑 ~594 的差 ≈ 1,739 中：变体膨胀（.any.js 的
?default/?wss 双变体、.window.js 变体 URL）+ `.htm` 未解锁面 + 排除面（manual/
iframe/worker/idlharness）。DC-4 复测时三面通过率分母逐 subtest 对照按 M0 甄别表
口径解释，不做 den 口径的无谓对齐承诺。

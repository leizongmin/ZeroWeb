# 决策包:M4 收口新发现 ⑤ media-host 沙箱 / ⑥ DevTools frontend token 限制

**日期**:2026-11-02　**拍板基线**:`97d344e57`
**拍板(2026-11-02,用户)**:⑤ → **A**(③A-media-host 最小档案);⑥ → **B**(文档化接受);同日 master 缺口表在册项 ①(renderer landlock file:// 取舍)→ **A**(browser 预解析 → LoadHtml)。拍板时补强核查:file:// 子资源已走 browser 侧 fetch_proxy(browser 侧直读),renderer 直读仅主文档+模块脚本,①A 改动面收敛于主文档读迁移 + 模块脚本改代理路径。
**性质**:设计选项细化(无代码变更)。两项均为 M4 收口/切片 5 交付期发现,原 2026-11-01 [决策包](2026-11-01-decision-package-1234.md) 范围外;细化到可拍板粒度供逐项确认。在册第三项待拍板(renderer landlock 默认值 file:// 三选项)粒度已够,见 [缺口表](master.md),不在此重复。

## 背景事实(代码查实,2026-11-02)

- **media-host**(`apps/media-host`):零沙箱钩子(main.rs/kernel.rs 无 seccomp/landlock/apply_early)。spawn 方 = renderer(`webview::media_proxy` 经 `Command`,与 image-decoder 的 `webview::image_decoder` 同机制、同目录部署)。IO 面:stdio IPC(PipeTransport 双向)+ CPU 解码;**网络 socket 零命中;`audio-cpal`(cpal 设备输出)为默认关 feature**(headless/CI 默认关,桌面音频输出启用时才有设备面);`fs::read` 仅测试 fixture helper。即默认构建 IO 面 ≈ image-decoder(纯 stdio + 内存)。
- **DevTools frontend page-direct**(切片 5 记档):ZW_DEVTOOLS_FRONTEND_DIR provision 的实例,Chrome stock frontend 经 `/devtools/inspector.html?ws=<addr>/devtools/page/<id>` 建 page 直连 socket——URL 不携带 token,强制 token 下升级被拒(-32001)。browser 级 socket 同受认证。**默认产品构建不 serve frontend**(provision opt-in 才受影响),影响面限 operator/开发场景。

## ⑤ media-host 沙箱

| 选项 | 内容 | 评价 |
|---|---|---|
| **A:③A-media-host 最小档案(推荐)** | 镜像 [切片 3](evidence/2026-11-01-m3-slice3-image-decoder-sandbox.md) 同构:seccomp 网络/exec 黑名单 + landlock 空 allowed-set(默认 IO 面零 fs 依赖);`audio-cpal` 启用时分档授予音频运行面(archive: `/dev/snd` rw 或 pulse 运行目录读,按 feature 检测分档)。④C 联动:新档案新语义 = fail-closed | 由 image-decoder 模板复制,风险最低;补齐 DC-4 最后一个 helper 面 |
| B:维持 fail-open 记风险 | [剩余风险报告](evidence/2026-11-02-m4-closing-retests-and-risk-report.md) 在册;攻击面限定 stdio+音频设备(默认构建无设备面) | 零工作量;DC-4 长期 PARTIAL,被攻破 media-host 可执行任意 syscall(含外联) |

验收形态:镜像切片 3——真实 restrict 单元回归 + 真实进程安装日志 + 受限操作子探针(fs/execve/socket 拒绝)+ 功能不回退(真实媒体消息族解码往返;音频 feature 腿按环境)。预估工作量 ≈ 切片 3(最小档案 + 分档音频面)。

## ⑥ DevTools frontend page-direct token 限制

| 选项 | 内容 | 评价 |
|---|---|---|
| A:ws 参数携带 token | per-tab frontend URL 的 `ws=` 值内追加 `?auth_token=`(如 `ws=addr/devtools/page/id%3Fauth_token=t`),server 端 `verify_upgrade_query` 容忍嵌套 `?`;需实测 stock frontend 对 ws 值内 `?` 的透传行为(Chrome 源码历史按 last-`?` 分割,倾向可行) | 工作量中(一天内含实测);恢复 provision 场景 frontend 完整可用 |
| **B:接受限制文档化(推荐)** | README/操作文档记「provision frontend 与强制 token 互斥;frontend 场景用自管 CDP 客户端或独立无 frontend 实例」;代码零变更 | frontend 为 operator 调试设施、非最终用户功能;等有真实需求再立项 A |
| C:`/devtools/page/` 路径豁免 token | page 直连免认证 | **不推荐**——绕过 ①a 强制 token,page socket 即整页控制面,安全回退 |

## 落地顺序建议(拍板后)

⑤ 与 ⑥ 互相独立,均可随时开工;⑤ 建议优先(补齐 DC-4 的最后 helper 面,goal 内收敛);file:// 三选项拍板后翻 landlock 默认(③A-renderer 收尾项)。

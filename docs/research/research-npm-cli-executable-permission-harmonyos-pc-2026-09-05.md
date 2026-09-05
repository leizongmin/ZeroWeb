# HarmonyOS PC 原生 CLI 分发：关键不是 chmod，而是 XPM 代码签名

> 调研日期：2026-09-05
> 场景：`npm install -g @zeroseed/cli` 后，`zs` / `zeroseed` 的最终 ELF 已为 `-rwxrwxr-x`，直接运行仍报 `Permission denied`；第三方 `loader zs` 可以运行。

> ⚠️ **勘误说明**：初版报告根据不完整信息，将问题优先归因于 npm 安装后执行位丢失，并推荐在 `postinstall` 中补 `chmod`。用户随后提供真机证据：最终 ELF 已有执行位。结合 HarmonyOS 官方安全白皮书、官方开发者 FAQ、OpenHarmony HNP/签名工具源码以及鸿蒙 PC 移植案例，实际主因应是 XPM 代码强制签名校验。`chmod` 只能解决 DAC mode bit，不能解决未签名 ELF 被内核拒绝执行。

> ⚠️ **二次勘误说明（真机验证）**：`@zeroseed/cli-openharmony-arm64@0.0.2-harmonyos-test.1` 经官方 Java `binary-sign-tool` 自签后不再报 `Permission denied`，但直接执行以 139（`SIGSEGV`）退出，第三方 `loader` 仍可运行。签名前后 ELF 对比确认，官方工具把可写 `PT_LOAD` 的文件偏移从 `0x7577c8` 改成 `0x756e48`，却保留虚拟地址 `0xb677c8`，破坏了 4 KiB 页下的加载映射同余关系。社区 `codex-harmonyos` 使用的 append-only `ohos-bst-light` 签名器不会修改 program headers；同一 ZeroSeed 原始 ELF 经该工具签名后也可通过官方 `display-sign` 验签。因此，“任意 ELF 优先使用官方 Java 工具”并不成立，必须按 ELF 来源选择签名器并执行签名后结构校验。

## 来源分级

| 分级 | 本文使用方式 |
|---|---|
| 官方事实 | 华为安全白皮书、华为开发者 FAQ、OpenHarmony HNP 与 hapsigner 源码 |
| 一手发布物 | ZeroSeed、BitFun、esbuild、Rollup 的 npm 包内容及 ELF section 检查 |
| 工程实践 | Electron、Termony、OHcode、Harmonybrew、Codex/DeepSeek 鸿蒙移植项目 |
| ⚠️ 待真机验证 | 预签名 ZeroSeed ELF 能否在目标系统版本和 HiShell 域直接运行 |

## 30 秒速览

- `rwx` 只通过了传统文件权限检查；HarmonyOS NEXT 还会在代码加载阶段验证签名，未签名 ELF 不会获得可执行内存权限。[HarmonyOS NEXT 安全技术白皮书](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/privacy/privary-new/250610/down/HarmonyOS%20NEXT%E5%AE%89%E5%85%A8%E6%8A%80%E6%9C%AF%E7%99%BD%E7%9A%AE%E4%B9%A6.pdf)
- 鸿蒙 PC 的 Electron 实践记录表明，PC 25 镜像以后，未签名 ELF 会被 XPM 拦截，并产生 `get signature info failed` 日志。[Electron HNP 指南](https://gitcode.com/CPF-Electron/Electron/blob/main/docs/hnp-packaging-guide/README.md)
- 华为官方 FAQ 给出的允许路径是：二进制必须签名，并在系统中开启“运行来自非应用市场的扩展程序”。插件来源本身不受限制。[华为开发者 FAQ](https://developer.huawei.com/consumer/cn/doc/doccenter-dev-faq/faqs-access-control-23)
- `loader` 属于 Termony 等项目的 ELF loader，利用调试应用/JIT 权限加载代码，不是 HarmonyOS PC 标准命令，不能作为 npm CLI 的公共契约。[Termony](https://github.com/TermonyHQ/Termony)
- 官方生产级方案是 HNP：原生软件包通过 `hnpcli` 打成 HNP、嵌入 HAP、随 HAP 一起签名和安装；public HNP 的命令可进入公共 PATH。[OpenHarmony Native 软件包指南](https://gitcode.com/openharmony/startup_appspawn/blob/master/service/hnp/README_zh.md)
- 若必须保留 `npm install -g` 体验，最现实的候选是发布前对 OpenHarmony ELF 预签名；`postinstall` 现场自签仅适合作为检测到 DevBox/SDK 时的兜底。

## 1. 根因：XPM 代码签名，而不是 Unix 执行位

HarmonyOS NEXT 官方白皮书明确描述了代码强制签名：应用安装时生成签名树信息，代码加载和申请执行权限时由系统验证签名，只有合法签名软件才获得可执行权限。系统同时采用强制访问控制与 syscall filtering，因此 POSIX mode 并不是最终授权依据。

这解释了当前现象：

```text
-rwxrwxr-x  zs-target
zs                 -> Permission denied
loader zs          -> 可以运行
```

`chmod` 已经满足 DAC，但直接 `execve` 在映射 ELF 代码段时仍可能被 XPM 拒绝。第三方 loader 走的是另一条加载路径，并不能证明该 ELF 已通过系统原生执行策略。

可用系统日志确认：

```sh
hilog | grep -E 'xpm|signature|unsigned file|get signature info failed'
```

典型 XPM 日志包含：

```text
event_type: get signature info failed
code_type: ELF
```

如果能看到这类记录，就可以把 `noexec`、npm symlink 和 mode bit 从主因中排除。

> **来源说明（第 1 章）**
>
> - **官方事实**：安全白皮书明确规定代码强制签名；华为 FAQ 明确要求插件 ELF 具备签名。
> - **工程实测**：Electron HNP 指南给出了 PC 镜像上的 XPM 拦截日志。
> - **💡 推理**：尚未取得本次 ZeroSeed 失败时的 hilog；XPM 是高度吻合的根因，但仍建议用日志闭环。

## 2. 网上已有的实际解决路径

### 2.1 官方方案：HNP + 签名 HAP

OpenHarmony 为 Python、Node、Java 等生产力 Native 软件包提供了 HNP（OpenHarmony Native Package）机制：

1. 使用 SDK 中的 `hnpcli` 把原生软件目录打成 `.hnp`；
2. 把 HNP 嵌入 HAP；
3. 对 HAP 进行签名后安装。

public HNP 安装到 `/data/service/hnp`，可被所有应用访问；private HNP 安装到 `/data/app`，只允许所属 HAP 使用。对应的 bin 路径会进入执行环境的 PATH。[HNP 官方源码文档](https://gitcode.com/openharmony/startup_appspawn/tree/master/service/hnp)

已有实践：

- Termony 把 bash、busybox 等做成 HNP，并可在 HiShell 使用；其 loader 只是额外的开发能力。[Termony](https://github.com/TermonyHQ/Termony)
- OHcode 用 HNP 分发 Node、bash、ripgrep、Electron。[OHcode](https://github.com/HanversionOvO/OHcode)
- 鸿蒙版 Electron 明确把 HNP 作为解决 XPM 拦截的系统方案。[Electron HNP 指南](https://gitcode.com/CPF-Electron/Electron/blob/main/docs/hnp-packaging-guide/README.md)

结论：这是最符合平台安全模型、最适合正式产品发布的路径，但交付物必须是签名 HAP，不能只靠 npm registry 完成安装。

### 2.2 保留 npm 流程：发布前给 ELF 预签名

OpenHarmony 官方 `developtools_hapsigner` 提供 `binary-sign-tool`，支持标准 ELF 可执行文件和 `.so`，并支持证书签名与 self-sign：[OpenHarmony hapsigner](https://gitcode.com/openharmony/developtools_hapsigner/tree/master)

```sh
binary-sign-tool sign \
  -inFile zeroseed \
  -outFile zeroseed-signed \
  -selfSign 1

binary-sign-tool display-sign -inFile zeroseed-signed
```

华为官方 FAQ 同时说明：扩展程序来源没有限制，但其中的二进制必须签名；IDE/弱沙箱场景还需用户开启：

```text
设置 → 隐私和安全 → 运行来自非应用市场的扩展程序
```

因此可在 ZeroSeed release pipeline 中：

```text
Rust build → strip（若需要）→ binary-sign-tool → 校验 .codesign → npm pack/publish
```

签名必须是最后一个修改 ELF 内容的步骤；签名后不得再 strip、patch 或写入 section。npm 解包和 chmod 不改变文件内容，因此通常不会破坏签名。

这个方向已有可复查实践：

- `codex-harmonyos` 下载原生 tarball后为 Codex、helper 和 ripgrep 注入 `.codesign`，再建立 PATH 链接，报告在鸿蒙 PC 真机运行成功。[codex-harmonyos](https://github.com/QinpanWan/codex-harmonyos)
- 社区使用官方 `binary-sign-tool -selfSign 1` 运行 busybox、tree、less、aria2；同时报告 self-sign 对 JIT、终端控制和动态库符号链接存在版本相关限制。[鸿蒙 PC ELF 签名实测](https://hu60.net/q.php/bbs.topic.107186.html) [aria2-harmonyos](https://github.com/HanversionOvO/aria2-harmonyos)
- Harmonybrew 要求开启开发者选项和“运行来自非应用市场的扩展程序”，然后以包管理器方式分发原生工具。[Harmonybrew](https://harmonybrew.atomgit.com/)

限制：self-sign 更接近开发者/侧载模式。面向普通消费者且不希望用户打开安全开关时，应回到签名 HAP/HNP 的正式渠道。

### 2.3 安装后现场自签：可自动化，但不是零前置

如果不方便在 CI 预签，可在 `postinstall` 检测 `binary-sign-tool`：

```sh
binary-sign-tool sign -inFile zeroseed -outFile zeroseed -selfSign 1
```

HarmonyOS PC 上通常需要先从应用市场安装 DevBox 才有该命令；官方工具也可来自 OpenHarmony SDK。这个方案可以封进 npm 安装脚本，但必须满足：

- `process.platform === "openharmony"` 时才执行；
- 签名前验证目标确为预期 ELF，签名后用 `display-sign` 验证；
- 不把签名失败静默吞掉，而是给出 DevBox/安全开关的明确指引；
- 避免对已有 `.codesign` section 重复签名；升级覆盖后重新签名；
- 所有随附 `.so` 也要签名。

它仍不具备完全普适性，因为 npm 无法凭空提供系统信任能力，用户至少需要 DevBox/SDK 和系统安全开关。

> **来源说明（第 2 章）**
>
> - **官方事实**：HNP 分发流程、binary-sign-tool、自签命令、华为扩展程序 FAQ。
> - **工程实践**：Termony、OHcode、Electron、Harmonybrew、Codex 与 aria2 的真机路径。
> - **⚠️ 限制**：社区对 self-sign 的 JIT/TTY/动态库限制存在系统版本差异，ZeroSeed 需要独立验收。

## 3. npm 生态样本是否已经解决了这个问题

### `@bitfun-test/bitfun-cli`

该包是目前最接近 ZeroSeed 的样本：顶层 JavaScript launcher 根据 `process.platform === "openharmony"` 选择 `aarch64-unknown-linux-ohos` 平台包，然后用 Node `spawn()` 直接启动原生 ELF。[BitFun npm 包](https://www.npmjs.com/package/@bitfun-test/bitfun-cli)

但其公开包中没有 HNP、签名步骤或 loader 逻辑。它证明 npm 可以完成 OpenHarmony 平台选择，却不能证明在开启 XPM 的鸿蒙 PC 上直接 spawn 未签名 ELF 是可靠的。公开资料中也没有找到对应的鸿蒙 PC 签名验收说明。

### Rollup、Oxc、Rolldown 等 `*-openharmony-arm64`

这些包多数是 `.node` N-API binding，不是用户直接执行的 CLI。抽查 `@rollup/rollup-openharmony-arm64@4.63.1`，产物是 stripped AArch64 shared object，未发现 `.codesign` section。[Rollup OpenHarmony 包](https://www.npmjs.com/package/@rollup/rollup-openharmony-arm64)

它们解决的是“Node addon 针对 OpenHarmony ABI 构建和选择”的问题，不等同于解决 HarmonyOS PC 的 XPM/弱沙箱分发问题。

### esbuild 的 OpenHarmony 包

`@esbuild/openharmony-arm64@0.28.2` 没有直接执行原生 ELF，而是用带 Node shebang 的 JavaScript 启动器加载 `esbuild.wasm`，包描述也明确称其为 WebAssembly shim。[esbuild OpenHarmony 包](https://www.npmjs.com/package/@esbuild/openharmony-arm64)

这是唯一真正绕开 ELF 代码签名边界的 npm 设计模式，但 ZeroSeed 依赖文件系统、网络、TTY、子进程和本地运行时能力，迁移到 WASI/WASM 的成本及能力缺口远大于给现有 ELF 签名。

### 纯 JavaScript CLI

像 DeepSeek Harness 这类工具的主入口可以由已经获准运行的 Node 执行，但其原生依赖仍会遇到相同限制。鸿蒙移植实践需要通过 Harmonybrew 安装已适配工具、重新构建 native addon，并避开被 strip 后无法加载的 unsigned ELF。[deepseek-harness-harmonyos](https://github.com/shd101wyy/deepseek-harness-harmonyos)

结论：npm 生态里暂未找到“普通 native CLI 仅靠 package.json 配置就普适运行于鸿蒙 PC”的成熟范例。成功案例最终都落到以下之一：

1. HNP/HAP 安装并签名；
2. 对 ELF 自签名；
3. 使用 Harmonybrew 等已适配分发系统；
4. 改为 JS/WASM，不再直接 exec ELF；
5. 使用 Termony loader/QEMU 等特定运行环境。

> **来源说明（第 3 章）**
>
> - **一手发布物**：抽查 BitFun、Rollup 与 esbuild npm tarball；Rollup ELF section 未发现 `.codesign`，esbuild 内容为 JS + WASM。
> - **工程实践**：DeepSeek 鸿蒙移植对 Node native addon、签名和 Harmonybrew 的记录。
> - **💡 推理**：未找到公开验收不表示 BitFun 一定不能运行，只表示它不能作为“已经解决 XPM”的证据。

## 4. 对 ZeroSeed 的推荐方案

| 目标 | 推荐方案 | 用户前置 | npm-only | 适用性 |
|---|---|---|---:|---|
| 开发者现在能安装 | OpenHarmony 平台 ELF 在 CI 预先 self-sign | 鸿蒙 6、开发者模式、安全开关 | 是 | **优先验证** |
| npm 安装时自动补救 | `postinstall` 调官方 `binary-sign-tool` | DevBox/SDK、安全开关 | 是 | 兜底 |
| 面向普通用户正式发布 | public HNP 嵌入签名 HAP | 安装签名应用 | 否 | **正式方案** |
| 完全无本地签名/安全开关 | JS/WASM 重构 | 已获准运行的 Node | 是 | 成本很高、能力受限 |
| `loader zs` | 第三方 loader | 特定终端/调试应用 | 是 | 不作为产品方案 |

### 推荐的验证顺序

1. 在现有鸿蒙 PC 上确认 XPM 日志。
2. 不改代码，先手工签名当前 npm 安装得到的最终 ELF：

   ```sh
   ZS_REAL="$(readlink -f "$(command -v zs)")"
   binary-sign-tool display-sign -inFile "$ZS_REAL"
   binary-sign-tool sign -inFile "$ZS_REAL" -outFile "$ZS_REAL.signed" -selfSign 1
   chmod 775 "$ZS_REAL.signed"
   "$ZS_REAL.signed" --version
   ```

3. 若签名版直接运行成功，把签名移到 release pipeline，在 npm tarball 中直接携带 signed ELF。
4. 用至少两台鸿蒙 PC 验证同一预签名 artifact，确认系统版本、安全开关、首次授权弹窗和跨设备行为。
5. 验收 ZeroSeed 的交互 TTY、Ctrl-C、网络、文件读写、子进程及所有动态库；不能只测 `--version`。
6. 若目标用户不能接受开发者模式/安全开关，停止优化 npm-only 路径，转为 public HNP + 签名 HAP。

### 发布流水线注意事项

- 继续保留未签名 ELF 的 SHA-256，签名后再生成最终发布 SHA-256；两者不能混用。
- `strip` 必须发生在签名前；签名后任何字节修改都可能使 XPM 校验失败。
- 发布测试必须执行 `binary-sign-tool display-sign`，并检查 `.codesign` section。
- 若 ZeroSeed 仍复用 `aarch64-unknown-linux-musl` artifact，签名只能解决 XPM；ABI、syscall、证书路径等兼容性仍需真机覆盖。
- 如果平台包包含 `.so`，必须逐个签名，并避免安装过程再次修改它们。
- 不建议把第三方逆向签名实现直接塞进 npm `postinstall`；优先使用官方工具，除非完成许可证、安全和算法兼容审查。

## 5. Linux 本机签名与 ZeroSeed npm 发布落点

### 5.1 `binary-sign-tool` 能否在当前 Linux 执行

可以。二进制签名工具文档明确提供三种宿主形态：Linux 原生工具、HarmonyOS PC/2in1 的 DevBox 工具，以及 JDK 8+ 可运行的 `binary-sign-tool.jar`。Linux 原生程序和 Java JAR 都位于 Command Line Tools 的 `openHarmony/toolchains/lib`。[二进制签名工具文档](https://developer.harmonyos.cool/docs/tools/cli-tools/binary-sign-tool/)

对当前工作机的只读检查结果是：

```text
宿主：x86_64 Linux
已有：node、python3、readelf
缺少：binary-sign-tool、binary-sign-tool.jar、java
```

所以准确结论是：**当前 Linux 平台支持在本机签名，但当前环境尚未安装官方签名工具；下载 Linux 版 Command Line Tools 即可走原生工具，或再安装 JDK 8+ 后走 JAR。** 不应把面向 OHOS/ARM64 目标机的可执行文件误当成 Linux host 工具；应使用 SDK `linux/toolchains/lib` 下与宿主匹配的版本。

自签名的最小发布命令为：

```sh
OHOS_SDK="${OHOS_SDK:?set OHOS_SDK to the Linux SDK directory}"
SIGN_TOOL="$OHOS_SDK/toolchains/lib/binary-sign-tool"

"$SIGN_TOOL" sign \
  -inFile artifacts/aarch64-unknown-linux-musl/zeroseed \
  -outFile artifacts/aarch64-unknown-linux-musl/zeroseed.signed \
  -selfSign 1

"$SIGN_TOOL" display-sign \
  -inFile artifacts/aarch64-unknown-linux-musl/zeroseed.signed
readelf -SW artifacts/aarch64-unknown-linux-musl/zeroseed.signed | grep -F .codesign
```

JAR 版本等价命令：

```sh
java -jar "$OHOS_SDK/toolchains/lib/binary-sign-tool.jar" sign \
  -inFile unsigned-elf \
  -outFile signed-elf \
  -selfSign 1
```

OpenHarmony 官方 hapsigner 源码文档同时给出了 `sign`、`display-sign`、`-selfSign 1` 及证书签名的完整参数，并说明 Java 版本要求 Java 8+。[OpenHarmony hapsigner](https://gitcode.com/openharmony/developtools_hapsigner/tree/master)

### 5.2 应该插入 ZeroSeed 现有发布流程的哪里

ZeroSeed 当前对 OpenHarmony 平台包复用 `aarch64-unknown-linux-musl` artifact，`package-npm.mjs` 再复制到 `@zeroseed/cli-openharmony-arm64`。当前 release workflow 在归档构建产物时已经生成 SHA-256，assemble 阶段会再次校验该摘要。因此不能直接改共享 musl artifact，否则会同时改变 Linux musl 包且使原摘要失效。

最小且边界清晰的接入点是 `stagePlatformPackage()`：

```text
读取已校验的共享 musl artifact
  → 复制到 OpenHarmony npm staging 目录
  → 仅当 target.os === "openharmony" 时签名 staging 副本
  → display-sign + .codesign 校验
  → 对签名后的副本计算 npm manifest SHA-256/size
  → npm pack
  → 解包复核 tarball 内 ELF 哈希
  → 先发布平台包，再发布 @zeroseed/cli
```

这样 Linux musl 包仍发布原始 ELF，OpenHarmony 包发布 signed ELF，也不需要让两个逻辑平台共享同一个最终摘要。签名必须发生在 `chmod`、`strip`、section 修改等所有 ELF 内容处理之后；签名后只允许字节不变的复制和打包。

本机手工发布所需的前置条件是：

1. Linux 版 Command Line Tools，或 JDK 8+ 与 `binary-sign-tool.jar`；
2. npm 登录凭据及发布权限；
3. 与仓库版本一致的完整跨平台 artifacts；
4. 对 OpenHarmony staging 副本完成签名及验签；
5. 跑现有 `package-npm.mjs`、preflight，再按“平台包优先、入口包最后”的既有 `publish-npm.mjs` 顺序发布。

注意：`selfSign` 仍要求鸿蒙 PC 开启开发者模式和“运行来自非应用市场的扩展程序”。如果目标是普通消费者无需安全开关，应改用 HNP + 正式签名 HAP，而不是把 npm 自签流程继续包装。

## 6. `codex-harmonyos` 的真实实现与“发布流程”

`codex-harmonyos` 没有构建 Codex，也没有向 npm 发布一个二次打包后的鸿蒙包。它是一个 MIT 许可的纯文本安装器仓库；其所谓“发布”是把安装脚本发布到 GitHub，由用户在鸿蒙 PC 上运行，安装器再消费 OpenAI 已发布的 npm tarball。[codex-harmonyos](https://github.com/QinpanWan/codex-harmonyos)

其完整运行链路如下：

1. `codex-install.sh` 查找 Node 和 Python；找不到 PATH 中的命令时，回退到 DevEco/鸿蒙环境的固定工具位置。
2. `resolveTarball()` 查询 npm registry 的 `@openai/codex` metadata。指定普通版本号时追加 `-linux-arm64`；未指定版本时读取 `linux-arm64` dist-tag。
3. tarball 下载到 `~/.codex-hm/`，而不是 `/tmp`，再用系统 `tar` 解压。
4. 对三个 ELF 逐个签名：`codex`、`codex-code-mode-host`、内置 `rg`。
5. 签名不是调用官方 `binary-sign-tool`，而是 vendored 的 `tools/self-sign.py`：它在 ELF64 尾部增加 4 KiB 对齐的 `.codesign` section，计算 SHA-256 Merkle root，写入 fs-verity descriptor 和 `FLAG_SELF_SIGN`。
6. 建立 `~/.local/bin/codex` 软链接，并向已有 `~/.codex/config.toml` 幂等追加 DeepSeek provider 配置。
7. `verify` 先运行 `codex --version`，再在有 API key 时发送一次真实模型请求作为探活。
8. `update` 先备份当前三个已签 ELF，再下载新 tarball 并重新签名；`rollback` 从最近备份恢复。
9. 因 HMFS 会密封已经执行的 ELF，回滚不是直接覆写，而是先写 `.restore` 临时文件，再用 rename 替换目标。

它的签名脚本只支持带 section header table 的 ELF64；发现已有 `.codesign` 会拒绝重复签名。安装器之所以每次升级重新解压全新 tarball，就是为了从未签名原件开始。项目声称产物与官方工具“段级等价”，但这是项目自己的兼容性声明，不是华为官方背书。

本次在当前 Linux 上对其 `self-sign.py` 做了独立烟测：以 `/bin/true` 副本为输入，脚本成功生成签名副本，`readelf -SW` 可见 4 KiB、4 KiB 对齐的 `.codesign` section。该结果只证明脚本能在当前 Linux 修改 ELF；最终是否被目标鸿蒙版本接受，仍必须在鸿蒙 PC 上直接 `execve` 验收。

### 可借鉴与不可直接照搬

可借鉴：纯文本签名工具便于离线安装；签所有 helper；每次升级从干净原件重签；签名后验证；用临时文件 + rename 处理 HMFS 密封。

不可直接照搬：它没有 npm publish、provenance、tarball 内哈希复核，也没有正式证书/HNP 流程；签名发生在用户设备，而不是发布机；默认修改模型 provider；其 Python 实现来自第三方项目而非官方 SDK。ZeroSeed 若采用它，应固定已审计的上游版本、校验源码摘要、增加签名后 program-header 门禁，并在发布机签名后通过 npm 分发。

### 6.1 为什么 `codex-harmonyos` 能运行，而当前 ZeroSeed 测试包会崩溃

此前文档所指的 [QinpanWan/codex-harmonyos](https://github.com/QinpanWan/codex-harmonyos) 并不是 OHOS 原生编译版。它下载 OpenAI 官方 `aarch64-unknown-linux-musl` tarball，再调用 vendored `tools/self-sign.py`。该脚本来自 `ohos-bst-light`，其关键行为是 append-only：

1. 找到原 ELF 所有 section 和 section-header table 的末尾；
2. 在文件尾部按 4096 字节对齐追加 `.codesign`；
3. 把更新后的 `.shstrtab` 和 section-header table 放到更后面；
4. 只修改 ELF header 中的 section-table 位置和数量，不重新布局任何 `PT_LOAD`、TLS 或 RELRO segment；
5. 写入 fs-verity descriptor、Merkle root 和 `FLAG_SELF_SIGN`。

对同一个 ZeroSeed 0.0.2 未签名 ELF 的本机复测结果：

| 项目 | 未签名原件 | 官方 Java 工具 | `ohos-bst-light`/Codex 脚本 |
|---|---:|---:|---:|
| 可写 `PT_LOAD.p_offset` | `0x7577c8` | `0x756e48` | `0x7577c8` |
| `PT_LOAD.p_vaddr` | `0xb677c8` | `0xb677c8` | `0xb677c8` |
| program headers 是否保持 | 基准 | 否 | 是 |
| 官方 `display-sign` | 未签名 | 通过 | 通过 |
| 鸿蒙 PC 直接执行 | `Permission denied` | `SIGSEGV`（139） | 待真机验证 |

目标鸿蒙 PC 页大小为 4096。官方工具产物中 `0x756e48 % 4096 = 0xe48`，而 `0xb677c8 % 4096 = 0x7c8`，二者不同；这可以直接解释系统原生加载后在启动阶段崩溃。`ohos-bst-light` 产物保持原 program headers，因此没有引入这一结构缺陷。官方工具签名时同时输出 `.tdata`、`.tbss`、`.got`、`.data` 和多个 segment changed 警告，但仍返回 `sign success`，说明仅检查退出码和 `display-sign success` 不足以作为发布门禁。

社区也在 2026-07-27 记录了官方 `binary-sign-tool` 会导致 Bun 预编译 ELF 无法正常工作，建议改用 `ohos-bst-light` 的轻量签名实现。这与 ZeroSeed 的真机症状和结构对比一致。[鸿蒙 PC ELF 签名实测](https://hu60.net/q.php/bbs.topic.107186.html) [ohos-bst-light](https://github.com/hqzing/ohos-bst-light)

### 6.2 另一个同名方向：真正的 OHOS 原生 Codex npm 包

2026 年 7 月发布的 `@ohos-ports/codex@0.140.0-beta.0` 是另一条路线，不应与 `QinpanWan/codex-harmonyos` 混淆。它的 npm 包只是 JavaScript wrapper，依赖 GitCode 上的定制 `@openai/codex` tarball；后者包含从源码使用 OHOS SDK clang 编译的 `vendor/aarch64-unknown-linux-ohos/bin/codex`。[npm 发布物](https://www.npmjs.com/package/@ohos-ports/codex) [OpenHarmony PC Developer](https://gitcode.com/OpenHarmonyPCDeveloper/JavaScript_Package_For_HarmonyOS)

该原生产物的直接证据包括：

- 动态解释器是 `/lib/ld-musl-aarch64.so.1`；
- 依赖 `libtime_service_ndk.so` 和 `libc.so`，并随包携带 OpenSSL 1.1 动态库；
- program headers 有四个布局合规的 `PT_LOAD`，而不是复用上游 Linux 静态二进制；
- `.codesign` 可被官方 `display-sign` 验证；
- 项目归档的真机报告记录 `node bin/codex.js --version`、`--help` 和 native spawn 均通过；
- 为适配 OHOS，项目修改了 `nix`、TLS、V8、network proxy 等依赖和条件编译，不是只做签名。

这条路线进一步说明：长期正确方案仍是 `aarch64-unknown-linux-ohos` 源码构建后使用官方签名工具；短期复用 Linux-musl 产物时，`codex-harmonyos` 成功的关键不是系统设置，而是它使用不会重排 `PT_LOAD` 的轻量 append-only 签名器。

## 7. 证据矩阵

| 关键结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|---|---|---|---|---|
| `rwx` 后仍 EACCES 可由代码签名策略导致 | 华为安全白皮书 | Electron PC 25 XPM 日志 | 一致 | 高 |
| 第三方扩展 ELF 必须签名 | 华为官方 FAQ | OpenHarmony hapsigner 文档 | 一致 | 高 |
| HNP 是平台提供的 Native 软件分发方式 | startup_appspawn HNP 文档 | Termony/OHcode 实现 | 一致 | 高 |
| `loader` 不是系统通用能力 | Termony 自述 elf-loader | HNP 官方方案不依赖 loader | 一致 | 高 |
| npm 可分发 OpenHarmony artifact，但不会自动建立系统信任 | BitFun/Rollup npm 包 | 华为签名要求 | 一致 | 高 |
| JS + WASM 可绕开直接 exec ELF | esbuild npm 发布物 | 其 package description/launcher | 一致 | 高 |
| CI 预签名可保持 npm 安装体验 | 官方签名工具可对 ELF 签名 | Codex/aria2 社区实测 | 支持 | 中高 |
| 同一 self-signed ELF 可跨所有鸿蒙 PC | 官方 FAQ 未限制来源 | 社区口径有过变化 | 尚需 ZeroSeed 双机验证 | 中 |
| Linux 可在发布阶段签 ELF | 二进制签名工具文档列出 Linux host 工具 | hapsigner 官方源码提供 Java JAR | 一致 | 高 |
| codex-harmonyos 不发布二次 npm 包 | 仓库仅含安装器/签名脚本 | 安装器直接请求 npm registry 上游 tarball | 一致 | 高 |
| Codex 脚本签名器不会重排 `PT_LOAD` | `tools/self-sign.py` 源码 | ZeroSeed 签名前后 `readelf -lW` 实测 | 一致 | 高 |
| 官方 Java 工具会破坏当前 ZeroSeed ELF 布局 | 签名日志的 section/segment changed 警告 | 真机退出 139 与 program-header 对比 | 一致 | 高 |
| OHOS 原生 Codex 是源码移植而非 Linux 包改名 | GitCode 迁移补丁与测试报告 | 发布 ELF 的解释器、依赖和 target 目录 | 一致 | 高 |

## 8. 最终判断

目前最接近“普适解”的不是 loader，也不是 npm 权限配置，而是 **HarmonyOS 的 ELF 代码签名**。

对于 ZeroSeed，可以先把 OpenHarmony artifact 在发布阶段 self-sign，继续通过 npm 分发，并明确要求鸿蒙 PC 用户开启开发者模式和“运行来自非应用市场的扩展程序”。这是保留 `npm install -g` 体验、改动最小且已有同类成功实践的方案。

但它仍是开发者侧载路径，不是面向所有普通用户的零配置方案。真正符合系统产品分发模型的方案是 public HNP + 签名 HAP。npm 本身无法绕过 XPM 信任边界，也没有一个 package.json 字段可以声明“允许执行此 ELF”。

## 参考资料

| 来源 | 类型 | 用途 |
|---|---|---|
| [HarmonyOS NEXT 安全技术白皮书](https://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/privacy/privary-new/250610/down/HarmonyOS%20NEXT%E5%AE%89%E5%85%A8%E6%8A%80%E6%9C%AF%E7%99%BD%E7%9A%AE%E4%B9%A6.pdf) | 华为官方 | 代码强制签名与执行权限 |
| [华为开发者 FAQ：插件二进制权限与签名](https://developer.huawei.com/consumer/cn/doc/doccenter-dev-faq/faqs-access-control-23) | 华为官方 | 非市场扩展、安全开关、签名要求 |
| [OpenHarmony hapsigner](https://gitcode.com/openharmony/developtools_hapsigner/tree/master) | 官方源码 | ELF 签名工具与 self-sign |
| [二进制签名工具文档](https://developer.harmonyos.cool/docs/tools/cli-tools/binary-sign-tool/) | 文档镜像 | Linux/PC/Java 工具获取方式与命令参数 |
| [OpenHarmony HNP 指南](https://gitcode.com/openharmony/startup_appspawn/blob/master/service/hnp/README_zh.md) | 官方源码文档 | HNP 打包、HAP 集成和签名 |
| [Electron HNP 指南](https://gitcode.com/CPF-Electron/Electron/blob/main/docs/hnp-packaging-guide/README.md) | 工程实践 | PC 25 XPM 日志与 HNP 解法 |
| [Termony](https://github.com/TermonyHQ/Termony) | 工程实践 | HNP 与第三方 elf-loader 来源 |
| [OHcode](https://github.com/HanversionOvO/OHcode) | 工程实践 | Node/bash/rg/Electron 的 HNP 分发 |
| [Harmonybrew](https://harmonybrew.atomgit.com/) | 工程实践 | 鸿蒙 PC 原生包管理与安全开关 |
| [codex-harmonyos](https://github.com/QinpanWan/codex-harmonyos) | 工程实践 | npm tarball 下载后自签并运行 |
| [ohos-bst-light](https://github.com/hqzing/ohos-bst-light) | 第三方源码 | append-only 自签算法与多语言实现 |
| [@ohos-ports/codex](https://www.npmjs.com/package/@ohos-ports/codex) | 一手发布物 | npm wrapper 与 GitCode 定制依赖 |
| [JavaScript_Package_For_HarmonyOS](https://gitcode.com/OpenHarmonyPCDeveloper/JavaScript_Package_For_HarmonyOS) | 工程实践与一手发布物 | Codex OHOS 源码移植补丁、真机报告和 tarball |
| [deepseek-harness-harmonyos](https://github.com/shd101wyy/deepseek-harness-harmonyos) | 工程实践 | JS CLI 与 native addon 适配 |
| [鸿蒙 PC ELF 签名实测](https://hu60.net/q.php/bbs.topic.107186.html) | 社区实测 | self-sign 命令与限制 |
| [aria2-harmonyos](https://github.com/HanversionOvO/aria2-harmonyos) | 社区实测 | 独立 ELF 自签运行 |
| [BitFun CLI npm 包](https://www.npmjs.com/package/@bitfun-test/bitfun-cli) | 一手发布物 | OpenHarmony 平台 launcher 样本 |
| [Rollup OpenHarmony 包](https://www.npmjs.com/package/@rollup/rollup-openharmony-arm64) | 一手发布物 | N-API binding 样本 |
| [esbuild OpenHarmony 包](https://www.npmjs.com/package/@esbuild/openharmony-arm64) | 一手发布物 | JS + WASM 规避 ELF 样本 |

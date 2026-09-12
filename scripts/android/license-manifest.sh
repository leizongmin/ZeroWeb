#!/usr/bin/env bash
# FR-010/NFR-008：APK 交付的依赖与许可证清单（可重复生成）。
# 以 zero-android-browser 为根遍历 cargo metadata 的目标平台依赖闭包，
# 提取 name/version/license 输出 Markdown 清单。未知许可证的包单独列「跟进」。
#
# 用法: license-manifest.sh [output.md]
#   默认输出 docs/goal/android-browser/evidence/license-manifest.md
#   平台由 ZERO_LICENSE_TARGET 控制（默认 aarch64-linux-android，即 release APK ABI）
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
output="${1:-$repo_root/docs/goal/android-browser/evidence/license-manifest.md}"
target="${ZERO_LICENSE_TARGET:-aarch64-linux-android}"
meta_file=$(mktemp)
trap 'rm -f "$meta_file"' EXIT

command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 2; }
cargo metadata --format-version 1 --filter-platform "$target" >"$meta_file"

python3 - "$meta_file" "$output" "$target" <<'PYEOF'
import json
import subprocess
import sys
from datetime import datetime, timezone

meta_file, output, target = sys.argv[1], sys.argv[2], sys.argv[3]
with open(meta_file, encoding="utf-8") as handle:
    meta = json.load(handle)

root = next((n for n in meta["resolve"]["nodes"] if "#zero-android-browser@" in n["id"]), None)
if root is None:
    sys.exit("zero-android-browser not found in cargo metadata resolve graph")

nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
seen, stack = set(), [root["id"]]
while stack:
    current = stack.pop()
    if current in seen:
        continue
    seen.add(current)
    stack.extend(nodes[current]["dependencies"])

packages = {p["id"]: p for p in meta["packages"]}
rows = sorted(
    (p["name"], p["version"], p.get("license") or "unknown")
    for i in seen
    for p in [packages[i]]
)
unknown = [(name, version) for name, version, lic in rows if lic == "unknown"]
root_pkgs = [name for name, _, _ in rows if name.startswith("zero-")]

lines = [
    "# Android APK 依赖与许可证清单",
    "",
    f"**生成日期**: {datetime.now(timezone.utc).date().isoformat()}",
    f"**目标平台**: {target}（release APK ABI；`make android-renderer-apk` 产物）",
    f"**闭包根**: zero-android-browser（browser 进程 cdylib，全部 JNI/Rust 面）",
    f"**包总数**: {len(rows)}（workspace 内 {len(root_pkgs)}）",
    "",
    "## Rust 依赖闭包（cargo metadata --filter-platform，可重复）",
    "",
    "| crate | version | license |",
    "|-------|---------|---------|",
]
lines += [f"| {name} | {version} | {lic} |" for name, version, lic in rows]

lines += [
    "",
    "## APK 捆绑组件（非 cargo 闭包）",
    "",
    "| 组件 | 许可证 | 说明 |",
    "|------|--------|------|",
    "| libc++_shared.so | Apache-2.0 WITH LLVM-exception | NDK C++ 运行时（renderer/完整版 APK 打包） |",
    "| V8（renderer feature APK） | BSD-3-Clause（Chromium/V8） | rusty_v8 v150.2.0 源码交叉编译，静态链接进 libzero_android_browser.so |",
    "| Kotlin/Compose 运行时（Jetpack Compose、Material3） | Apache-2.0 | gradle libs.versions.toml 版本锁（apps/android-browser/gradle/libs.versions.toml） |",
    "| Android framework API | Android SDK 许可证 | minSdk 26 / targetSdk 36 |",
    "",
    "## 许可证未知项跟进",
    "",
]
lines += [f"- {name} {version}" for name, version in unknown] if unknown else ["（无）"]
lines.append("")

with open(output, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
print(f"wrote {output}: {len(rows)} packages, {len(unknown)} unknown licenses")
PYEOF

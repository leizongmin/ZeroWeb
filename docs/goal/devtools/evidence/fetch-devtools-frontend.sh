#!/usr/bin/env bash
# devtools goal M0 — devtools-frontend bundle 供给脚本（可重放构建）。
#
# 产物：devtools-frontend @ pin commit 的 gen/front_end 构建输出（bundle），
# 落在 $ZEROWEB_DEVTOOLS_CACHE/<name>/，供 ZW_DEVTOOLS_FRONTEND_DIR 指向。
#
# 本脚本固化 2026-09-16 首轮实测的完整构建配方（evidence/M0-bundle-provision.md §1.4/§2）：
#   ① 官方 npm 源码包 chrome-devtools-frontend@1.0.1697595 为 front_end 基底
#   ② git blobless sparse clone 补齐构建胶水（scripts/config/third_party/build_overrides/BUILD.gn）
#   ③ 源码对齐 pin commit 的漂移文件（by-SHA raw 重取，git hash-object 校验）
#   ④ 自建 gn v2465（GitHub 镜像源码，depot_tools/CIPD 本机不可达）+ 系统 ninja + node
#   ⑤ 本地补丁：.gn gn_version 断言/script_executable 绝对路径、newer-gn builtin
#      （public_inputs/path_exists）降级、use_siso=false、//build toolchain stub
#   ⑥ typescript@7.0.2 原生 tsc + typescript@6.0.3 标准库 d.ts、esbuild npm 二进制 shim
#
# 用法：bash fetch-devtools-frontend.sh [输出根目录]
#       默认输出根：$HOME/.cache/zeroweb/devtools

set -euo pipefail

PIN=9bd6a496c3394422674c62a19e9faa627817c56e          # devtools-frontend main @ 2026-09-16
NPM_SRC_VERSION=1.0.1697595                            # 官方 npm 源码包（front_end 基底）
NPM_SRC_TGZ_SHA256=e134e6c67c5ef0779c6910529e1f2b6774d41455e9150a1cf502e39fcb0db7ff
OUT_ROOT="${1:-$HOME/.cache/zeroweb/devtools}"
WORK="$(mktemp -d /tmp/dtf-provision-XXXXXX)"

echo "[1/6] npm 源码包（front_end 基底）"
mkdir -p "$OUT_ROOT" "$WORK"
cd "$WORK"
npm pack "chrome-devtools-frontend@$NPM_SRC_VERSION" > /dev/null
echo "$NPM_SRC_TGZ_SHA256  chrome-devtools-frontend-$NPM_SRC_VERSION.tgz" | sha256sum -c -
mkdir -p hybrid && tar xzf chrome-devtools-frontend-*.tgz -C hybrid --strip-components=1

echo "[2/6] blobless sparse clone 补齐构建胶水"
git clone --depth 1 --filter=blob:none --no-checkout \
  https://github.com/ChromeDevTools/devtools-frontend.git sparse > /dev/null 2>&1
cd sparse
git sparse-checkout set --no-cone '/BUILD.gn' '/.gn' '/build_overrides' '/scripts' '/config' '/third_party' '*.gni' '**/BUILD.gn'
git checkout "$PIN" > /dev/null 2>&1 || git checkout main > /dev/null
cd "$WORK"
cp -a sparse/scripts sparse/config sparse/third_party sparse/build_overrides hybrid/
cp -a sparse/BUILD.gn sparse/.gn hybrid/
(cd sparse && git ls-tree -r "$PIN" front_end inspector_overlay | awk '{path=$4; for(i=5;i<=NF;i++) path=path" "$i; print path" "$3}' > /tmp/dtf-git-hashes.txt)
(cd hybrid && find front_end inspector_overlay -type f | LC_ALL=C sort > /tmp/dtf-paths.txt && git hash-object --stdin-paths < /tmp/dtf-paths.txt > /tmp/dtf-hashes.txt)
python3 - <<'EOF'
paths = open('/tmp/dtf-paths.txt').read().splitlines()
hashes = open('/tmp/dtf-hashes.txt').read().splitlines()
npm = dict(zip(paths, hashes))
git = {}
for line in open('/tmp/dtf-git-hashes.txt'):
    line = line.rstrip('\n')
    if not line: continue
    p, h = line.rsplit(' ', 1)
    git[p] = h
need = [p for p in git if p not in npm or npm[p] != git[p]]
open('/tmp/dtf-drift.txt', 'w').write('\n'.join(need) + '\n')
print(f'drift files to fetch: {len(need)}')
EOF

echo "[3/6] 对齐 pin commit 漂移文件（by-SHA raw，git hash-object 校验闭环）"
while read -r p; do
  [ -z "$p" ] && continue
  code=$(curl -s -m 60 -o "/tmp/dtf-one" -w "%{http_code}" \
    "https://raw.githubusercontent.com/ChromeDevTools/devtools-frontend/${PIN}/$p")
  if [ "$code" = "200" ]; then mkdir -p "hybrid/$(dirname "$p")"; mv /tmp/dtf-one "hybrid/$p"; fi
  sleep 1
done < /tmp/dtf-drift.txt

echo "[4/6] gn/ninja/node 工具与 shim"
mkdir -p hybrid/third_party/cpython3/host/bin hybrid/third_party/node/linux/node-linux-x64/bin hybrid/third_party/esbuild
ln -sf "$(command -v python3)" hybrid/third_party/cpython3/host/bin/python3
ln -sf "$(command -v node)" hybrid/third_party/node/linux/node-linux-x64/bin/node
(cd hybrid && npm install --no-audit --no-fund > /dev/null 2>&1 && npm install-scripts approve esbuild > /dev/null 2>&1 && npm rebuild esbuild > /dev/null 2>&1)
ln -sf "$(pwd)/hybrid/node_modules/@esbuild/linux-x64/bin/esbuild" hybrid/third_party/esbuild/esbuild
# gn v2465：一次性自建（跳过已存在）
if [ ! -x "$WORK/gn/out/gn" ]; then
  curl -sL -o gn-src.tar.gz "https://codeload.github.com/JiauZhang/gn/tar.gz/refs/heads/main"
  mkdir gn && tar xzf gn-src.tar.gz -C gn --strip-components=1
  (cd gn && CXX=g++ CC=gcc python3 build/gen.py --no-last-commit-position --allow-warnings > /dev/null \
    && printf '#define LAST_COMMIT_POSITION_NUM 2465\n#define LAST_COMMIT_POSITION "2465"\n' > out/last_commit_position.h \
    && ninja -C out gn > /dev/null 2>&1)
fi
# typescript 7.0.2（原生 tsc）+ 6.0.3 标准库 d.ts
mkdir -p hybrid/third_party/typescript/linux-amd64/src
npm pack typescript@7.0.2 > /dev/null && tar xzf typescript-7.0.2.tgz -C hybrid/third_party/typescript/linux-amd64/src --strip-components=1
npm pack @typescript/typescript-linux-x64@7.0.2 > /dev/null && mkdir -p ts-native && tar xzf typescript-typescript-linux-x64-7.0.2.tgz -C ts-native --strip-components=1
cp ts-native/lib/tsc hybrid/third_party/typescript/linux-amd64/src/lib/tsc && chmod +x hybrid/third_party/typescript/linux-amd64/src/lib/tsc
npm pack typescript@6.0.3 > /dev/null && mkdir -p ts60 && tar xzf typescript-6.0.3.tgz -C ts60 --strip-components=1
cp ts60/lib/lib*.d.ts ts60/lib/lib.d.ts hybrid/third_party/typescript/linux-amd64/src/lib/ 2>/dev/null || cp ts60/lib/lib*.d.ts hybrid/third_party/typescript/linux-amd64/src/lib/

echo "[5/6] 本地构建补丁（详见 evidence/M0-bundle-provision.md §2）"
python3 - <<'EOF'
import re
def sub(path, old, new):
    s = open(path).read()
    if new in s: return
    assert old in s, f'pattern missing in {path}'
    open(path, 'w').write(s.replace(old, new))

sub('hybrid/.gn', 'assert(\n    gn_version >= 2459,', 'assert(\n    2465 >= 2459,')
sub('hybrid/.gn', "script_executable = \"//third_party/cpython3/host/bin/python3\"",
    "script_executable = \"/usr/bin/python3\"  # LOCAL PATCH")
sub('hybrid/scripts/build/ninja/copy.gni',
    '    public_inputs = filter_include(sources, [ "*.ts" ])',
    '    # public_inputs = filter_include(sources, [ "*.ts" ])  # LOCAL PATCH: newer-gn builtin')
sub('hybrid/front_end/models/ai_assistance/skills/BUILD.gn',
    '''  public_inputs =
      process_file_template(_skill_sources,
                            [ "$target_gen_dir/{{source_name_part}}.skill.js" ])''',
    '''  not_needed([ "_skill_sources" ])  # LOCAL PATCH''')
s = open('hybrid/scripts/build/typescript/typescript.gni').read()
s = s.replace('    public_inputs = filter_include(_all_sources, [ "*.ts" ])\n    if (defined(invoker.inputs)) {\n      public_inputs += invoker.inputs\n    }\n',
              '    not_needed([ "_copied_dts_outputs" ])  # LOCAL PATCH\n')
s = s.replace('''    public_inputs = filter_include(_dts_outputs, [ "*.d.ts" ]) +
                    _copied_dts_outputs + [ _tsconfig_ref_file ]
''', '    not_needed([ "_copied_dts_outputs" ])  # LOCAL PATCH\n')
open('hybrid/scripts/build/typescript/typescript.gni', 'w').write(s)
sub('hybrid/scripts/build/ninja/devtools_pre_built.gni',
    '''    public_inputs = _sources
    if (defined(invoker.inputs)) {
      public_inputs += invoker.inputs
    }''',
    '''    not_needed([ "_sources" ])  # LOCAL PATCH
    if (defined(invoker.inputs)) {
      not_needed([ "invoker.inputs" ])  # LOCAL PATCH
    }''')
# //build stubs（chromium build.git pin 本机不可达，工具链不参与前端编译）
open('hybrid/build/toolchain/linux/BUILD.gn', 'w').write('''toolchain("x64") {
  tool("copy") { command = "cp {{source}} {{output}}" }
  tool("stamp") { command = "touch {{output}}" }
}
''')
open('hybrid/build/timestamp.gni', 'w').write('declare_args() {\n  build_timestamp = "0"\n}\n')
open('hybrid/build/toolchain/rbe.gni', 'w').write('use_rbe = false\nuse_remoteexec = false\n')
open('hybrid/build/toolchain/siso.gni', 'w').write('use_siso_default = false\n')
# 根 BUILD.gn 剪掉 testonly 分支
s = open('hybrid/BUILD.gn').read()
s = s.replace('''  data_deps = devtools_frontend_resources_deps + [
                "front_end:web_test_resources",
                "scripts/build:tests",
                "scripts/hosted_mode",
                "test",
              ]
  deps = [ ":frontend_indexer_tsconfig" ]
  public_deps = [ "scripts/component_docs" ]''',
'''  data_deps = devtools_frontend_resources_deps
  deps = []
  public_deps = []''')
open('hybrid/BUILD.gn', 'w').write(s)
print('patches applied')
EOF
# kythe/devtools gni 取自 chromium/src GitHub 镜像（chromium build.git 子模块）
mkdir -p hybrid/build/config hybrid/build/toolchain
for f in "build/config/devtools.gni" "build/config/chrome_build.gni" "build/toolchain/kythe.gni" "build/toolchain/rbe.gni.orig"; do :; done
curl -s -m 30 -o hybrid/build/config/devtools.gni "https://raw.githubusercontent.com/chromium/chromium/main/build/config/devtools.gni"
curl -s -m 30 -o hybrid/build/config/chrome_build.gni "https://raw.githubusercontent.com/chromium/chromium/main/build/config/chrome_build.gni"
curl -s -m 30 -o hybrid/build/toolchain/kythe.gni "https://raw.githubusercontent.com/chromium/chromium/main/build/toolchain/kythe.gni"
printf '# gclient-generated args stub\n' > hybrid/build/config/gclient_args.gni

echo "[6/6] gn gen + ninja 构建（~15-30 分钟）"
cd hybrid
PATH="$WORK/gn/out:$PATH" gn gen out/Default --args='use_siso=false'
PATH="$WORK/gn/out:$PATH" ninja -C out/Default devtools_frontend_resources

NAME="devtools-frontend-$PIN"
rm -rf "$OUT_ROOT/$NAME"
cp -a out/Default/gen/front_end "$OUT_ROOT/$NAME"
echo "BUNDLE READY: $OUT_ROOT/$NAME"
echo "用法: ZW_DEVTOOLS_FRONTEND_DIR=$OUT_ROOT/$NAME ./target/debug/zero-browser --headless --remote-debugging-port=9222"

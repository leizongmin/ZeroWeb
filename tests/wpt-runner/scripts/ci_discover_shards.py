#!/usr/bin/env python3
# 分片自动发现（discover-shards job）：从 wpt-runner 源码 match 臂提取全部
# testharness-* 套件，diff ci-shards.yml 手调分片，剩余套件按 ci-catchall.tsv
# （fetch 目标覆写，缺省约定 fetch-wpt-<suite>）两个一组打包成 catch-all 分片，
# 与手调分片一起输出为 matrix JSON——新 runner 套件零 CI 编辑即入全量对账。
#
# 约束与失败形态：
# - 手调分片里出现 runner 没有的套件名 → 硬错误（改名/删套件未同步，先红在这里）。
# - catch-all 套件无 fetch 目标（registry 未登记且无约定目标）→ fetch 步骤
#   `make fetch-wpt-<suite>` 失败显式红，按报错补一行 registry 即可。
# - 预算取保守默认（time_limit 5400 / job_timeout 300）：跑不动的面该分片红、
#   该面记 0（与未跑同态），需要常驻时再提升进 ci-shards.yml 手调。
import argparse
import json
import re
import sys


def extract_runner_suites(src_path):
    """main.rs 分发 match 臂的字面量（`"testharness-x" =>`）即套件全集。"""
    text = open(src_path, encoding="utf-8").read()
    suites = re.findall(r'(?m)^\s*"testharness-([a-z0-9-]+)"\s*=>', text)
    dup = {s for s in suites if suites.count(s) > 1}
    if dup:
        sys.exit(f"runner 源码存在重复分发臂: {sorted(dup)}")
    if len(suites) < 40:
        sys.exit(f"runner 套件提取异常：仅 {len(suites)} 个，疑似源码结构变化")
    return sorted(suites)


def parse_tuned_shards(yml_path):
    """ci-shards.yml 手调分片（缩进相对解析：`shards:` 根键 + `- name:` 条目）。"""
    lines = open(yml_path, encoding="utf-8").read().split("\n")
    root = next((i for i, l in enumerate(lines) if re.match(r"^shards:\s*$", l)), None)
    if root is None:
        sys.exit(f"{yml_path} 缺少 shards: 根键")
    entry_re = re.compile(r"^( *)- name: (.*)$")
    key_re = re.compile(r"^( *)([A-Za-z_-]+): (.*)$")
    entry_indent = None
    shards, cur = [], None
    i = root + 1
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            i += 1
            continue
        indent = len(line) - len(line.lstrip())
        if entry_indent is not None and indent < entry_indent:
            break  # 条目区结束（文件尾或回到根层级；相邻条目同缩进不受影响）
        m = entry_re.match(line)
        if m:
            entry_indent = len(m.group(1))
            cur = {"name": m.group(2).strip().strip('"'), "suites": "", "fetch": ""}
            shards.append(cur)
            i += 1
            continue
        m = key_re.match(line)
        if m and cur is not None:
            key, val = m.group(2), m.group(3).strip()
            if val in (">-", "|"):
                cont = []
                i += 1
                while i < len(lines):
                    nxt = lines[i]
                    ns = nxt.strip()
                    if not ns or ns.startswith("#"):
                        i += 1
                        continue
                    ni = len(nxt) - len(nxt.lstrip())
                    if ni <= entry_indent + 2:
                        break
                    cont.append(ns)
                    i += 1
                val = " ".join(cont)
                i -= 1  # 外层统一 i += 1
            else:
                q = re.match(r'^"(.*)"(?:\s+#.*)?$', val)
                if q:
                    val = q.group(1)
                else:
                    val = val.split(" #", 1)[0].strip()
            cur[key] = val
        i += 1
    if len(shards) < 15:
        sys.exit(f"{yml_path} 分片解析异常：仅 {len(shards)} 条")
    return shards


def parse_registry(path):
    """ci-catchall.tsv：suite<TAB>make fetch 目标（空格分隔）。# 开头为注释。"""
    reg = {}
    try:
        for line in open(path, encoding="utf-8"):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            cols = line.split("\t")
            if len(cols) != 2 or not re.fullmatch(r"[A-Za-z0-9- ]+", cols[1].strip()):
                sys.exit(f"{path} 行格式错误（应为 suite<TAB>fetch 目标）: {line}")
            reg[cols[0].strip()] = cols[1].strip()
    except FileNotFoundError:
        pass  # 无 registry = 全部走缺省约定
    return reg


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--runner-src", required=True)
    ap.add_argument("--tuned", required=True)
    ap.add_argument("--registry", default="")
    ap.add_argument("--out-shards", default="")
    ap.add_argument("--chunk", type=int, default=2, help="catch-all 每分片套件数")
    args = ap.parse_args()

    runner = extract_runner_suites(args.runner_src)
    tuned = parse_tuned_shards(args.tuned)
    registry = parse_registry(args.registry) if args.registry else {}

    tuned_suites = []
    for shard in tuned:
        for s in shard.get("suites", "").split():
            if s not in runner:
                sys.exit(f"手调分片 {shard['name']} 的套件 {s} 不在 runner 源码中（改名/删除未同步？）")
            tuned_suites.append(s)
    tuned_set = set(tuned_suites)
    leftovers = [s for s in runner if s not in tuned_set]

    catchall = []
    for n in range(0, len(leftovers), args.chunk):
        group = leftovers[n : n + args.chunk]
        fetchs = []
        for s in group:
            t = registry.get(s, f"fetch-wpt-{s}")
            if t not in fetchs:
                fetchs.append(t)
        catchall.append(
            {
                "name": f"catch-all-{n // args.chunk + 1}",
                "suites": " ".join(group),
                "fetch": " ".join(fetchs),
                "fetch_scripts": "",
                "time_limit": "5400",
                "job_timeout": "300",
            }
        )

    shards = tuned + catchall
    out = json.dumps(shards, ensure_ascii=False, separators=(",", ":"))
    if args.out_shards:
        open(args.out_shards, "w", encoding="utf-8").write(out)
    print(
        f"runner 套件 {len(runner)} / 手调分片 {len(tuned)} 覆盖 {len(tuned_set)} 套件"
        f" / 零头 {len(leftovers)}: {leftovers or '无'} / catch-all 分片 {len(catchall)}"
    )
    for c in catchall:
        print(f"  {c['name']}: suites=[{c['suites']}] fetch=[{c['fetch']}]")


if __name__ == "__main__":
    main()

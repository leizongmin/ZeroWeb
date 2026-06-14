//! test-guard — 跨平台 (macOS / Linux) 测试进程内存与超时防护。
//!
//! 包裹 `cargo test` / `zero-wpt-runner` 等命令，监控整棵子进程树的内存：
//! 任一子进程 RSS 超过单进程阈值、或全树 RSS 总和超过总量阈值、或总时长超过
//! 超时上限时，杀掉整棵进程树并以 124 退出。防止单个内存型 bug（如无限循环
//! realloc、CSS parser 未闭合括号死循环）吃光内存触发系统级 OOM，连累
//! tmux session / rally 无人值守流程被整体回收。
//!
//! 用法：
//!     test-guard [--per-proc-mem <GB>] [--total-mem <GB>] [--time-limit <sec>] -- <cmd> [args...]
//! 默认：单进程 6 GB，总量 16 GB，超时 1800 s。
//!
//! 退出码：正常 = 透传子进程退出码；内存/超时触发 = 124；参数错误 = 2；
//!         无法启动命令 = 127。
//!
//! 跨平台：依赖 `ps -ax -o pid=,ppid=,rss=` 与 `kill`（macOS/Linux 均自带），
//! 不引入外部 crate，单文件 `rustc -O` 编译。

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// 一次 `ps` 快照：pid -> (ppid, rss_kb)。
struct Procs {
    map: HashMap<u32, (u32, u64)>,
}

impl Procs {
    fn sample() -> Option<Procs> {
        let out = Command::new("ps")
            .args(["-ax", "-o", "pid=,ppid=,rss="])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout);
        let mut map = HashMap::new();
        for line in s.lines() {
            let mut it = line.split_whitespace();
            let (Some(pid), Some(ppid), Some(rss)) = (it.next(), it.next(), it.next()) else {
                continue;
            };
            if let (Ok(pid), Ok(ppid), Ok(rss)) = (pid.parse::<u32>(), ppid.parse::<u32>(), rss.parse::<u64>()) {
                map.insert(pid, (ppid, rss));
            }
        }
        Some(Procs { map })
    }

    /// 以 root 为根的后代树所有 pid（含 root 自身）。
    fn descendants(&self, root: u32) -> Vec<u32> {
        let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
        for (&pid, &(ppid, _)) in &self.map {
            children.entry(ppid).or_default().push(pid);
        }
        let mut out = vec![root];
        let mut stack = vec![root];
        while let Some(p) = stack.pop() {
            if let Some(kids) = children.get(&p) {
                for &k in kids {
                    out.push(k);
                    stack.push(k);
                }
            }
        }
        out
    }
}

/// 杀掉以 root 为根的整棵进程树。三重保险：
/// 1) 进程组 kill（root 作为 process_group(0) 的 leader，pgid == root）
/// 2) ps 重建树，对每个后代 pid 发 KILL（兜底，防子进程 setsid 脱组）
/// 3) 短暂等待后再扫一遍残留强制 kill
fn kill_tree(root: u32) {
    #[cfg(unix)]
    let _ = Command::new("kill").args(["-TERM", &format!("-{}", root)]).output();
    let kill_pid = |pid: u32| {
        #[cfg(unix)]
        let _ = Command::new("kill").args(["-KILL", &pid.to_string()]).output();
        #[cfg(not(unix))]
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    };
    if let Some(procs) = Procs::sample() {
        for pid in procs.descendants(root) {
            kill_pid(pid);
        }
    }
    std::thread::sleep(Duration::from_millis(100));
    if let Some(procs) = Procs::sample() {
        for pid in procs.descendants(root) {
            kill_pid(pid);
        }
    }
}

/// 解析命令行：返回 (单进程 GB, 总量 GB, 超时秒, 命令及其参数)。
fn parse_args() -> Result<(f64, f64, u64, Vec<String>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut per_proc = 6.0_f64;
    let mut total = 16.0_f64;
    let mut time_limit = 1800_u64;
    let mut cmd = Vec::new();
    let mut seen_dd = false;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if seen_dd {
            cmd.push(a.clone());
            i += 1;
            continue;
        }
        match a.as_str() {
            "--" => {
                seen_dd = true;
                i += 1;
            }
            "--per-proc-mem" => {
                per_proc = args
                    .get(i + 1)
                    .ok_or("--per-proc-mem 需要值")?
                    .parse()
                    .map_err(|_| "--per-proc-mem 需要数字(GB)")?;
                i += 2;
            }
            "--total-mem" => {
                total = args
                    .get(i + 1)
                    .ok_or("--total-mem 需要值")?
                    .parse()
                    .map_err(|_| "--total-mem 需要数字(GB)")?;
                i += 2;
            }
            "--time-limit" => {
                time_limit = args
                    .get(i + 1)
                    .ok_or("--time-limit 需要值")?
                    .parse()
                    .map_err(|_| "--time-limit 需要数字(秒)")?;
                i += 2;
            }
            other => return Err(format!("未知参数: {}（用 -- 分隔命令）", other)),
        }
    }
    if cmd.is_empty() {
        return Err("缺少要执行的命令（在 -- 之后给出）".into());
    }
    Ok((per_proc, total, time_limit, cmd))
}

fn main() -> std::process::ExitCode {
    let (per_proc_gb, total_gb, time_limit_s, cmd) = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("test-guard: {e}");
            eprintln!(
                "用法: test-guard [--per-proc-mem <GB>] [--total-mem <GB>] [--time-limit <sec>] -- <cmd> [args...]"
            );
            return std::process::ExitCode::from(2);
        }
    };
    let per_proc_kb = (per_proc_gb * 1024.0 * 1024.0) as u64;
    let total_kb = (total_gb * 1024.0 * 1024.0) as u64;

    let mut builder = Command::new(&cmd[0]);
    builder
        .args(&cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    #[cfg(unix)]
    builder.process_group(0);

    let mut child = match builder.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("test-guard: 无法启动 {:?}: {e}", cmd[0]);
            return std::process::ExitCode::from(127);
        }
    };
    let root = child.id();
    let deadline = Instant::now() + Duration::from_secs(time_limit_s);
    let interval = Duration::from_millis(250);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return status
                    .code()
                    .map(|c| std::process::ExitCode::from(c as u8))
                    .unwrap_or(std::process::ExitCode::from(1));
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("test-guard: wait 失败: {e}");
                return std::process::ExitCode::from(1);
            }
        }

        if Instant::now() >= deadline {
            kill_tree(root);
            let _ = child.wait();
            eprintln!("test-guard: 超时 ({time_limit_s}s)，已杀死进程树 (root pid {root})");
            return std::process::ExitCode::from(124);
        }

        if let Some(procs) = Procs::sample() {
            let pids = procs.descendants(root);
            let mut max_rss = 0_u64;
            let mut sum_rss = 0_u64;
            for pid in &pids {
                if let Some(&(_, rss)) = procs.map.get(pid) {
                    if rss > max_rss {
                        max_rss = rss;
                    }
                    sum_rss += rss;
                }
            }
            if max_rss > per_proc_kb {
                kill_tree(root);
                let _ = child.wait();
                eprintln!(
                    "test-guard: 单进程内存超限 ({max_rss} KB > {per_proc_kb} KB)，已杀死进程树 (root pid {root})"
                );
                return std::process::ExitCode::from(124);
            }
            if sum_rss > total_kb {
                kill_tree(root);
                let _ = child.wait();
                eprintln!("test-guard: 总内存超限 ({sum_rss} KB > {total_kb} KB)，已杀死进程树 (root pid {root})");
                return std::process::ExitCode::from(124);
            }
        }

        std::thread::sleep(interval);
    }
}

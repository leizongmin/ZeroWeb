//! test-guard — 跨平台 (macOS / Linux) 测试进程内存与超时防护。
//!
//! 包裹已编译的测试命令 / `zero-wpt-runner` 等命令，监控整棵子进程树的内存：
//! 任一子进程 RSS 超过单进程阈值、或全树 RSS 总和超过总量阈值、或总时长超过
//! 超时上限时，杀掉整棵进程树并以 124 退出。防止单个内存型 bug（如无限循环
//! realloc、CSS parser 未闭合括号死循环）吃光内存触发系统级 OOM，连累
//! tmux session / rally 无人值守流程被整体回收。
//!
//! 用法：
//!     test-guard [--compile-first] [--per-proc-mem <GB>] [--total-mem <GB>] [--time-limit <sec>] -- <cmd> [args...]
//! 默认：单进程 6 GB，总量 16 GB，超时 1800 s。
//!
//! 退出码：正常 = 透传子进程退出码；内存/超时触发 = 124；参数错误 = 2；
//!         无法启动命令 = 127。
//!
//! 跨平台：依赖 `ps -ax -o pid=,ppid=,rss=` 与 `kill`（macOS/Linux 均自带），
//! 不引入外部 crate，单文件 `rustc -O` 编译。
//!
//! core 转储预防：被杀/崩溃的子进程在 Linux 上经 `prlimit64(RLIMIT_CORE=0)`
//! 禁止产生 core 文件——OOM 尸体单个可达数百 MB（2026-08-18 实测仓库根积压
//! 23 个/973MB），且无人调试消费，只会吃满磁盘。std-only 约束下无 setrlimit
//! 接口，syscall 号因 arch 而异，故经 /proc/self/... 由父进程对子 pid 调
//! `prlimit --core=0 --pid`（util-linux，主流发行版自带）。prlimit 不可用
//! 时静默降级（转储再由 target-disk-guard.sh 兜底清理）。

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

/// 解析命令行：返回（先编译、单进程 GB、总量 GB、超时秒、命令及其参数）。
fn parse_args() -> Result<(bool, f64, f64, u64, Vec<String>), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut per_proc = 6.0_f64;
    let mut total = 16.0_f64;
    let mut time_limit = 1800_u64;
    let mut compile_first = false;
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
            "--compile-first" => {
                compile_first = true;
                i += 1;
            }
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
    Ok((compile_first, per_proc, total, time_limit, cmd))
}

/// 从 `cargo test` 命令构造只编译、不运行且输出 artifact 清单的等价命令。
///
/// `--` 后的参数属于测试二进制，`--no-run` 阶段不应接收它们。
fn cargo_test_compile_command(cmd: &[String]) -> Result<Vec<String>, String> {
    if cmd.first().map(String::as_str) != Some("cargo") || cmd.get(1).map(String::as_str) != Some("test") {
        return Err("--compile-first 仅支持 `cargo test ...` 命令".to_string());
    }
    let mut compile = vec![
        "cargo".to_string(),
        "test".to_string(),
        "--no-run".to_string(),
        "--message-format=json".to_string(),
    ];
    compile.extend(cmd[2..].iter().take_while(|arg| arg.as_str() != "--").cloned());
    Ok(compile)
}

/// 从 Cargo JSON 行提取一个字符串字段。Cargo artifact 路径只需要处理 JSON 转义。
fn json_string_field(line: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\":\"");
    let mut rest = line.split_once(&marker)?.1.chars();
    let mut value = String::new();
    while let Some(ch) = rest.next() {
        match ch {
            '"' => return Some(value),
            '\\' => match rest.next()? {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                '/' => value.push('/'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                other => {
                    value.push('\\');
                    value.push(other);
                }
            },
            other => value.push(other),
        }
    }
    None
}

fn test_artifact_executable(line: &str) -> Option<String> {
    let profile = line.split_once("\"profile\":")?.1;
    if line.contains("\"reason\":\"compiler-artifact\"") && profile.contains("\"test\":true") {
        return json_string_field(line, "executable");
    }
    None
}

/// 无内存阈值编译 Cargo tests，并返回可直接执行的 test artifact。
fn compile_cargo_tests(cmd: &[String]) -> Result<Vec<String>, String> {
    let compile_cmd = cargo_test_compile_command(cmd)?;
    let output = Command::new(&compile_cmd[0])
        .args(&compile_cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("无法启动 {:?}: {error}", compile_cmd[0]))?;
    if !output.status.success() {
        return Err(format!("Cargo 测试编译失败（退出码 {:?}）", output.status.code()));
    }
    let mut artifacts = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(executable) = test_artifact_executable(line) {
            artifacts.push(executable);
        }
    }
    artifacts.sort();
    artifacts.dedup();
    if artifacts.is_empty() {
        return Err("Cargo 未产出可执行测试 artifact".to_string());
    }
    Ok(artifacts)
}

/// `--` 后的参数属于 Rust test runner，可直接传给每个已编译 artifact。
fn test_runner_args(cmd: &[String]) -> Vec<String> {
    cmd.iter()
        .position(|arg| arg == "--")
        .map(|index| cmd[index + 1..].to_vec())
        .unwrap_or_default()
}

/// 提取 `cargo test` 的可选 TESTNAME 位置参数，保留目标测试范围。
fn cargo_test_filter(cmd: &[String]) -> Option<String> {
    let options_with_values = [
        "-p",
        "--package",
        "--exclude",
        "--features",
        "--target",
        "--target-dir",
        "--manifest-path",
        "-j",
        "--jobs",
        "--profile",
        "--color",
        "--message-format",
        "--bin",
        "--example",
        "--test",
        "--bench",
    ];
    let mut index = 2;
    while index < cmd.len() {
        let arg = &cmd[index];
        if arg == "--" {
            return None;
        }
        if options_with_values.contains(&arg.as_str()) {
            index += 2;
            continue;
        }
        if arg.starts_with('-') {
            index += 1;
            continue;
        }
        return Some(arg.clone());
    }
    None
}

fn main() -> std::process::ExitCode {
    let (compile_first, per_proc_gb, total_gb, time_limit_s, cmd) = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("test-guard: {e}");
            eprintln!(
                "用法: test-guard [--compile-first] [--per-proc-mem <GB>] [--total-mem <GB>] [--time-limit <sec>] -- <cmd> [args...]"
            );
            return std::process::ExitCode::from(2);
        }
    };
    let per_proc_kb = (per_proc_gb * 1024.0 * 1024.0) as u64;
    let total_kb = (total_gb * 1024.0 * 1024.0) as u64;

    if compile_first {
        let artifacts = match compile_cargo_tests(&cmd) {
            Ok(artifacts) => artifacts,
            Err(error) => {
                eprintln!("test-guard: {error}");
                return std::process::ExitCode::from(2);
            }
        };
        let executable = match std::env::current_exe() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("test-guard: 无法定位自身: {error}");
                return std::process::ExitCode::from(127);
            }
        };
        let runner_args = test_runner_args(&cmd);
        let test_filter = cargo_test_filter(&cmd);
        for artifact in artifacts {
            let status = Command::new(&executable)
                .args([
                    "--per-proc-mem".to_string(),
                    per_proc_gb.to_string(),
                    "--total-mem".to_string(),
                    total_gb.to_string(),
                    "--time-limit".to_string(),
                    time_limit_s.to_string(),
                    "--".to_string(),
                    artifact,
                ])
                .args(test_filter.iter())
                .args(&runner_args)
                .status();
            match status {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    return status
                        .code()
                        .map(|code| std::process::ExitCode::from(code as u8))
                        .unwrap_or(std::process::ExitCode::from(1));
                }
                Err(error) => {
                    eprintln!("test-guard: 无法启动测试 artifact: {error}");
                    return std::process::ExitCode::from(127);
                }
            }
        }
        return std::process::ExitCode::SUCCESS;
    }

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
    // 对整棵进程树的根设 RLIMIT_CORE=0（子进程 fork 继承；prlimit 只动 core，
    // 不碰内存阈值——那由本守卫轮询监管）。不可用则静默降级。
    #[cfg(target_os = "linux")]
    let _ = Command::new("prlimit")
        .args(["--core=0", "--pid", &root.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
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

#[cfg(test)]
mod tests {
    use super::cargo_test_compile_command;

    #[test]
    fn compile_command_preserves_cargo_options() {
        let command = vec![
            "cargo".into(),
            "test".into(),
            "-p".into(),
            "zero-net".into(),
            "dns_prefetch".into(),
        ];
        assert_eq!(
            cargo_test_compile_command(&command).unwrap(),
            vec!["cargo", "test", "--no-run", "--message-format=json", "-p", "zero-net", "dns_prefetch"]
        );
    }

    #[test]
    fn compile_command_strips_test_binary_arguments() {
        let command = vec![
            "cargo".into(),
            "test".into(),
            "--workspace".into(),
            "--".into(),
            "--test-threads=1".into(),
        ];
        assert_eq!(
            cargo_test_compile_command(&command).unwrap(),
            vec!["cargo", "test", "--no-run", "--message-format=json", "--workspace"]
        );
    }

    #[test]
    fn compile_command_rejects_non_test_command() {
        assert!(cargo_test_compile_command(&["cargo".into(), "build".into()]).is_err());
    }

    #[test]
    fn filter_preserves_testname_but_not_package_value() {
        let command = vec![
            "cargo".into(),
            "test".into(),
            "-p".into(),
            "zero-net".into(),
            "dns_prefetch".into(),
            "--".into(),
            "--exact".into(),
        ];
        assert_eq!(super::cargo_test_filter(&command), Some("dns_prefetch".into()));
        assert_eq!(super::test_runner_args(&command), vec!["--exact"]);
    }

    #[test]
    fn filter_skips_test_target_value() {
        let command = vec!["cargo".into(), "test".into(), "--test".into(), "http_session".into()];
        assert_eq!(super::cargo_test_filter(&command), None);
    }

    #[test]
    fn json_field_unescapes_artifact_path() {
        let line = r#"{"reason":"compiler-artifact","executable":"target/debug/a\"b"}"#;
        assert_eq!(super::json_string_field(line, "executable"), Some("target/debug/a\"b".into()));
    }

    #[test]
    fn artifact_filter_excludes_normal_binary() {
        let normal = r#"{"reason":"compiler-artifact","target":{"test":true},"profile":{"test":false},"executable":"target/debug/app"}"#;
        let test = r#"{"reason":"compiler-artifact","profile":{"test":true},"executable":"target/debug/deps/app-test"}"#;
        assert_eq!(super::test_artifact_executable(normal), None);
        assert_eq!(
            super::test_artifact_executable(test),
            Some("target/debug/deps/app-test".into())
        );
    }
}

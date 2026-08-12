//! R3347 deep-review 修复回归测试（zero-wasm-sandbox）。
//!
//! 本轮 deep-review 发现并修复的 `read_memory` 整数溢出 panic bug 常驻断言：
//!
//! **`read_memory` offset+len 溢出致 OOB 切片 panic（高危）**：wasmi_backend 与
//! wasmtime_backend 的 `read_memory` 用裸 `offset + len > data.len()` 边界检查——
//! `offset = usize::MAX, len >= 2` 时 `offset + len` 溢出回绕为小值（如 1），
//! `> data.len()` 误判通过，随后 `data[offset..offset+len]` 在巨大 offset 处切片 →
//! **panic**（OOB，宿主代码可触发）。**不一致证据**：同两文件的 `write_memory`
//! 正确用 `checked_add` 防溢出，`read_memory` 漏——证明疏漏非刻意。改 `read_memory`
//! 两后端均用 `checked_add`，溢出返回 None。

#![cfg(feature = "wasmi")]

use crate::WasmSandbox;

/// 辅助函数：编译 WAT 文本为 WASM 字节（镜像 basic.rs）。
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("invalid WAT")
}

/// 构建一个带导出内存的实例。
fn instance_with_mem() -> crate::WasmInstance {
    let sandbox = WasmSandbox::new();
    let wasm = wat_to_wasm(
        r#"(module
            (memory (export "mem") 1)
        )"#,
    );
    let module = sandbox.compile(&wasm).expect("compile");
    module.instantiate(&sandbox).expect("instantiate")
}

// ── Bug：read_memory offset+len 溢出不得 panic，须返回 None ─────────────

#[test]
fn test_read_memory_offset_overflow_no_panic_r3347() {
    let instance = instance_with_mem();
    // offset = usize::MAX, len = 2：offset+len 溢出回绕为 1，裸检查 `1 > data.len()`
    // 误判通过 → data[usize::MAX..usize::MAX+2] OOB panic（修复前）。
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        instance.read_memory("mem", usize::MAX, 2)
    }));
    match result {
        Ok(None) => { /* 修复后正确拒绝 */ }
        Ok(Some(_)) => panic!("read_memory offset 溢出须返回 None，实际返回了数据"),
        Err(_) => panic!("read_memory offset 溢出触发 panic（OOB 切片，bug）"),
    }
}

#[test]
fn test_read_memory_offset_max_len1_r3347() {
    let instance = instance_with_mem();
    // usize::MAX + 1 = 0（回绕），`0 > data.len()` 为 false → 通过 → data[usize::MAX..] panic。
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        instance.read_memory("mem", usize::MAX, 1)
    }));
    match result {
        Ok(None) => {}
        Ok(Some(_)) => panic!("usize::MAX+len=0 回绕误通过，须返回 None"),
        Err(_) => panic!("usize::MAX offset len=1 触发 panic"),
    }
}

#[test]
fn test_read_memory_large_offset_overflow_r3347() {
    let instance = instance_with_mem();
    // 偏大的 offset（非 MAX）+ len 溢出：如 offset = usize::MAX - 1, len = 5。
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        instance.read_memory("mem", usize::MAX - 1, 5)
    }));
    assert!(matches!(result, Ok(None)), "溢出须返回 None，不得 panic");
}

// ── 合法 read_memory 回归保护（修复不得破坏正常路径）──────────────────

#[test]
fn test_read_memory_normal_read_unchanged_r3347() {
    let mut instance = instance_with_mem();
    instance.write_memory("mem", 0, b"hello").expect("write");
    let data = instance.read_memory("mem", 0, 5).expect("read");
    assert_eq!(&data, b"hello");
}

#[test]
fn test_read_memory_zero_len_unchanged_r3347() {
    let instance = instance_with_mem();
    // len=0：offset+0 不溢出，正常返回空 vec（合法）。
    let data = instance.read_memory("mem", 0, 0).expect("zero-len read");
    assert!(data.is_empty());
}

#[test]
fn test_read_memory_normal_oob_unchanged_r3347() {
    let instance = instance_with_mem();
    let size = instance.memory_size("mem").unwrap();
    // 正常 OOB（非溢出）仍返回 None（既有行为回归）。
    assert!(instance.read_memory("mem", size - 1, 10).is_none());
}

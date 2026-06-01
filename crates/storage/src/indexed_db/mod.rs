//! IndexedDB 实现。
//!
//! 提供 IndexedDB 的核心数据结构：数据库、对象仓库、索引、游标、事务。

pub mod types;
pub mod cursor;

pub use types::*;
pub use cursor::*;

#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_advanced;

//! IndexedDB 实现。
//!
//! 提供 IndexedDB 的核心数据结构：数据库、对象仓库、索引、游标、事务。

pub mod cursor;
pub(crate) mod persistence;
pub mod types;

pub use cursor::*;
pub use types::*;

#[cfg(test)]
mod tests_advanced;
#[cfg(test)]
mod tests_basic;
#[cfg(test)]
mod tests_bool_coverage;
#[cfg(test)]
mod tests_edge;
#[cfg(test)]
mod types_coverage;
#[cfg(test)]
mod types_coverage2;
#[cfg(test)]
mod types_coverage3;
#[cfg(test)]
mod types_coverage4;
#[cfg(test)]
mod types_coverage5;

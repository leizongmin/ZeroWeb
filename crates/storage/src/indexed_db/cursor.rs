//! IDB Cursor 和 Transaction 实现。

use std::cell::RefCell;

use super::types::*;
use crate::StorageError;

/// 游标方向。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorDirection {
    /// 正向（从小到大）。
    Next,
    /// 反向（从大到小）。
    Prev,
}

/// 键游标，只迭代键。
pub struct IdbCursor {
    /// 游标方向。
    pub direction: CursorDirection,
    /// 排序后的键列表。
    pub(crate) keys: Vec<IdbKey>,
    /// 当前位置索引。
    pub(crate) current: usize,
    /// 所属 store 名称。
    pub(crate) store_name: String,
}

impl IdbCursor {
    /// 获取当前键。
    pub fn key(&self) -> Option<&IdbKey> {
        self.keys.get(self.current)
    }

    /// 前进 N 步。
    pub fn advance(&mut self, count: usize) -> bool {
        if count == 0 {
            self.current = 0;
            return true;
        }
        self.current += count;
        self.current < self.keys.len()
    }

    /// 继续到指定键。
    pub fn continue_to(&mut self, key: &IdbKey) -> bool {
        let start = self.current + 1;
        for i in start..self.keys.len() {
            if &self.keys[i] >= key {
                self.current = i;
                return true;
            }
        }
        false
    }

    /// 游标是否已到达末尾。
    pub fn is_finished(&self) -> bool {
        self.current >= self.keys.len()
    }

    /// 获取所属 store 名称。
    pub fn store_name(&self) -> &str {
        &self.store_name
    }
}

/// 值游标，迭代记录。
pub struct IdbCursorWithValue {
    /// 游标方向。
    pub direction: CursorDirection,
    /// 排序后的记录位置列表（在 store.records 中的索引）。
    pub(crate) positions: Vec<usize>,
    /// 当前位置索引。
    pub(crate) current: usize,
    /// 所属 store 名称。
    pub(crate) store_name: String,
}

impl IdbCursorWithValue {
    /// 获取当前主键。
    pub fn key(&self) -> usize {
        self.current
    }

    /// 前进 N 步。返回 false 表示已到达末尾。
    pub fn advance(&mut self, count: usize) -> bool {
        if count == 0 {
            self.current = 0;
            return true;
        }
        self.current += count;
        self.current < self.positions.len()
    }

    /// 继续到下一个位置。返回 false 表示已到达末尾。
    pub fn continue_next(&mut self) -> bool {
        self.current += 1;
        self.current < self.positions.len()
    }

    /// 游标是否已到达末尾。
    pub fn is_finished(&self) -> bool {
        self.current >= self.positions.len()
    }

    /// 获取所属 store 名称。
    pub fn store_name(&self) -> &str {
        &self.store_name
    }

    /// 获取当前位置索引（在 positions 数组中）。
    pub fn position(&self) -> usize {
        self.current
    }
}

/// IndexedDB 事务。
pub struct IdbTransaction {
    /// 涉及的 store 名称。
    pub(crate) store_names: Vec<String>,
    /// 事务模式。
    pub(crate) mode: IdbTransactionMode,
    /// 数据库名称。
    pub(crate) db_name: String,
    /// 数据库版本。
    pub(crate) db_version: u32,
    /// 是否已中止。
    pub(crate) aborted: bool,
    /// 是否已提交。
    pub(crate) committed: bool,
    /// 缓冲的变更操作。
    pub(crate) mutations: RefCell<Vec<TxMutation>>,
}

impl IdbTransaction {
    /// 检查事务是否活跃且包含指定 store。
    pub(crate) fn check_active(&self, store_name: &str) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction has been aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database("Transaction has been committed".to_string()));
        }
        if !self.store_names.iter().any(|s| s == store_name) {
            return Err(StorageError::Database(format!(
                "Store '{}' not in transaction scope",
                store_name
            )));
        }
        Ok(())
    }

    /// 提交事务。
    pub fn commit(&mut self) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction has been aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database("Transaction already committed".to_string()));
        }
        self.committed = true;
        Ok(())
    }

    /// 中止事务，丢弃缓冲的变更。
    pub fn abort(&mut self) -> Result<(), StorageError> {
        if self.aborted {
            return Err(StorageError::Database("Transaction already aborted".to_string()));
        }
        if self.committed {
            return Err(StorageError::Database(
                "Transaction already committed, cannot abort".to_string(),
            ));
        }
        self.mutations.borrow_mut().clear();
        self.aborted = true;
        Ok(())
    }

    /// 事务是否已中止。
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }

    /// 事务是否已提交。
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 获取事务模式。
    pub fn mode(&self) -> IdbTransactionMode {
        self.mode
    }

    /// 获取涉及的 store 名称。
    pub fn store_names(&self) -> &[String] {
        &self.store_names
    }

    /// 获取数据库名称。
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// 获取数据库版本。
    pub fn db_version(&self) -> u32 {
        self.db_version
    }
}

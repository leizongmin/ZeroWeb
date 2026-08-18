//! IDB Cursor 和 Transaction 实现。

use std::cell::RefCell;
use std::collections::HashMap;

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
        // R3361：saturating_add 防 usize 溢出——`current += count` 在 count 极大（如 usize::MAX）
        // 时 debug panic（overflow-checks）/ release 回绕致 current 错乱，后续 continue_to 的
        // `current + 1` 二次溢出。advance 越过 keys 末尾应判为完成（返 false），而非 panic。
        self.current = self.current.saturating_add(count);
        self.current < self.keys.len()
    }

    /// 继续到指定键。
    pub fn continue_to(&mut self, key: &IdbKey) -> bool {
        // R3361：saturating_add 防 `current + 1` 溢出——advance 经 saturating 可置 current=usize::MAX，
        // 此处 +1 溢出 panic（debug）/ 回绕（release）。
        let start = self.current.saturating_add(1);
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
        // R3361：saturating_add 防 usize 溢出（同 IdbCursor::advance）。
        self.current = self.current.saturating_add(count);
        self.current < self.positions.len()
    }

    /// 继续到下一个位置。返回 false 表示已到达末尾。
    pub fn continue_next(&mut self) -> bool {
        // R3387：saturating_add 防 `current + 1` 溢出——advance 经 saturating 可置 current=usize::MAX，
        // 此处裸 `+= 1` 溢出 panic（debug，overflow-checks）/ 回绕 0（release，然后 0 < len 误返 true
        // 「重启」游标）。同 R3361 修复的 advance/continue_to 溢出家族，本方法是当时漏修的孪生。
        self.current = self.current.saturating_add(1);
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
    pub(crate) db_version: u64,
    /// 是否已中止。
    pub(crate) aborted: bool,
    /// 是否已提交。
    pub(crate) committed: bool,
    /// 缓冲的变更操作。
    pub(crate) mutations: RefCell<Vec<TxMutation>>,
    /// R3229：事务局部 key generator 视图（store_name → next_key）——auto-inc 在此推进，
    /// 不触碰 live store.next_key；commit_tx 写回，abort 丢弃（store.next_key 未改 → 自动回滚）。
    /// 闭合 W3C IndexedDB §5.10：旧实现立即推进 live store.next_key，abort 不回滚。
    pub(crate) key_gens: RefCell<HashMap<String, u64>>,
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
    pub fn db_version(&self) -> u64 {
        self.db_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    // ── IdbCursor ──────────────────────────────────────────────────────

    fn make_cursor(keys: Vec<IdbKey>) -> IdbCursor {
        IdbCursor {
            direction: CursorDirection::Next,
            keys,
            current: 0,
            store_name: "test_store".to_string(),
        }
    }

    #[test]
    fn test_cursor_key_returns_current() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0), IdbKey::Number(3.0)]);
        assert_eq!(cursor.key(), Some(&IdbKey::Number(1.0)));
        cursor.current = 2;
        assert_eq!(cursor.key(), Some(&IdbKey::Number(3.0)));
    }

    #[test]
    fn test_cursor_key_out_of_bounds() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0)]);
        cursor.current = 5;
        assert!(cursor.key().is_none());
    }

    #[test]
    fn test_cursor_advance_by_one() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0), IdbKey::Number(3.0)]);
        assert!(cursor.advance(1));
        assert_eq!(cursor.key(), Some(&IdbKey::Number(2.0)));
    }

    #[test]
    fn test_cursor_advance_past_end() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        assert!(!cursor.advance(5)); // past end
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_cursor_advance_zero_resets() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        cursor.current = 1;
        assert!(cursor.advance(0));
        assert_eq!(cursor.current, 0);
    }

    // R3361：advance(count) 极大值不再 usize 溢出 panic——`current += count` 在 current+count >
    // usize::MAX 时 debug panic（overflow-checks）/ release 回绕致 current 错乱。
    #[test]
    fn test_cursor_advance_huge_count_no_overflow_r3361() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        cursor.current = 10; // current + count 溢出窗口：10 + (usize::MAX - 5) > usize::MAX
        // 旧实现 `current += count` debug panic（overflow-checks）。修复后 saturating 置 usize::MAX > keys.len → 完成。
        let advanced = cursor.advance(usize::MAX - 5);
        assert!(!advanced, "advance 越过末尾应判完成（返 false），不 panic");
        assert!(cursor.is_finished());
    }

    // R3361：advance 越过后 continue_to 不再二次溢出 panic——current 经 saturating 置 usize::MAX
    // 时，旧 `current + 1` 溢出 panic；saturating_add 后 start=usize::MAX > keys.len → 空迭代返 false。
    #[test]
    fn test_cursor_continue_to_after_advance_overflow_r3361() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        // current = usize::MAX（经 saturating_add；旧实现 advance(usize::MAX) from current=0 不溢出，
        // 直接赋 usize::MAX 模拟 saturating 后状态）。
        cursor.current = usize::MAX;
        // 旧实现 continue_to 的 `self.current + 1` 在此 panic（usize::MAX + 1 溢出）。
        let found = cursor.continue_to(&IdbKey::Number(5.0));
        assert!(!found, "current 已越过末尾，continue_to 应安全返 false 不 panic");
    }

    #[test]
    fn test_cursor_continue_to_existing_key() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(3.0), IdbKey::Number(5.0)]);
        assert!(cursor.continue_to(&IdbKey::Number(3.0)));
        assert_eq!(cursor.key(), Some(&IdbKey::Number(3.0)));
    }

    #[test]
    fn test_cursor_continue_to_skips_current() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(3.0), IdbKey::Number(5.0)]);
        // current is at index 0 (key=1), continue_to(1) should find key >=1 after index 0
        assert!(cursor.continue_to(&IdbKey::Number(1.0)));
        // It should move to index 1 since it starts from current+1
        assert_eq!(cursor.key(), Some(&IdbKey::Number(3.0)));
    }

    #[test]
    fn test_cursor_continue_to_nonexistent_returns_false() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0), IdbKey::Number(2.0)]);
        assert!(!cursor.continue_to(&IdbKey::Number(100.0)));
    }

    #[test]
    fn test_cursor_is_finished() {
        let mut cursor = make_cursor(vec![IdbKey::Number(1.0)]);
        assert!(!cursor.is_finished());
        cursor.advance(1);
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_cursor_store_name() {
        let cursor = make_cursor(vec![]);
        assert_eq!(cursor.store_name(), "test_store");
    }

    #[test]
    fn test_cursor_empty_keys() {
        let cursor = make_cursor(vec![]);
        assert!(cursor.is_finished());
        assert!(cursor.key().is_none());
    }

    // ── IdbCursorWithValue ─────────────────────────────────────────────

    fn make_value_cursor(positions: Vec<usize>) -> IdbCursorWithValue {
        IdbCursorWithValue {
            direction: CursorDirection::Next,
            positions,
            current: 0,
            store_name: "test_store".to_string(),
        }
    }

    #[test]
    fn test_value_cursor_key_is_current_index() {
        let cursor = make_value_cursor(vec![10, 20, 30]);
        assert_eq!(cursor.key(), 0);
    }

    #[test]
    fn test_value_cursor_advance() {
        let mut cursor = make_value_cursor(vec![10, 20, 30]);
        assert!(cursor.advance(2));
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn test_value_cursor_advance_past_end() {
        let mut cursor = make_value_cursor(vec![10, 20]);
        assert!(!cursor.advance(5));
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_value_cursor_advance_zero_resets() {
        let mut cursor = make_value_cursor(vec![10, 20]);
        cursor.current = 1;
        assert!(cursor.advance(0));
        assert_eq!(cursor.current, 0);
    }

    // R3361：IdbCursorWithValue::advance(count) 极大值不再 usize 溢出 panic（saturating_add）。
    #[test]
    fn test_value_cursor_advance_huge_count_no_overflow_r3361() {
        let mut cursor = make_value_cursor(vec![10, 20, 30]);
        cursor.current = 10; // current + count 溢出窗口
        let advanced = cursor.advance(usize::MAX - 5);
        assert!(!advanced, "advance 越过末尾应判完成，不 panic");
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_value_cursor_continue_next() {
        let mut cursor = make_value_cursor(vec![10, 20, 30]);
        assert!(cursor.continue_next());
        assert_eq!(cursor.position(), 1);
    }

    #[test]
    fn test_value_cursor_continue_next_past_end() {
        let mut cursor = make_value_cursor(vec![10]);
        assert!(!cursor.continue_next());
        assert!(cursor.is_finished());
    }

    // R3387：continue_next 经 saturating advance 置 current=usize::MAX 后不再 +1 溢出
    // panic（debug）/ 回绕 0 误判重启（release）。R3361 修了 advance/continue_to 的
    // saturating_add，但孪生方法 continue_next 用裸 `current += 1` 漏修。
    #[test]
    fn test_value_cursor_continue_next_after_saturating_advance_no_overflow_r3387() {
        let mut cursor = make_value_cursor(vec![10, 20, 30]);
        cursor.current = 10; // current + count 溢出窗口
        // saturating advance 置 current=usize::MAX（> positions.len → 完成）。
        assert!(!cursor.advance(usize::MAX - 5));
        assert_eq!(cursor.current, usize::MAX);
        // 旧实现 `self.current += 1` 在 usize::MAX + 1 溢出 panic（debug）/ 回绕 0（release，
        // 然后 0 < positions.len()=3 → 误返 true「重启」游标）。修复后 saturating 安全返 false。
        assert!(
            !cursor.continue_next(),
            "current 已饱和到 usize::MAX，continue_next 应安全判完成（返 false），不 panic/回绕"
        );
        assert!(cursor.is_finished());
    }

    #[test]
    fn test_value_cursor_store_name() {
        let cursor = make_value_cursor(vec![]);
        assert_eq!(cursor.store_name(), "test_store");
    }

    #[test]
    fn test_value_cursor_position() {
        let mut cursor = make_value_cursor(vec![10, 20, 30]);
        assert_eq!(cursor.position(), 0);
        cursor.advance(1);
        assert_eq!(cursor.position(), 1);
    }

    // ── IdbTransaction ─────────────────────────────────────────────────

    fn make_transaction() -> IdbTransaction {
        IdbTransaction {
            store_names: vec!["store1".to_string(), "store2".to_string()],
            mode: IdbTransactionMode::ReadWrite,
            db_name: "test_db".to_string(),
            db_version: 1,
            aborted: false,
            committed: false,
            mutations: RefCell::new(vec![]),
            key_gens: RefCell::new(HashMap::new()),
        }
    }

    #[test]
    fn test_transaction_commit() {
        let mut tx = make_transaction();
        assert!(tx.commit().is_ok());
        assert!(tx.is_committed());
        assert!(!tx.is_aborted());
    }

    #[test]
    fn test_transaction_commit_twice() {
        let mut tx = make_transaction();
        tx.commit().unwrap();
        assert!(tx.commit().is_err());
    }

    #[test]
    fn test_transaction_abort() {
        let mut tx = make_transaction();
        assert!(tx.abort().is_ok());
        assert!(tx.is_aborted());
        assert!(!tx.is_committed());
    }

    #[test]
    fn test_transaction_abort_twice() {
        let mut tx = make_transaction();
        tx.abort().unwrap();
        assert!(tx.abort().is_err());
    }

    #[test]
    fn test_transaction_commit_then_abort() {
        let mut tx = make_transaction();
        tx.commit().unwrap();
        assert!(tx.abort().is_err());
    }

    #[test]
    fn test_transaction_abort_then_commit() {
        let mut tx = make_transaction();
        tx.abort().unwrap();
        assert!(tx.commit().is_err());
    }

    #[test]
    fn test_transaction_mode() {
        let tx = make_transaction();
        assert_eq!(tx.mode(), IdbTransactionMode::ReadWrite);
    }

    #[test]
    fn test_transaction_store_names() {
        let tx = make_transaction();
        assert_eq!(tx.store_names(), &["store1", "store2"]);
    }

    #[test]
    fn test_transaction_db_name() {
        let tx = make_transaction();
        assert_eq!(tx.db_name(), "test_db");
    }

    #[test]
    fn test_transaction_db_version() {
        let tx = make_transaction();
        assert_eq!(tx.db_version(), 1);
    }

    #[test]
    fn test_transaction_check_active_valid() {
        let tx = make_transaction();
        assert!(tx.check_active("store1").is_ok());
        assert!(tx.check_active("store2").is_ok());
    }

    #[test]
    fn test_transaction_check_active_wrong_store() {
        let tx = make_transaction();
        assert!(tx.check_active("store3").is_err());
    }

    #[test]
    fn test_transaction_check_active_after_abort() {
        let mut tx = make_transaction();
        tx.abort().unwrap();
        assert!(tx.check_active("store1").is_err());
    }

    #[test]
    fn test_transaction_check_active_after_commit() {
        let mut tx = make_transaction();
        tx.commit().unwrap();
        assert!(tx.check_active("store1").is_err());
    }

    // ── CursorDirection ────────────────────────────────────────────────

    #[test]
    fn test_cursor_direction_equality() {
        assert_eq!(CursorDirection::Next, CursorDirection::Next);
        assert_eq!(CursorDirection::Prev, CursorDirection::Prev);
        assert_ne!(CursorDirection::Next, CursorDirection::Prev);
    }
}

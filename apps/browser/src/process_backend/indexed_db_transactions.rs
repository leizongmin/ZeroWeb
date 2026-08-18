use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use serde::Deserialize;
use zero_engine::IndexedDbHandler;
use zero_storage::StorageManager;

use super::ProcessTabBackend;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum TransactionMode {
    Readonly,
    Readwrite,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(super) enum TransactionWireRequest {
    RequestTransactionStart {
        database: String,
        stores: Vec<String>,
        mode: TransactionMode,
    },
    PollTransactionStart {
        request: u64,
    },
    CancelTransactionStart {
        request: u64,
    },
    BeginTransaction {
        database: String,
        stores: Vec<String>,
        mode: TransactionMode,
        #[serde(default)]
        lease: Option<u64>,
    },
    CommitTransaction {
        transaction: u64,
    },
    AbortTransaction {
        transaction: u64,
    },
}

pub(super) fn parse_transaction_request(request: &str) -> Result<Option<TransactionWireRequest>, String> {
    let value: serde_json::Value =
        serde_json::from_str(request).map_err(|error| format!("DataError: invalid IndexedDB request: {error}"))?;
    let Some(operation) = value.get("op").and_then(serde_json::Value::as_str) else {
        return Ok(None);
    };
    if !matches!(
        operation,
        "request_transaction_start"
            | "poll_transaction_start"
            | "cancel_transaction_start"
            | "begin_transaction"
            | "commit_transaction"
            | "abort_transaction"
    ) {
        return Ok(None);
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| format!("DataError: invalid IndexedDB transaction request: {error}"))
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DatabaseScope {
    private: bool,
    origin: String,
    database: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TransactionScope {
    database: DatabaseScope,
    stores: Vec<String>,
    mode: TransactionMode,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TransactionKey {
    renderer_id: u64,
    transaction_id: u64,
}

#[derive(Debug)]
struct PendingTransaction {
    renderer_id: u64,
    scope: TransactionScope,
}

#[derive(Debug)]
struct ActiveLease {
    renderer_id: u64,
    scope: TransactionScope,
    transaction_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TransactionStartStatus {
    Pending(u64),
    Ready(u64),
}

#[derive(Default)]
pub(super) struct IndexedDbTransactionOwner {
    next_id: u64,
    pending: HashMap<u64, PendingTransaction>,
    queues: HashMap<DatabaseScope, VecDeque<u64>>,
    leases: HashMap<u64, ActiveLease>,
    transactions: HashMap<TransactionKey, u64>,
}

impl IndexedDbTransactionOwner {
    pub(super) fn request_start(
        &mut self,
        renderer_id: u64,
        private: bool,
        origin: &str,
        database: &str,
        stores: Vec<String>,
        mode: TransactionMode,
    ) -> Result<TransactionStartStatus, String> {
        let scope = TransactionScope {
            database: DatabaseScope {
                private,
                origin: origin.to_string(),
                database: database.to_string(),
            },
            stores: normalize_stores(stores)?,
            mode,
        };
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| "UnknownError: IndexedDB transaction lease id overflow".to_string())?;
        let request_id = self.next_id;
        self.pending.insert(
            request_id,
            PendingTransaction {
                renderer_id,
                scope: scope.clone(),
            },
        );
        self.queues
            .entry(scope.database.clone())
            .or_default()
            .push_back(request_id);
        self.poll_start(renderer_id, request_id)
    }

    pub(super) fn poll_start(&mut self, renderer_id: u64, request_id: u64) -> Result<TransactionStartStatus, String> {
        let request = self
            .pending
            .get(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB transaction start request does not exist".to_string())?;
        if request.renderer_id != renderer_id {
            return Err("SecurityError: IndexedDB transaction start request belongs to another renderer".to_string());
        }
        if !self.is_eligible(request_id, &request.scope) {
            return Ok(TransactionStartStatus::Pending(request_id));
        }
        let request = self
            .pending
            .remove(&request_id)
            .expect("transaction start request checked above");
        self.remove_from_queue(request_id, &request.scope.database);
        self.leases.insert(
            request_id,
            ActiveLease {
                renderer_id,
                scope: request.scope,
                transaction_id: None,
            },
        );
        Ok(TransactionStartStatus::Ready(request_id))
    }

    pub(super) fn cancel_start(&mut self, renderer_id: u64, request_id: u64) -> Result<(), String> {
        let request = self
            .pending
            .get(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB transaction start request does not exist".to_string())?;
        if request.renderer_id != renderer_id {
            return Err("SecurityError: IndexedDB transaction start request belongs to another renderer".to_string());
        }
        let request = self
            .pending
            .remove(&request_id)
            .expect("transaction start request checked above");
        self.remove_from_queue(request_id, &request.scope.database);
        Ok(())
    }

    pub(super) fn bind(
        &mut self,
        renderer_id: u64,
        lease_id: u64,
        database: &str,
        stores: Vec<String>,
        mode: TransactionMode,
        transaction_id: u64,
    ) -> Result<(), String> {
        let stores = normalize_stores(stores)?;
        let lease = self
            .leases
            .get_mut(&lease_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB transaction lease does not exist".to_string())?;
        if lease.renderer_id != renderer_id {
            return Err("SecurityError: IndexedDB transaction lease belongs to another renderer".to_string());
        }
        if lease.scope.database.database != database || lease.scope.stores != stores || lease.scope.mode != mode {
            return Err("SecurityError: IndexedDB transaction lease scope mismatch".to_string());
        }
        if lease.transaction_id.is_some() {
            return Err("InvalidStateError: IndexedDB transaction lease is already bound".to_string());
        }
        let key = TransactionKey {
            renderer_id,
            transaction_id,
        };
        if self.transactions.contains_key(&key) {
            return Err("InvalidStateError: IndexedDB transaction is already registered".to_string());
        }
        lease.transaction_id = Some(transaction_id);
        self.transactions.insert(key, lease_id);
        Ok(())
    }

    pub(super) fn cancel_lease(&mut self, renderer_id: u64, lease_id: u64) {
        if self
            .leases
            .get(&lease_id)
            .is_some_and(|lease| lease.renderer_id == renderer_id && lease.transaction_id.is_none())
        {
            self.leases.remove(&lease_id);
        }
    }

    pub(super) fn finish(&mut self, renderer_id: u64, transaction_id: u64) {
        let key = TransactionKey {
            renderer_id,
            transaction_id,
        };
        if let Some(lease_id) = self.transactions.remove(&key) {
            self.leases.remove(&lease_id);
        }
    }

    pub(super) fn remove_renderer(&mut self, renderer_id: u64) {
        let requests = self
            .pending
            .iter()
            .filter(|(_, request)| request.renderer_id == renderer_id)
            .map(|(request_id, _)| *request_id)
            .collect::<Vec<_>>();
        for request_id in requests {
            if let Some(request) = self.pending.remove(&request_id) {
                self.remove_from_queue(request_id, &request.scope.database);
            }
        }
        let leases = self
            .leases
            .iter()
            .filter(|(_, lease)| lease.renderer_id == renderer_id)
            .map(|(lease_id, _)| *lease_id)
            .collect::<Vec<_>>();
        for lease_id in leases {
            self.leases.remove(&lease_id);
        }
        self.transactions.retain(|key, _| key.renderer_id != renderer_id);
    }

    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (self.pending.len(), self.leases.len(), self.transactions.len())
    }

    fn is_eligible(&self, request_id: u64, scope: &TransactionScope) -> bool {
        if self
            .leases
            .values()
            .any(|lease| transactions_conflict(&lease.scope, scope))
        {
            return false;
        }
        self.queues
            .get(&scope.database)
            .into_iter()
            .flat_map(|queue| queue.iter())
            .take_while(|queued| **queued != request_id)
            .filter_map(|queued| self.pending.get(queued))
            .all(|earlier| !transactions_conflict(&earlier.scope, scope))
    }

    fn remove_from_queue(&mut self, request_id: u64, database: &DatabaseScope) {
        if let Some(queue) = self.queues.get_mut(database) {
            queue.retain(|queued| *queued != request_id);
            if queue.is_empty() {
                self.queues.remove(database);
            }
        }
    }
}

fn normalize_stores(mut stores: Vec<String>) -> Result<Vec<String>, String> {
    if stores.is_empty() {
        return Err("InvalidAccessError: IndexedDB transaction scope must not be empty".to_string());
    }
    stores.sort_unstable();
    stores.dedup();
    Ok(stores)
}

fn transactions_conflict(first: &TransactionScope, second: &TransactionScope) -> bool {
    if first.database != second.database
        || (first.mode == TransactionMode::Readonly && second.mode == TransactionMode::Readonly)
    {
        return false;
    }
    first
        .stores
        .iter()
        .any(|store| second.stores.binary_search(store).is_ok())
}

impl ProcessTabBackend {
    pub(super) fn handle_indexed_db_transaction_request(
        &mut self,
        renderer_id: u64,
        private: bool,
        origin: &str,
        storage: Arc<Mutex<StorageManager>>,
        raw_request: &str,
        request: TransactionWireRequest,
    ) -> Result<String, String> {
        match request {
            TransactionWireRequest::RequestTransactionStart { database, stores, mode } => {
                let status = self.indexed_db_transactions.request_start(
                    renderer_id,
                    private,
                    origin,
                    &database,
                    stores,
                    mode,
                )?;
                transaction_start_response(status)
            }
            TransactionWireRequest::PollTransactionStart { request } => {
                let status = self.indexed_db_transactions.poll_start(renderer_id, request)?;
                transaction_start_response(status)
            }
            TransactionWireRequest::CancelTransactionStart { request } => {
                self.indexed_db_transactions.cancel_start(renderer_id, request)?;
                Ok(serde_json::json!({"cancelled": true}).to_string())
            }
            TransactionWireRequest::BeginTransaction {
                database,
                stores,
                mode,
                lease,
            } => {
                let Some(lease_id) = lease else {
                    return Err("SecurityError: IndexedDB transaction lease is required".to_string());
                };
                let response = self.call_indexed_db_handler(renderer_id, Arc::clone(&storage), origin, raw_request);
                let response = match response {
                    Ok(response) => response,
                    Err(error) => {
                        self.indexed_db_transactions.cancel_lease(renderer_id, lease_id);
                        return Err(error);
                    }
                };
                let transaction_id = serde_json::from_str::<serde_json::Value>(&response)
                    .ok()
                    .and_then(|value| value.get("transaction").and_then(serde_json::Value::as_u64));
                let Some(transaction_id) = transaction_id else {
                    self.indexed_db_transactions.cancel_lease(renderer_id, lease_id);
                    return Err("UnknownError: IndexedDB begin transaction response is invalid".to_string());
                };
                if let Err(error) =
                    self.indexed_db_transactions
                        .bind(renderer_id, lease_id, &database, stores, mode, transaction_id)
                {
                    let abort = serde_json::json!({
                        "op": "abort_transaction",
                        "transaction": transaction_id,
                    })
                    .to_string();
                    let _ = self.call_indexed_db_handler(renderer_id, Arc::clone(&storage), origin, &abort);
                    self.indexed_db_transactions.cancel_lease(renderer_id, lease_id);
                    return Err(error);
                }
                Ok(response)
            }
            TransactionWireRequest::CommitTransaction { transaction }
            | TransactionWireRequest::AbortTransaction { transaction } => {
                let response = self.call_indexed_db_handler(renderer_id, storage, origin, raw_request);
                self.indexed_db_transactions.finish(renderer_id, transaction);
                response
            }
        }
    }

    fn call_indexed_db_handler(
        &mut self,
        renderer_id: u64,
        storage: Arc<Mutex<StorageManager>>,
        origin: &str,
        request: &str,
    ) -> Result<String, String> {
        let handler: IndexedDbHandler = Arc::clone(
            self.indexed_db_handlers
                .entry(renderer_id)
                .or_insert_with(|| zero_page_runtime::indexed_db_handler(storage)),
        );
        handler(origin, request)
    }
}

fn transaction_start_response(status: TransactionStartStatus) -> Result<String, String> {
    Ok(match status {
        TransactionStartStatus::Pending(request) => {
            serde_json::json!({"ready": false, "request": request})
        }
        TransactionStartStatus::Ready(lease) => {
            serde_json::json!({"ready": true, "lease": lease})
        }
    }
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn request(
        owner: &mut IndexedDbTransactionOwner,
        renderer_id: u64,
        private: bool,
        origin: &str,
        database: &str,
        stores: &[&str],
        mode: TransactionMode,
    ) -> TransactionStartStatus {
        owner
            .request_start(
                renderer_id,
                private,
                origin,
                database,
                stores.iter().map(|store| (*store).to_string()).collect(),
                mode,
            )
            .unwrap()
    }

    #[test]
    fn conflicting_transactions_wait_in_creation_order() {
        let mut owner = IndexedDbTransactionOwner::default();
        let first = request(
            &mut owner,
            1,
            false,
            "https://app.example",
            "db",
            &["items"],
            TransactionMode::Readwrite,
        );
        let first_lease = match first {
            TransactionStartStatus::Ready(lease) => lease,
            TransactionStartStatus::Pending(_) => panic!("first transaction should start"),
        };
        owner
            .bind(
                1,
                first_lease,
                "db",
                vec!["items".to_string()],
                TransactionMode::Readwrite,
                10,
            )
            .unwrap();

        let second = request(
            &mut owner,
            2,
            false,
            "https://app.example",
            "db",
            &["items"],
            TransactionMode::Readonly,
        );
        let second_request = match second {
            TransactionStartStatus::Pending(request) => request,
            TransactionStartStatus::Ready(_) => panic!("conflicting transaction started early"),
        };
        assert_eq!(
            owner.poll_start(2, second_request).unwrap(),
            TransactionStartStatus::Pending(second_request)
        );

        owner.finish(1, 10);
        assert_eq!(
            owner.poll_start(2, second_request).unwrap(),
            TransactionStartStatus::Ready(second_request)
        );
    }

    #[test]
    fn readonly_and_disjoint_transactions_can_run_in_parallel() {
        let mut owner = IndexedDbTransactionOwner::default();
        assert!(matches!(
            request(
                &mut owner,
                1,
                false,
                "https://app.example",
                "db",
                &["items"],
                TransactionMode::Readonly,
            ),
            TransactionStartStatus::Ready(_)
        ));
        assert!(matches!(
            request(
                &mut owner,
                2,
                false,
                "https://app.example",
                "db",
                &["items"],
                TransactionMode::Readonly,
            ),
            TransactionStartStatus::Ready(_)
        ));
        assert!(matches!(
            request(
                &mut owner,
                3,
                false,
                "https://app.example",
                "db",
                &["other"],
                TransactionMode::Readwrite,
            ),
            TransactionStartStatus::Ready(_)
        ));
    }

    #[test]
    fn scheduling_is_isolated_by_partition_origin_and_database() {
        let mut owner = IndexedDbTransactionOwner::default();
        assert!(matches!(
            request(
                &mut owner,
                1,
                false,
                "https://a.example",
                "db",
                &["items"],
                TransactionMode::Readwrite,
            ),
            TransactionStartStatus::Ready(_)
        ));
        for (renderer, private, origin, database) in [
            (2, true, "https://a.example", "db"),
            (3, false, "https://b.example", "db"),
            (4, false, "https://a.example", "other"),
        ] {
            assert!(matches!(
                request(
                    &mut owner,
                    renderer,
                    private,
                    origin,
                    database,
                    &["items"],
                    TransactionMode::Readwrite,
                ),
                TransactionStartStatus::Ready(_)
            ));
        }
    }

    #[test]
    fn renderer_removal_releases_leases_and_pending_requests() {
        let mut owner = IndexedDbTransactionOwner::default();
        let first = request(
            &mut owner,
            1,
            false,
            "https://app.example",
            "db",
            &["items"],
            TransactionMode::Readwrite,
        );
        let first_lease = match first {
            TransactionStartStatus::Ready(lease) => lease,
            TransactionStartStatus::Pending(_) => panic!("first transaction should start"),
        };
        owner
            .bind(
                1,
                first_lease,
                "db",
                vec!["items".to_string()],
                TransactionMode::Readwrite,
                10,
            )
            .unwrap();
        let second = request(
            &mut owner,
            2,
            false,
            "https://app.example",
            "db",
            &["items"],
            TransactionMode::Readonly,
        );
        let second_request = match second {
            TransactionStartStatus::Pending(request) => request,
            TransactionStartStatus::Ready(_) => panic!("conflicting transaction started early"),
        };

        owner.remove_renderer(1);
        assert_eq!(
            owner.poll_start(2, second_request).unwrap(),
            TransactionStartStatus::Ready(second_request)
        );
        owner.remove_renderer(2);
        assert_eq!(owner.counts(), (0, 0, 0));
    }

    #[test]
    fn browser_rejects_begin_without_a_lease() {
        let mut backend = ProcessTabBackend::with_renderer_bin(PathBuf::from("unused-renderer"));
        let storage = Arc::clone(&backend.storage);
        let error = backend
            .handle_indexed_db_transaction_request(
                1,
                false,
                "https://app.example",
                storage,
                r#"{"op":"begin_transaction","database":"db","stores":["items"],"mode":"readonly"}"#,
                TransactionWireRequest::BeginTransaction {
                    database: "db".to_string(),
                    stores: vec!["items".to_string()],
                    mode: TransactionMode::Readonly,
                    lease: None,
                },
            )
            .unwrap_err();

        assert_eq!(error, "SecurityError: IndexedDB transaction lease is required");
        assert!(!backend.indexed_db_handlers.contains_key(&1));
    }
}

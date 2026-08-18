use std::collections::{HashMap, HashSet, VecDeque};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(super) enum ConnectionWireRequest {
    ConnectionCapabilities,
    RegisterConnection {
        connection: u64,
        database: String,
        version: u64,
    },
    CloseConnection {
        connection: u64,
    },
    RequestConnectionChange {
        database: String,
        #[serde(default)]
        new_version: Option<u64>,
    },
    PollConnectionChange {
        request: u64,
    },
}

pub(super) fn parse_connection_request(request: &str) -> Result<Option<ConnectionWireRequest>, String> {
    let value: serde_json::Value =
        serde_json::from_str(request).map_err(|error| format!("DataError: invalid IndexedDB request: {error}"))?;
    let Some(operation) = value.get("op").and_then(serde_json::Value::as_str) else {
        return Ok(None);
    };
    if !matches!(
        operation,
        "connection_capabilities"
            | "register_connection"
            | "close_connection"
            | "request_connection_change"
            | "poll_connection_change"
    ) {
        return Ok(None);
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| format!("DataError: invalid IndexedDB connection request: {error}"))
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ConnectionKey {
    pub(super) renderer_id: u64,
    pub(super) connection_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Connection {
    private: bool,
    origin: String,
    database: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ScopeKey {
    private: bool,
    origin: String,
    database: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ConnectionEvent {
    pub(super) target: ConnectionKey,
    pub(super) request_id: u64,
    pub(super) old_version: u64,
    pub(super) new_version: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ConnectionRequestStatus {
    Pending,
    Blocked,
    Ready,
}

pub(super) struct ConnectionRequestUpdate {
    pub(super) status: ConnectionRequestStatus,
    pub(super) events: Vec<ConnectionEvent>,
}

#[derive(Debug)]
struct PendingRequest {
    requester: u64,
    scope: ScopeKey,
    old_version: u64,
    new_version: Option<u64>,
    started: bool,
    waiting: HashSet<ConnectionKey>,
    awaiting_ack: HashSet<ConnectionKey>,
}

#[derive(Default)]
pub(super) struct IndexedDbConnectionOwner {
    next_request_id: u64,
    connections: HashMap<ConnectionKey, Connection>,
    pending: HashMap<u64, PendingRequest>,
    queues: HashMap<ScopeKey, VecDeque<u64>>,
}

impl IndexedDbConnectionOwner {
    pub(super) fn register(
        &mut self,
        key: ConnectionKey,
        private: bool,
        origin: &str,
        database: &str,
    ) -> Result<(), String> {
        if self.connections.contains_key(&key) {
            return Err("InvalidStateError: IndexedDB connection already exists".to_string());
        }
        self.connections.insert(
            key,
            Connection {
                private,
                origin: origin.to_string(),
                database: database.to_string(),
            },
        );
        Ok(())
    }

    pub(super) fn close(&mut self, key: ConnectionKey) -> Result<(), String> {
        if self.connections.remove(&key).is_none() {
            return Err("InvalidStateError: IndexedDB connection does not exist".to_string());
        }
        for request in self.pending.values_mut() {
            request.waiting.remove(&key);
            request.awaiting_ack.remove(&key);
        }
        Ok(())
    }

    pub(super) fn begin_request(
        &mut self,
        requester: u64,
        private: bool,
        origin: &str,
        database: &str,
        old_version: u64,
        new_version: Option<u64>,
    ) -> Result<(Option<u64>, Vec<ConnectionEvent>), String> {
        let scope = ScopeKey {
            private,
            origin: origin.to_string(),
            database: database.to_string(),
        };
        let queue_is_empty = self.queues.get(&scope).is_none_or(VecDeque::is_empty);
        if queue_is_empty && self.targets(&scope).is_empty() {
            return Ok((None, Vec::new()));
        }
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or_else(|| "UnknownError: IndexedDB connection request id overflow".to_string())?;
        let request_id = self.next_request_id;
        self.pending.insert(
            request_id,
            PendingRequest {
                requester,
                scope: scope.clone(),
                old_version,
                new_version,
                started: false,
                waiting: HashSet::new(),
                awaiting_ack: HashSet::new(),
            },
        );
        self.queues.entry(scope).or_default().push_back(request_id);
        let events = if queue_is_empty {
            self.activate(request_id)?
        } else {
            Vec::new()
        };
        Ok((Some(request_id), events))
    }

    pub(super) fn acknowledge(&mut self, request_id: u64, target: ConnectionKey) -> Result<(), String> {
        let request = self
            .pending
            .get_mut(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB connection request does not exist".to_string())?;
        if !request.awaiting_ack.remove(&target) {
            return Err("InvalidStateError: IndexedDB connection event was not pending".to_string());
        }
        Ok(())
    }

    pub(super) fn request_scope(&self, requester: u64, request_id: u64) -> Result<(bool, String, String), String> {
        let request = self
            .pending
            .get(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB connection request does not exist".to_string())?;
        if request.requester != requester {
            return Err("SecurityError: IndexedDB connection request belongs to another renderer".to_string());
        }
        Ok((
            request.scope.private,
            request.scope.origin.clone(),
            request.scope.database.clone(),
        ))
    }

    pub(super) fn status(
        &mut self,
        requester: u64,
        request_id: u64,
        current_old_version: u64,
    ) -> Result<ConnectionRequestUpdate, String> {
        let request = self
            .pending
            .get(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB connection request does not exist".to_string())?;
        if request.requester != requester {
            return Err("SecurityError: IndexedDB connection request belongs to another renderer".to_string());
        }
        let should_activate = !request.started
            && self
                .queues
                .get(&request.scope)
                .and_then(|queue| queue.front())
                .is_some_and(|queued| *queued == request_id);
        let events = if should_activate {
            self.pending
                .get_mut(&request_id)
                .expect("connection request checked above")
                .old_version = current_old_version;
            self.activate(request_id)?
        } else {
            Vec::new()
        };
        let request = self
            .pending
            .get(&request_id)
            .expect("connection request remains pending while status is computed");
        let status = if !request.started || !request.awaiting_ack.is_empty() {
            ConnectionRequestStatus::Pending
        } else if !request.waiting.is_empty() {
            ConnectionRequestStatus::Blocked
        } else {
            ConnectionRequestStatus::Ready
        };
        if status == ConnectionRequestStatus::Ready {
            let request = self
                .pending
                .remove(&request_id)
                .expect("ready connection request exists");
            if let Some(queue) = self.queues.get_mut(&request.scope) {
                debug_assert_eq!(queue.front(), Some(&request_id));
                queue.pop_front();
                if queue.is_empty() {
                    self.queues.remove(&request.scope);
                }
            }
        }
        Ok(ConnectionRequestUpdate { status, events })
    }

    pub(super) fn remove_renderer(&mut self, renderer_id: u64) {
        let connections = self
            .connections
            .keys()
            .filter(|key| key.renderer_id == renderer_id)
            .copied()
            .collect::<Vec<_>>();
        for key in connections {
            self.connections.remove(&key);
            for request in self.pending.values_mut() {
                request.waiting.remove(&key);
                request.awaiting_ack.remove(&key);
            }
        }
        let owned = self
            .pending
            .iter()
            .filter(|(_, request)| request.requester == renderer_id)
            .map(|(request_id, _)| *request_id)
            .collect::<Vec<_>>();
        for request_id in owned {
            if let Some(request) = self.pending.remove(&request_id)
                && let Some(queue) = self.queues.get_mut(&request.scope)
            {
                queue.retain(|queued| *queued != request_id);
                if queue.is_empty() {
                    self.queues.remove(&request.scope);
                }
            }
        }
    }

    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize) {
        (self.connections.len(), self.pending.len())
    }

    fn targets(&self, scope: &ScopeKey) -> HashSet<ConnectionKey> {
        self.connections
            .iter()
            .filter(|(_, connection)| {
                connection.private == scope.private
                    && connection.origin == scope.origin
                    && connection.database == scope.database
            })
            .map(|(key, _)| *key)
            .collect()
    }

    fn activate(&mut self, request_id: u64) -> Result<Vec<ConnectionEvent>, String> {
        let request = self
            .pending
            .get(&request_id)
            .ok_or_else(|| "InvalidStateError: IndexedDB connection request does not exist".to_string())?;
        let targets = self.targets(&request.scope);
        let old_version = request.old_version;
        let new_version = request.new_version;
        let request = self
            .pending
            .get_mut(&request_id)
            .expect("connection request checked above");
        request.started = true;
        request.waiting.clone_from(&targets);
        request.awaiting_ack.clone_from(&targets);
        Ok(targets
            .into_iter()
            .map(|target| ConnectionEvent {
                target,
                request_id,
                old_version,
                new_version,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(renderer_id: u64, connection_id: u64) -> ConnectionKey {
        ConnectionKey {
            renderer_id,
            connection_id,
        }
    }

    #[test]
    fn request_waits_for_event_ack_and_connection_close() {
        let mut owner = IndexedDbConnectionOwner::default();
        owner.register(key(1, 10), false, "https://app.example", "db").unwrap();
        owner.register(key(2, 20), false, "https://app.example", "db").unwrap();

        let (request_id, events) = owner
            .begin_request(3, false, "https://app.example", "db", 1, Some(2))
            .unwrap();
        let request_id = request_id.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            owner.status(3, request_id, 1).unwrap().status,
            ConnectionRequestStatus::Pending
        );

        owner.acknowledge(request_id, key(1, 10)).unwrap();
        owner.acknowledge(request_id, key(2, 20)).unwrap();
        assert_eq!(
            owner.status(3, request_id, 1).unwrap().status,
            ConnectionRequestStatus::Blocked
        );

        owner.close(key(1, 10)).unwrap();
        assert_eq!(
            owner.status(3, request_id, 1).unwrap().status,
            ConnectionRequestStatus::Blocked
        );
        owner.close(key(2, 20)).unwrap();
        assert_eq!(
            owner.status(3, request_id, 1).unwrap().status,
            ConnectionRequestStatus::Ready
        );
        assert!(owner.status(3, request_id, 1).is_err());
    }

    #[test]
    fn request_isolated_by_partition_origin_and_database() {
        let mut owner = IndexedDbConnectionOwner::default();
        owner.register(key(1, 1), false, "https://a.example", "db").unwrap();
        owner.register(key(2, 2), true, "https://a.example", "db").unwrap();
        owner.register(key(3, 3), false, "https://b.example", "db").unwrap();
        owner.register(key(4, 4), false, "https://a.example", "other").unwrap();

        let (_, events) = owner
            .begin_request(9, false, "https://a.example", "db", 1, None)
            .unwrap();
        assert_eq!(
            events.iter().map(|event| event.target).collect::<Vec<_>>(),
            vec![key(1, 1)]
        );
    }

    #[test]
    fn renderer_removal_unblocks_waiters_and_cancels_owned_requests() {
        let mut owner = IndexedDbConnectionOwner::default();
        owner.register(key(1, 1), false, "https://a.example", "db").unwrap();
        let (request_id, _) = owner
            .begin_request(2, false, "https://a.example", "db", 1, Some(2))
            .unwrap();
        let request_id = request_id.unwrap();

        owner.remove_renderer(1);
        assert_eq!(
            owner.status(2, request_id, 1).unwrap().status,
            ConnectionRequestStatus::Ready
        );

        owner.register(key(3, 3), false, "https://a.example", "db").unwrap();
        let (owned_request, _) = owner
            .begin_request(2, false, "https://a.example", "db", 1, None)
            .unwrap();
        owner.remove_renderer(2);
        assert!(owner.status(2, owned_request.unwrap(), 1).is_err());
    }

    #[test]
    fn duplicate_connection_and_cross_renderer_poll_are_rejected() {
        let mut owner = IndexedDbConnectionOwner::default();
        owner.register(key(1, 1), false, "https://a.example", "db").unwrap();
        assert!(owner.register(key(1, 1), false, "https://a.example", "db").is_err());
        let (request_id, _) = owner
            .begin_request(2, false, "https://a.example", "db", 1, Some(2))
            .unwrap();
        assert!(owner.status(3, request_id.unwrap(), 1).is_err());
    }

    #[test]
    fn connection_changes_are_fifo_with_fresh_targets() {
        let mut owner = IndexedDbConnectionOwner::default();
        owner.register(key(1, 1), false, "https://a.example", "db").unwrap();
        let (first, first_events) = owner
            .begin_request(2, false, "https://a.example", "db", 1, Some(2))
            .unwrap();
        let (second, second_events) = owner
            .begin_request(3, false, "https://a.example", "db", 1, None)
            .unwrap();
        let first = first.unwrap();
        let second = second.unwrap();
        assert_eq!(first_events.len(), 1);
        assert!(second_events.is_empty());
        assert_eq!(
            owner.status(3, second, 1).unwrap().status,
            ConnectionRequestStatus::Pending
        );

        owner.acknowledge(first, key(1, 1)).unwrap();
        owner.close(key(1, 1)).unwrap();
        assert_eq!(
            owner.status(2, first, 1).unwrap().status,
            ConnectionRequestStatus::Ready
        );

        owner.register(key(2, 2), false, "https://a.example", "db").unwrap();
        let update = owner.status(3, second, 2).unwrap();
        assert_eq!(update.status, ConnectionRequestStatus::Pending);
        assert_eq!(
            update.events.iter().map(|event| event.target).collect::<Vec<_>>(),
            vec![key(2, 2)]
        );
        assert_eq!(update.events[0].old_version, 2);
    }
}

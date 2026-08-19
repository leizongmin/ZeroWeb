//! Service Worker registration and lifecycle coordination.

use std::collections::{HashMap, HashSet};
use zero_script_sandbox::{
    SandboxConfig, ServiceWorkerEvent, ServiceWorkerLifecyclePhase, ServiceWorkerRuntime, ServiceWorkerScriptErrorKind,
};
use zero_storage::{ServiceWorkerRegistration, ServiceWorkerRegistry, ServiceWorkerState};

const DEFAULT_RUNTIME_LIMIT: usize = 32;
const MAX_URL_BYTES: usize = 64 * 1024;
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;

/// Stable registration key within one storage partition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceWorkerRegistrationKey {
    /// Serialized origin (`scheme://host:port`).
    pub origin: String,
    /// Normalized registration scope.
    pub scope: String,
}

impl ServiceWorkerRegistrationKey {
    /// Construct a registration key after basic non-empty validation.
    pub fn new(origin: &str, scope: &str) -> Result<Self, ServiceWorkerManagerError> {
        if origin.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker origin is empty".into(),
            ));
        }
        if scope.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker scope is empty".into(),
            ));
        }
        if origin.len() > MAX_URL_BYTES || scope.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker origin or scope exceeds the length limit".into(),
            ));
        }
        Ok(Self {
            origin: origin.to_string(),
            scope: scope.to_string(),
        })
    }
}

/// Observable version slots for one registration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServiceWorkerVersionSlots {
    /// Version currently evaluating or running its install event.
    pub installing: Option<u64>,
    /// Installed version waiting for activation.
    pub waiting: Option<u64>,
    /// Version currently active for this key.
    pub active: Option<u64>,
}

/// Typed manager event produced while polling worker runtimes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerManagerEvent {
    /// Top-level script evaluation completed.
    ScriptEvaluated {
        /// Registration version ID.
        registration_id: u64,
    },
    /// Evaluation or runtime execution failed.
    ScriptFailed {
        /// Registration version ID.
        registration_id: u64,
        /// Stable failure category.
        kind: ServiceWorkerScriptErrorKind,
        /// Engine diagnostic that does not contain script source.
        message: String,
    },
    /// Runtime lifecycle dispatch and all `waitUntil()` promises settled.
    LifecycleSettled {
        /// Registration version ID.
        registration_id: u64,
        /// Install or activate phase.
        phase: ServiceWorkerLifecyclePhase,
        /// Whether dispatch and all lifetime promises fulfilled.
        succeeded: bool,
        /// Rejection or dispatch error diagnostic.
        message: String,
    },
    /// Manager could not apply or dispatch an internal lifecycle transition.
    CoordinationFailed {
        /// Registration version ID.
        registration_id: u64,
        /// Internal transition diagnostic.
        message: String,
    },
    /// Install result moved the version to waiting or redundant.
    InstallCompleted {
        /// Registration version ID.
        registration_id: u64,
        /// Whether all install lifetime promises fulfilled.
        succeeded: bool,
    },
    /// Activate result moved the version to active or redundant.
    ActivationCompleted {
        /// Registration version ID.
        registration_id: u64,
        /// Whether all activate lifetime promises fulfilled.
        succeeded: bool,
    },
}

/// Service Worker manager operation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWorkerManagerError {
    /// Host input failed validation.
    InvalidInput(String),
    /// Another installing version already owns this registration key.
    JobInProgress(u64),
    /// Install completion was requested before script evaluation completed.
    EvaluationPending(u64),
    /// The manager reached its live runtime budget.
    CapacityExceeded {
        /// Maximum number of live runtimes.
        limit: usize,
    },
    /// The requested version does not exist.
    UnknownRegistration(u64),
    /// The operation does not match the version's current lifecycle state.
    InvalidState {
        /// Registration version ID.
        registration_id: u64,
        /// Required lifecycle state.
        expected: ServiceWorkerState,
        /// Actual lifecycle state.
        actual: ServiceWorkerState,
    },
    /// The script runtime could not be started or commanded.
    Runtime(String),
}

impl std::fmt::Display for ServiceWorkerManagerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::JobInProgress(id) => {
                write!(formatter, "registration {id} is already installing")
            }
            Self::EvaluationPending(id) => {
                write!(formatter, "registration {id} script evaluation is pending")
            }
            Self::CapacityExceeded { limit } => {
                write!(formatter, "Service Worker runtime limit reached ({limit})")
            }
            Self::UnknownRegistration(id) => {
                write!(formatter, "registration {id} does not exist")
            }
            Self::InvalidState {
                registration_id,
                expected,
                actual,
            } => write!(
                formatter,
                "registration {registration_id} is {actual}, expected {expected}"
            ),
            Self::Runtime(message) => write!(formatter, "runtime error: {message}"),
        }
    }
}

impl std::error::Error for ServiceWorkerManagerError {}

/// Single owner for registration slots and Service Worker runtimes.
///
/// This manager owns the storage registry instance. Callers can inspect
/// registrations but cannot mutate lifecycle state outside manager methods.
pub struct ServiceWorkerManager {
    registry: ServiceWorkerRegistry,
    slots: HashMap<ServiceWorkerRegistrationKey, ServiceWorkerVersionSlots>,
    registration_keys: HashMap<u64, ServiceWorkerRegistrationKey>,
    runtimes: HashMap<u64, ServiceWorkerRuntime>,
    evaluated: HashSet<u64>,
    runtime_limit: usize,
}

impl ServiceWorkerManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self {
            registry: ServiceWorkerRegistry::new(),
            slots: HashMap::new(),
            registration_keys: HashMap::new(),
            runtimes: HashMap::new(),
            evaluated: HashSet::new(),
            runtime_limit: DEFAULT_RUNTIME_LIMIT,
        }
    }

    /// Start evaluating one fetched script as the installing version.
    ///
    /// URL fetching and security validation remain host responsibilities in
    /// M1-2. The manager rejects overlapping jobs for the same origin/scope.
    pub fn start_evaluation(
        &mut self,
        script_url: &str,
        scope: &str,
        origin: &str,
        script: &str,
        config: SandboxConfig,
    ) -> Result<u64, ServiceWorkerManagerError> {
        if script_url.trim().is_empty() {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script URL is empty".into(),
            ));
        }
        if script_url.len() > MAX_URL_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script URL exceeds the length limit".into(),
            ));
        }
        if script.len() > MAX_SCRIPT_BYTES {
            return Err(ServiceWorkerManagerError::InvalidInput(
                "Service Worker script exceeds the size limit".into(),
            ));
        }
        let key = ServiceWorkerRegistrationKey::new(origin, scope)?;
        if let Some(id) = self.slots.get(&key).and_then(|slot| slot.installing) {
            return Err(ServiceWorkerManagerError::JobInProgress(id));
        }
        if self.runtimes.len() >= self.runtime_limit {
            return Err(ServiceWorkerManagerError::CapacityExceeded {
                limit: self.runtime_limit,
            });
        }

        let mut runtime =
            ServiceWorkerRuntime::new(config).map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;
        runtime
            .evaluate(script, script_url)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))?;

        let id = self.registry.register(script_url, scope, origin);
        self.registry.get_mut(id).expect("new registration must exist").state = ServiceWorkerState::Installing;
        self.slots.entry(key.clone()).or_default().installing = Some(id);
        self.registration_keys.insert(id, key);
        self.runtimes.insert(id, runtime);
        Ok(id)
    }

    /// Drain all currently available runtime events and apply state changes.
    pub fn poll(&mut self) -> Vec<ServiceWorkerManagerEvent> {
        let mut pending = Vec::new();
        for (&registration_id, runtime) in &self.runtimes {
            while let Some(event) = runtime.try_recv() {
                pending.push((registration_id, event));
            }
        }

        let mut output = Vec::new();
        for (registration_id, event) in pending {
            match event {
                ServiceWorkerEvent::Evaluated { .. } => {
                    self.evaluated.insert(registration_id);
                    output.push(ServiceWorkerManagerEvent::ScriptEvaluated { registration_id });
                    if let Err(error) = self.dispatch_install(registration_id) {
                        self.fail_installing_version(registration_id);
                        output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                            registration_id,
                            message: error.to_string(),
                        });
                    }
                }
                ServiceWorkerEvent::ScriptError { kind, message, .. } => {
                    self.fail_installing_version(registration_id);
                    output.push(ServiceWorkerManagerEvent::ScriptFailed {
                        registration_id,
                        kind,
                        message,
                    });
                }
                ServiceWorkerEvent::LifecycleSettled {
                    phase,
                    succeeded,
                    message,
                    ..
                } => {
                    output.push(ServiceWorkerManagerEvent::LifecycleSettled {
                        registration_id,
                        phase,
                        succeeded,
                        message,
                    });
                    let transition = match phase {
                        ServiceWorkerLifecyclePhase::Install => self.apply_install_result(registration_id, succeeded),
                        ServiceWorkerLifecyclePhase::Activate => {
                            self.apply_activation_result(registration_id, succeeded)
                        }
                    };
                    match transition {
                        Ok(completed) => {
                            output.push(completed);
                            if phase == ServiceWorkerLifecyclePhase::Install
                                && succeeded
                                && self
                                    .key_for(registration_id)
                                    .ok()
                                    .and_then(|key| self.slots.get(key))
                                    .is_some_and(|slot| slot.active.is_none())
                                && let Err(error) = self.activate_waiting(registration_id)
                            {
                                self.mark_redundant_and_stop(registration_id);
                                output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                    registration_id,
                                    message: error.to_string(),
                                });
                            }
                        }
                        Err(error) => {
                            self.mark_redundant_and_stop(registration_id);
                            output.push(ServiceWorkerManagerEvent::CoordinationFailed {
                                registration_id,
                                message: error.to_string(),
                            });
                        }
                    }
                }
                ServiceWorkerEvent::Closed => {
                    if self.is_installing(registration_id) {
                        self.fail_installing_version(registration_id);
                        output.push(ServiceWorkerManagerEvent::ScriptFailed {
                            registration_id,
                            kind: ServiceWorkerScriptErrorKind::EngineUnavailable,
                            message: "Service Worker runtime closed".into(),
                        });
                    }
                }
            }
        }
        output
    }

    /// Apply the result of the install event and its lifetime promises.
    fn apply_install_result(
        &mut self,
        registration_id: u64,
        succeeded: bool,
    ) -> Result<ServiceWorkerManagerEvent, ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installing)?;
        if !self.evaluated.remove(&registration_id) {
            return Err(ServiceWorkerManagerError::EvaluationPending(registration_id));
        }
        let key = self.key_for(registration_id)?.clone();
        let slot = self.slots.get_mut(&key).expect("registration key must have slots");
        if slot.installing != Some(registration_id) {
            return Err(ServiceWorkerManagerError::JobInProgress(
                slot.installing.unwrap_or(registration_id),
            ));
        }
        slot.installing = None;

        if succeeded {
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Installed;
            if let Some(old_waiting) = slot.waiting.replace(registration_id) {
                self.mark_redundant_and_stop(old_waiting);
            }
        } else {
            self.mark_redundant_and_stop(registration_id);
        }

        Ok(ServiceWorkerManagerEvent::InstallCompleted {
            registration_id,
            succeeded,
        })
    }

    /// Dispatch the real install event for an evaluated installing version.
    fn dispatch_install(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installing)?;
        if !self.evaluated.contains(&registration_id) {
            return Err(ServiceWorkerManagerError::EvaluationPending(registration_id));
        }
        self.runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .dispatch_install(registration_id)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    /// Move a waiting version into the activating state.
    fn begin_activation(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Installed)?;
        let key = self.key_for(registration_id)?;
        if self.slots.get(key).and_then(|slot| slot.waiting) != Some(registration_id) {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Installed,
                actual: ServiceWorkerState::Redundant,
            });
        }
        self.registry
            .get_mut(registration_id)
            .expect("validated registration must exist")
            .state = ServiceWorkerState::Activating;
        Ok(())
    }

    /// Dispatch the real activate event for an activating version.
    fn dispatch_activate(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        self.runtimes
            .get_mut(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?
            .dispatch_activate(registration_id)
            .map_err(|error| ServiceWorkerManagerError::Runtime(error.to_string()))
    }

    /// Apply the result of the activate event and its lifetime promises.
    fn apply_activation_result(
        &mut self,
        registration_id: u64,
        succeeded: bool,
    ) -> Result<ServiceWorkerManagerEvent, ServiceWorkerManagerError> {
        self.require_state(registration_id, ServiceWorkerState::Activating)?;
        let key = self.key_for(registration_id)?.clone();
        let slot = self.slots.get_mut(&key).expect("registration key must have slots");
        if slot.waiting != Some(registration_id) {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected: ServiceWorkerState::Activating,
                actual: ServiceWorkerState::Redundant,
            });
        }
        slot.waiting = None;

        if succeeded {
            if let Some(old_active) = slot.active.replace(registration_id) {
                self.mark_redundant_and_stop(old_active);
            }
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Activated;
        } else {
            self.mark_redundant_and_stop(registration_id);
        }

        Ok(ServiceWorkerManagerEvent::ActivationCompleted {
            registration_id,
            succeeded,
        })
    }

    /// Activate a waiting replacement version.
    ///
    /// The first successful installation activates automatically. A replacement
    /// remains waiting until the host determines it can activate or implements
    /// `skipWaiting()` semantics.
    pub fn activate_waiting(&mut self, registration_id: u64) -> Result<(), ServiceWorkerManagerError> {
        self.begin_activation(registration_id)?;
        if let Err(error) = self.dispatch_activate(registration_id) {
            self.registry
                .get_mut(registration_id)
                .expect("validated registration must exist")
                .state = ServiceWorkerState::Installed;
            return Err(error);
        }
        Ok(())
    }

    /// Inspect one registration version.
    pub fn registration(&self, registration_id: u64) -> Option<&ServiceWorkerRegistration> {
        self.registry.get(registration_id)
    }

    /// Inspect version slots for one origin/scope key.
    pub fn slots(&self, key: &ServiceWorkerRegistrationKey) -> Option<ServiceWorkerVersionSlots> {
        self.slots.get(key).copied()
    }

    /// Find the active registration with the longest matching scope.
    pub fn active_registration_for_url(&self, origin: &str, url: &str) -> Option<&ServiceWorkerRegistration> {
        self.slots
            .iter()
            .filter(|(key, slot)| key.origin == origin && slot.active.is_some())
            .filter_map(|(key, slot)| {
                let registration = self.registry.get(slot.active?)?;
                registration.is_in_scope(url).then_some((key.scope.len(), registration))
            })
            .max_by_key(|(scope_length, _)| *scope_length)
            .map(|(_, registration)| registration)
    }

    /// Return the number of live worker runtimes.
    pub fn runtime_count(&self) -> usize {
        self.runtimes.len()
    }

    fn key_for(&self, registration_id: u64) -> Result<&ServiceWorkerRegistrationKey, ServiceWorkerManagerError> {
        self.registration_keys
            .get(&registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))
    }

    fn require_state(
        &self,
        registration_id: u64,
        expected: ServiceWorkerState,
    ) -> Result<(), ServiceWorkerManagerError> {
        let registration = self
            .registry
            .get(registration_id)
            .ok_or(ServiceWorkerManagerError::UnknownRegistration(registration_id))?;
        if registration.state != expected {
            return Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                expected,
                actual: registration.state,
            });
        }
        Ok(())
    }

    fn is_installing(&self, registration_id: u64) -> bool {
        self.registry
            .get(registration_id)
            .is_some_and(|registration| registration.state == ServiceWorkerState::Installing)
    }

    fn fail_installing_version(&mut self, registration_id: u64) {
        self.evaluated.remove(&registration_id);
        if let Some(key) = self.registration_keys.get(&registration_id)
            && let Some(slot) = self.slots.get_mut(key)
            && slot.installing == Some(registration_id)
        {
            slot.installing = None;
        }
        self.mark_redundant_and_stop(registration_id);
    }

    fn mark_redundant_and_stop(&mut self, registration_id: u64) {
        if let Some(registration) = self.registry.get_mut(registration_id) {
            registration.mark_redundant();
        }
        if let Some(mut runtime) = self.runtimes.remove(&registration_id) {
            runtime.shutdown();
        }
    }
}

impl Default for ServiceWorkerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ServiceWorkerManager {
    fn drop(&mut self) {
        for runtime in self.runtimes.values_mut() {
            runtime.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn key(scope: &str) -> ServiceWorkerRegistrationKey {
        ServiceWorkerRegistrationKey::new("https://example.test", scope).unwrap()
    }

    fn wait_for_event(manager: &mut ServiceWorkerManager) -> ServiceWorkerManagerEvent {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = manager.poll().into_iter().next() {
                return event;
            }
            assert!(Instant::now() < deadline, "manager event timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn start(manager: &mut ServiceWorkerManager, scope: &str, script: &str) -> u64 {
        manager
            .start_evaluation(
                "https://example.test/sw.js",
                scope,
                "https://example.test",
                script,
                SandboxConfig {
                    timeout_ms: 200,
                    ..Default::default()
                },
            )
            .unwrap()
    }

    fn wait_for_state(
        manager: &mut ServiceWorkerManager,
        registration_id: u64,
        expected: ServiceWorkerState,
    ) -> Vec<ServiceWorkerManagerEvent> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut events = Vec::new();
        loop {
            events.extend(manager.poll());
            if manager
                .registration(registration_id)
                .is_some_and(|registration| registration.state == expected)
            {
                return events;
            }
            assert!(Instant::now() < deadline, "manager state timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn start_active(manager: &mut ServiceWorkerManager, scope: &str, script: &str) -> u64 {
        let id = start(manager, scope, script);
        wait_for_state(manager, id, ServiceWorkerState::Activated);
        id
    }

    #[test]
    fn successful_version_moves_through_slots() {
        let mut manager = ServiceWorkerManager::new();
        let id = start(&mut manager, "/app/", "globalThis.ready = true;");
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Installing);
        assert_eq!(manager.slots(&key("/app/")).unwrap().installing, Some(id));

        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Activated);
        assert!(events.contains(&ServiceWorkerManagerEvent::ScriptEvaluated { registration_id: id }));
        assert!(events.contains(&ServiceWorkerManagerEvent::InstallCompleted {
            registration_id: id,
            succeeded: true,
        }));
        assert!(events.contains(&ServiceWorkerManagerEvent::ActivationCompleted {
            registration_id: id,
            succeeded: true,
        }));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(manager.slots(&key("/app/")).unwrap().active, Some(id));
    }

    #[test]
    fn script_failure_clears_installing_slot() {
        let mut manager = ServiceWorkerManager::new();
        let id = start(&mut manager, "/", "function(");
        assert!(matches!(
            wait_for_event(&mut manager),
            ServiceWorkerManagerEvent::ScriptFailed {
                registration_id,
                kind: ServiceWorkerScriptErrorKind::Compile,
                ..
            } if registration_id == id
        ));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Redundant);
        assert_eq!(manager.slots(&key("/")).unwrap().installing, None);
        assert_eq!(manager.runtime_count(), 0);
    }

    #[test]
    fn lifecycle_runtime_outcomes_are_forwarded_typed() {
        let mut manager = ServiceWorkerManager::new();
        let id = start(
            &mut manager,
            "/",
            "addEventListener('install', event => {
                event.waitUntil(Promise.resolve());
            });
            addEventListener('activate', event => {
                event.waitUntil(Promise.reject(new Error('activate rejected')));
            });",
        );
        let events = wait_for_state(&mut manager, id, ServiceWorkerState::Redundant);
        assert!(events.contains(&ServiceWorkerManagerEvent::LifecycleSettled {
            registration_id: id,
            phase: ServiceWorkerLifecyclePhase::Install,
            succeeded: true,
            message: String::new(),
        }));
        assert!(matches!(
            events.iter().find(|event| matches!(
                event,
                ServiceWorkerManagerEvent::LifecycleSettled {
                    phase: ServiceWorkerLifecyclePhase::Activate,
                    ..
                }
            )),
            Some(ServiceWorkerManagerEvent::LifecycleSettled {
                registration_id,
                phase: ServiceWorkerLifecyclePhase::Activate,
                succeeded: false,
                message,
            }) if *registration_id == id && message.contains("activate rejected")
        ));
    }

    #[test]
    fn install_failure_preserves_existing_active() {
        let mut manager = ServiceWorkerManager::new();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");

        let second = start(
            &mut manager,
            "/",
            "addEventListener('install', event => {
                event.waitUntil(Promise.reject(new Error('install rejected')));
            });",
        );
        wait_for_state(&mut manager, second, ServiceWorkerState::Redundant);

        let slots = manager.slots(&key("/")).unwrap();
        assert_eq!(slots.active, Some(first));
        assert_eq!(slots.installing, None);
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Activated
        );
        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Redundant
        );
    }

    #[test]
    fn activation_failure_preserves_existing_active() {
        let mut manager = ServiceWorkerManager::new();
        let first = start_active(&mut manager, "/", "globalThis.version = 1;");

        let second = start(
            &mut manager,
            "/",
            "addEventListener('activate', event => {
                event.waitUntil(Promise.reject(new Error('activate rejected')));
            });",
        );
        wait_for_state(&mut manager, second, ServiceWorkerState::Installed);
        manager.activate_waiting(second).unwrap();
        wait_for_state(&mut manager, second, ServiceWorkerState::Redundant);

        let slots = manager.slots(&key("/")).unwrap();
        assert_eq!(slots.active, Some(first));
        assert_eq!(slots.waiting, None);
        assert_eq!(
            manager.registration(first).unwrap().state,
            ServiceWorkerState::Activated
        );
        assert_eq!(
            manager.registration(second).unwrap().state,
            ServiceWorkerState::Redundant
        );
    }

    #[test]
    fn activation_replaces_only_same_scope() {
        let mut manager = ServiceWorkerManager::new();
        let root = start_active(&mut manager, "/", "globalThis.scope = 'root';");
        let app = start_active(&mut manager, "/app/", "globalThis.scope = 'app';");

        let replacement = start(&mut manager, "/app/", "globalThis.scope = 'app-v2';");
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Installed);
        manager.activate_waiting(replacement).unwrap();
        wait_for_state(&mut manager, replacement, ServiceWorkerState::Activated);

        assert_eq!(manager.slots(&key("/")).unwrap().active, Some(root));
        assert_eq!(manager.slots(&key("/app/")).unwrap().active, Some(replacement));
        assert_eq!(
            manager
                .active_registration_for_url("https://example.test", "https://example.test/app/page.html",)
                .unwrap()
                .id,
            replacement
        );
        assert_eq!(
            manager
                .active_registration_for_url("https://example.test", "https://example.test/other/page.html",)
                .unwrap()
                .id,
            root
        );
        assert!(
            manager
                .active_registration_for_url("https://other.test", "https://other.test/app/page.html",)
                .is_none()
        );
        assert_eq!(manager.registration(root).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(manager.registration(app).unwrap().state, ServiceWorkerState::Redundant);
    }

    #[test]
    fn overlapping_job_for_same_key_is_rejected() {
        let mut manager = ServiceWorkerManager::new();
        let first = start(&mut manager, "/", "void 0;");
        let result = manager.start_evaluation(
            "https://example.test/sw-v2.js",
            "/",
            "https://example.test",
            "void 0;",
            SandboxConfig::default(),
        );
        assert_eq!(result, Err(ServiceWorkerManagerError::JobInProgress(first)));
    }

    #[test]
    fn different_scope_jobs_can_evaluate_concurrently() {
        let mut manager = ServiceWorkerManager::new();
        let root = start(&mut manager, "/", "globalThis.scope = 'root';");
        let app = start(&mut manager, "/app/", "globalThis.scope = 'app';");
        assert_ne!(root, app);
        assert_eq!(manager.runtime_count(), 2);
        assert_eq!(manager.slots(&key("/")).unwrap().installing, Some(root));
        assert_eq!(manager.slots(&key("/app/")).unwrap().installing, Some(app));

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut evaluated = HashSet::new();
        while evaluated.len() < 2 {
            for event in manager.poll() {
                if let ServiceWorkerManagerEvent::ScriptEvaluated { registration_id } = event {
                    evaluated.insert(registration_id);
                }
            }
            assert!(Instant::now() < deadline, "parallel manager events timed out");
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(evaluated, HashSet::from([root, app]));
    }

    #[test]
    fn waiting_activation_rejects_installing_version() {
        let mut manager = ServiceWorkerManager::new();
        let id = start(&mut manager, "/", "void 0;");
        assert!(matches!(
            manager.activate_waiting(id),
            Err(ServiceWorkerManagerError::InvalidState {
                registration_id,
                actual: ServiceWorkerState::Installing,
                ..
            }) if registration_id == id
        ));
        assert_eq!(manager.registration(id).unwrap().state, ServiceWorkerState::Installing);
    }

    #[test]
    fn capacity_rejection_does_not_create_registration() {
        let mut manager = ServiceWorkerManager::new();
        manager.runtime_limit = 1;
        let first = start(&mut manager, "/", "void 0;");
        let result = manager.start_evaluation(
            "https://example.test/app-sw.js",
            "/app/",
            "https://example.test",
            "void 0;",
            SandboxConfig::default(),
        );
        assert_eq!(result, Err(ServiceWorkerManagerError::CapacityExceeded { limit: 1 }));
        assert_eq!(manager.runtime_count(), 1);
        assert!(manager.slots(&key("/app/")).is_none());
        assert!(manager.registration(first + 1).is_none());
    }

    #[test]
    fn oversized_input_is_rejected_before_runtime_creation() {
        let mut manager = ServiceWorkerManager::new();
        let oversized_url = "x".repeat(MAX_URL_BYTES + 1);
        assert!(matches!(
            manager.start_evaluation(
                &oversized_url,
                "/",
                "https://example.test",
                "void 0;",
                SandboxConfig::default(),
            ),
            Err(ServiceWorkerManagerError::InvalidInput(_))
        ));

        let oversized_script = "x".repeat(MAX_SCRIPT_BYTES + 1);
        assert!(matches!(
            manager.start_evaluation(
                "https://example.test/sw.js",
                "/",
                "https://example.test",
                &oversized_script,
                SandboxConfig::default(),
            ),
            Err(ServiceWorkerManagerError::InvalidInput(_))
        ));
        assert_eq!(manager.runtime_count(), 0);
        assert!(manager.slots(&key("/")).is_none());
        assert!(manager.registration(0).is_none());
    }
}

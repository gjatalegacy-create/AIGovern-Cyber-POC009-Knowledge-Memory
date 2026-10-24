//! Source-derived scoped projection; POC008 supplies a separate, limited WAL adapter.
//! No epistemic verdict, knowledge write, authority token or separate persistence.
//! The POC008 Store publishes this projection only after its durable append.
use crate::pd_memory::{Hash, PdIdentity, PdOperationRecord, PdOperationRequest};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Default)]
pub(crate) struct PdProjection {
    heads: HashMap<Hash, Arc<PdOperationRecord>>,
    // Scoped revision references to the same Arc-owned Vault records. No payload copy.
    revisions: HashMap<(Hash, u64), Arc<PdOperationRecord>>,
    operations: HashMap<(Hash, String), Arc<PdOperationRecord>>,
    candidates: HashMap<(Hash, Hash), Arc<PdOperationRecord>>,
}

pub(crate) enum PdCommitPlan {
    Existing(Arc<PdOperationRecord>),
    Append(PdAppend),
}

pub(crate) struct PdAppend {
    key: Hash,
    operation_key: (Hash, String),
    revision_key: (Hash, u64),
    candidate_key: Option<(Hash, Hash)>,
    record: Arc<PdOperationRecord>,
}

impl PdAppend {
    pub(crate) fn record(&self) -> &PdOperationRecord {
        &self.record
    }
}

fn operation_valid(operation_id: &str) -> Result<(), String> {
    match operation_id.trim().is_empty()
        || operation_id.len() > 1024
        || operation_id.chars().any(|value| value == '\0')
    {
        true => Err("PD_OPERATION_ID_INVALID".into()),
        false => Ok(()),
    }
}

impl PdProjection {
    pub(crate) fn head(&self, identity: &PdIdentity) -> Result<Option<PdOperationRecord>, String> {
        self.clone_scoped(identity, self.heads.get(&identity.key()?))
    }

    /// Stable cognitive sessions for operational PD are exact identity hashes.
    /// Non-operational legacy native sessions have no matching projection key.
    pub(crate) fn operation(
        &self,
        identity: &PdIdentity,
        operation_id: &str,
    ) -> Result<Option<PdOperationRecord>, String> {
        operation_valid(operation_id)?;
        self.clone_scoped(
            identity,
            self.operations
                .get(&(identity.key()?, operation_id.to_string())),
        )
    }

    fn clone_scoped(
        &self,
        identity: &PdIdentity,
        record: Option<&Arc<PdOperationRecord>>,
    ) -> Result<Option<PdOperationRecord>, String> {
        match record {
            Some(record) if &record.request.identity == identity => {
                Ok(Some(record.as_ref().clone()))
            }
            Some(_) => Err("PD_IDENTITY_COLLISION".into()),
            None => Ok(None),
        }
    }

    /// Caller holds the exclusive Store file lock and mutable projection through durable append.
    pub(crate) fn prepare(&mut self, request: PdOperationRequest) -> Result<PdCommitPlan, String> {
        request.validate()?;
        let key = request.identity.key()?;
        let operation_key = (key, request.operation_id.clone());
        let request_digest = request.request_digest()?;
        // Deliberately precedes CAS: an exact retry remains historical success
        // after another operation has advanced the same head.
        match self.operations.get(&operation_key) {
            Some(prior) => {
                return match prior.request_digest == request_digest && prior.request == request {
                    true => Ok(PdCommitPlan::Existing(Arc::clone(prior))),
                    false => Err("PD_OPERATION_CONFLICT".into()),
                };
            }
            None => {}
        }
        let previous_record = match self.heads.get(&key) {
            Some(head) => {
                match head.request.identity == request.identity
                    && request.expected_revision == head.revision
                    && request.previous_state == head.state_digest
                {
                    true => head.record_id,
                    false => return Err("PD_SCOPED_CAS_CONFLICT".into()),
                }
            }
            None => match request.expected_revision == 0 && request.previous_state == [0; 32] {
                true => [0; 32],
                false => return Err("PD_SCOPED_CAS_MISSING_HEAD".into()),
            },
        };
        let record = Arc::new(PdOperationRecord::construct(request, previous_record)?);
        let candidate_key = match record.request.candidate.is_empty() {
            true => None,
            false => Some((key, record.request.candidate_key)),
        };
        let revision_key = (key, record.revision);
        if self.revisions.contains_key(&revision_key) {
            return Err("PD_HISTORY_DUPLICATE_REVISION".into());
        }
        self.revisions
            .try_reserve(1)
            .map_err(|_| "PD_HISTORY_INDEX_ALLOCATION")?;
        // Reserve map growth before append. Publication only inserts already
        // allocated keys and Arc references; it does not clone checkpoint bytes.
        self.heads
            .try_reserve(1)
            .map_err(|_| "PD_HEAD_ALLOCATION")?;
        self.operations
            .try_reserve(1)
            .map_err(|_| "PD_OPERATION_ALLOCATION")?;
        match candidate_key {
            Some(_) => self
                .candidates
                .try_reserve(1)
                .map_err(|_| "PD_CANDIDATE_ALLOCATION")?,
            None => {}
        }
        Ok(PdCommitPlan::Append(PdAppend {
            key,
            operation_key,
            revision_key,
            candidate_key,
            record,
        }))
    }

    pub(crate) fn publish(&mut self, append: PdAppend) -> Arc<PdOperationRecord> {
        let PdAppend {
            key,
            operation_key,
            revision_key,
            candidate_key,
            record,
        } = append;
        self.heads.insert(key, Arc::clone(&record));
        self.operations.insert(operation_key, Arc::clone(&record));
        self.revisions.insert(revision_key, Arc::clone(&record));
        match candidate_key {
            Some(key) => {
                self.candidates.insert(key, Arc::clone(&record));
            }
            None => {}
        }
        record
    }

    /// The record's internal hash is necessary but not sufficient: replay also
    /// rechecks scope, sequence, previous state and exact previous-record chain.
    pub(crate) fn replay(&mut self, record: PdOperationRecord) -> Result<(), String> {
        record.validate()?;
        match self.prepare(record.request.clone())? {
            PdCommitPlan::Existing(prior) => match prior.as_ref() == &record {
                true => Ok(()),
                false => Err("PD_REPLAY_OPERATION_CONFLICT".into()),
            },
            PdCommitPlan::Append(append) => match append.record() == &record {
                true => {
                    self.publish(append);
                    Ok(())
                }
                false => Err("PD_REPLAY_CHAIN_CONFLICT".into()),
            },
        }
    }
}

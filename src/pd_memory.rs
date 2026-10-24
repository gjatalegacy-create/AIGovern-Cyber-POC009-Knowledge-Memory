//! AIGovern-Cyber operational PD memory contract for validated request and record continuity.
//! Public data and bounded codecs only. These types cannot issue a verdict,
//! write a Vault, restore a capability, or certify a reported authority refusal.
use sha2::{Digest, Sha256};
pub const PD_MEMORY_VERSION: u16 = 1;
pub const MAX_CHECKPOINT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CANDIDATE_BYTES: usize = 64 * 1024;
pub const MAX_SOURCE_BYTES: usize = 256 * 1024;
pub const MAX_REQUEST_BYTES: usize =
    MAX_CHECKPOINT_BYTES + MAX_CANDIDATE_BYTES + MAX_SOURCE_BYTES + 256 * 1024;
pub const MAX_TEXT_BYTES: usize = 8192;
pub type Hash = [u8; 32];

fn hash(parts: &[&[u8]]) -> Hash {
    let mut h = Sha256::new();
    for part in parts {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part);
    }
    h.finalize().into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdIdentity {
    pub owner: String,
    pub obligation: String,
    pub case_id: String,
    pub scope: String,
    pub domain: String,
    pub lineage_root: String,
}
impl PdIdentity {
    pub fn validate(&self) -> Result<(), String> {
        for s in [
            &self.owner,
            &self.obligation,
            &self.case_id,
            &self.scope,
            &self.domain,
            &self.lineage_root,
        ] {
            text_valid(s, 1024)?;
        }
        Ok(())
    }
    pub fn key(&self) -> Result<Hash, String> {
        self.validate()?;
        Ok(hash(&[
            b"ESSMAI/PD/IDENTITY/1",
            self.owner.as_bytes(),
            self.obligation.as_bytes(),
            self.case_id.as_bytes(),
            self.scope.as_bytes(),
            self.domain.as_bytes(),
            self.lineage_root.as_bytes(),
        ]))
    }
}

/// A report about a predecessor, never a newly issued authority receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportClass {
    MissingConditions = 1,
    AuthorityRefused = 2,
    TransportUncertain = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextAction {
    AcquireScopedEvidence = 1,
    ExplainConditions = 2,
    ResolveCommitStatus = 3,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionWitness {
    pub predecessor: String,
    pub parent_authority: String,
    pub failed_action: String,
    pub condition_code: String,
    pub reason: String,
    pub source_component: String,
    pub source_sha256: Hash,
    pub attempt_trace: String,
    pub input_sha256: Hash,
    pub lgc_parent: Hash,
    pub report_class: ReportClass,
    pub permitted_next: NextAction,
}
impl ConditionWitness {
    pub fn validate(&self) -> Result<(), String> {
        for s in [
            &self.predecessor,
            &self.parent_authority,
            &self.failed_action,
            &self.condition_code,
            &self.reason,
            &self.source_component,
            &self.attempt_trace,
        ] {
            text_valid(s, MAX_TEXT_BYTES)?;
        }
        if self.source_sha256 == [0; 32]
            || self.input_sha256 == [0; 32]
            || self.lgc_parent == [0; 32]
        {
            return Err("PD_WITNESS_UNBOUND".into());
        }
        let allowed = matches!(
            (self.report_class, self.permitted_next),
            (
                ReportClass::MissingConditions,
                NextAction::AcquireScopedEvidence
            ) | (
                ReportClass::MissingConditions,
                NextAction::ExplainConditions
            ) | (ReportClass::AuthorityRefused, NextAction::ExplainConditions)
                | (
                    ReportClass::TransportUncertain,
                    NextAction::ResolveCommitStatus
                )
        );
        if !allowed {
            return Err("PD_NEXT_ACTION_OUTSIDE_REPORTED_SCOPE".into());
        }
        Ok(())
    }
}

/// Full bounded action material is retained. Digests alone are not a transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LgcStep {
    pub sequence: u32,
    pub predecessor: Hash,
    pub site: String,
    pub action: String,
    pub payload_digest: Hash,
    pub step_digest: Hash,
}
impl LgcStep {
    pub fn new(
        sequence: u32,
        predecessor: Hash,
        site: &str,
        action: &str,
        payload_digest: Hash,
    ) -> Result<Self, String> {
        text_valid(site, 1024)?;
        text_valid(action, 1024)?;
        if predecessor == [0; 32] || payload_digest == [0; 32] {
            return Err("PD_LGC_UNBOUND".into());
        }
        let step_digest = hash(&[
            b"ESSMAI/PD/LGC-STEP/1",
            &sequence.to_le_bytes(),
            &predecessor,
            site.as_bytes(),
            action.as_bytes(),
            &payload_digest,
        ]);
        Ok(Self {
            sequence,
            predecessor,
            site: site.into(),
            action: action.into(),
            payload_digest,
            step_digest,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if &Self::new(
            self.sequence,
            self.predecessor,
            &self.site,
            &self.action,
            self.payload_digest,
        )? != self
        {
            return Err("PD_LGC_INTEGRITY".into());
        }
        Ok(())
    }
}

/// No verdict fields exist here. Candidate bytes remain unadmitted material.
/// Caller authorization/admission is a Shadow responsibility, not a shape check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdOperationRequest {
    pub identity: PdIdentity,
    pub operation_id: String,
    pub source_input: Vec<u8>,
    pub expected_revision: u64,
    pub previous_state: Hash,
    pub witness: ConditionWitness,
    pub checkpoint: Vec<u8>,
    pub scoped_question: String,
    pub candidate: Vec<u8>,
    pub candidate_key: Hash,
    pub lgc_steps: Vec<LgcStep>,
}
#[cfg(test)]
thread_local! {
    static REQUEST_VALIDATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl PdOperationRequest {
    pub fn validate(&self) -> Result<(), String> {
        #[cfg(test)]
        REQUEST_VALIDATIONS.with(|count| count.set(count.get() + 1));
        self.identity.validate()?;
        text_valid(&self.operation_id, 1024)?;
        let (source_identity, source_operation) = match self.source_input.get(..4) {
            Some(b"PDC1") => {
                let input = decode_continuity_input(&self.source_input)?;
                (input.identity, input.operation_id)
            }
            _ => return Err("PD_SOURCE_INPUT_VERSION".into()),
        };
        let source_digest: Hash = Sha256::digest(&self.source_input).into();
        if source_identity != self.identity
            || source_operation != self.operation_id
            || source_digest != self.witness.input_sha256
        {
            return Err("PD_SOURCE_INPUT_BINDING".into());
        }
        self.witness.validate()?;
        text_valid(&self.scoped_question, MAX_TEXT_BYTES)?;
        if self.checkpoint.is_empty()
            || self.checkpoint.len() > MAX_CHECKPOINT_BYTES
            || self.candidate.len() > MAX_CANDIDATE_BYTES
        {
            return Err("PD_PAYLOAD_LIMIT".into());
        }
        if (self.expected_revision == 0) != (self.previous_state == [0; 32]) {
            return Err("PD_REVISION_STATE_MISMATCH".into());
        }

        if self.candidate.is_empty() != (self.candidate_key == [0; 32]) {
            return Err("PD_CANDIDATE_KEY_MISMATCH".into());
        }
        if self.lgc_steps.is_empty() || self.lgc_steps.len() > 32 {
            return Err("PD_LGC_LIMIT".into());
        }
        let mut parent = self.witness.input_sha256;
        for (index, step) in self.lgc_steps.iter().enumerate() {
            step.validate()?;
            if step.sequence != index as u32 || step.predecessor != parent {
                return Err("PD_LGC_ORDER".into());
            }
            parent = step.step_digest;
        }
        if parent != self.witness.lgc_parent {
            return Err("PD_LGC_TIP_MISMATCH".into());
        }
        self.expected_revision
            .checked_add(1)
            .ok_or("PD_REVISION_OVERFLOW")?;
        Ok(())
    }
    pub fn request_digest(&self) -> Result<Hash, String> {
        Ok(hash(&[b"ESSMAI/PD/REQUEST/1", &encode_request(self)?]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdOperationRecord {
    pub request: PdOperationRequest,
    pub request_digest: Hash,
    pub revision: u64,
    pub previous_record: Hash,
    pub state_digest: Hash,
    pub record_id: Hash,
}
// Call-local derivation only: this is not a stored validation flag or authority.
// Its caller has validated the exact immutable request in the current call.
struct RecordDerivation {
    request_digest: Hash,
    revision: u64,
    state_digest: Hash,
    record_id: Hash,
    encoded_request: Vec<u8>,
}
impl RecordDerivation {
    fn from_validated_request(
        request: &PdOperationRequest,
        previous_record: Hash,
    ) -> Result<Self, String> {
        if (request.expected_revision == 0) != (previous_record == [0; 32]) {
            return Err("PD_PREVIOUS_RECORD_MISMATCH".into());
        }
        let encoded_request = encode_validated_request(request)?;
        let request_digest = hash(&[b"ESSMAI/PD/REQUEST/1", &encoded_request]);
        let revision = request
            .expected_revision
            .checked_add(1)
            .ok_or("PD_REVISION_OVERFLOW")?;
        let key = request.identity.key()?;
        let state_digest = hash(&[
            b"ESSMAI/PD/STATE/1",
            &key,
            &revision.to_le_bytes(),
            &request.previous_state,
            &request_digest,
        ]);
        let record_id = hash(&[
            b"ESSMAI/PD/RECORD/1",
            &request_digest,
            &revision.to_le_bytes(),
            &previous_record,
            &state_digest,
        ]);
        Ok(Self {
            request_digest,
            revision,
            state_digest,
            record_id,
            encoded_request,
        })
    }
}
impl PdOperationRecord {
    /// Storage owner calls this only after admission and CAS. Public construction
    /// is not authorization; replay integrity is not an authority signature.
    pub fn construct(request: PdOperationRequest, previous_record: Hash) -> Result<Self, String> {
        request.validate()?;
        let derived = RecordDerivation::from_validated_request(&request, previous_record)?;
        Ok(Self {
            request,
            request_digest: derived.request_digest,
            revision: derived.revision,
            previous_record,
            state_digest: derived.state_digest,
            record_id: derived.record_id,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        self.validated_request_encoding().map(|_| ())
    }
    fn validated_request_encoding(&self) -> Result<Vec<u8>, String> {
        self.request.validate()?;
        self.request_encoding_after_validation()
    }
    // Also used immediately after decode_request validated this exact request.
    // No mutation or externally supplied validation result crosses this boundary.
    fn request_encoding_after_validation(&self) -> Result<Vec<u8>, String> {
        let expected =
            RecordDerivation::from_validated_request(&self.request, self.previous_record)?;
        if self.request_digest != expected.request_digest
            || self.revision != expected.revision
            || self.state_digest != expected.state_digest
            || self.record_id != expected.record_id
        {
            return Err("PD_RECORD_INTEGRITY".into());
        }
        Ok(expected.encoded_request)
    }
    /// A binding of this stored question and record, never a predicted future user i0.
    /// The historical hash domain stays unchanged for byte compatibility.
    pub fn operational_question_binding(&self) -> String {
        hex(&hash(&[
            b"ESSMAI/PD/OPERATIONAL-NEXT-I0/1",
            &self.record_id,
            &self.state_digest,
            self.request.scoped_question.as_bytes(),
        ]))
    }
    /// Historical API alias. This value identifies a question; it is not actual user input.
    pub fn operational_next_i0(&self) -> String {
        self.operational_question_binding()
    }
}
pub fn hex(bytes: &Hash) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn parse_hash(s: &str) -> Result<Hash, String> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("PD_HASH_ENCODING".into());
    }
    let mut out = [0; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| "PD_HASH_ENCODING")?;
    }
    Ok(out)
}
fn text_valid(s: &str, limit: usize) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > limit || s.chars().any(|c| c == '\0') {
        return Err("PD_TEXT_LIMIT_OR_EMPTY".into());
    }
    Ok(())
}
fn put_bytes(out: &mut Vec<u8>, value: &[u8], max: usize) -> Result<(), String> {
    if value.len() > max {
        return Err("PD_LENGTH_LIMIT".into());
    }
    let n = u32::try_from(value.len()).map_err(|_| "PD_LENGTH_OVERFLOW")?;
    out.extend_from_slice(&n.to_le_bytes());
    out.extend_from_slice(value);
    Ok(())
}
fn put_text(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    put_bytes(out, value.as_bytes(), MAX_TEXT_BYTES)
}
fn identity_out(out: &mut Vec<u8>, identity: &PdIdentity) -> Result<(), String> {
    for s in [
        &identity.owner,
        &identity.obligation,
        &identity.case_id,
        &identity.scope,
        &identity.domain,
        &identity.lineage_root,
    ] {
        put_text(out, s)?;
    }
    Ok(())
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).ok_or("PD_DECODE_OVERFLOW")?;
        let bytes = self.bytes.get(self.pos..end).ok_or("PD_TRUNCATED")?;
        self.pos = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| "PD_TRUNCATED")?,
        ))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| "PD_TRUNCATED")?,
        ))
    }
    fn hash(&mut self) -> Result<Hash, String> {
        self.take(32)?.try_into().map_err(|_| "PD_TRUNCATED".into())
    }
    fn bytes(&mut self, max: usize) -> Result<Vec<u8>, String> {
        let n = u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| "PD_TRUNCATED")?) as usize;
        if n > max {
            return Err("PD_LENGTH_LIMIT".into());
        }
        let raw = self.take(n)?;
        let mut v = Vec::new();
        v.try_reserve_exact(n).map_err(|_| "PD_ALLOCATION")?;
        v.extend_from_slice(raw);
        Ok(v)
    }
    fn text(&mut self) -> Result<String, String> {
        String::from_utf8(self.bytes(MAX_TEXT_BYTES)?).map_err(|_| "PD_UTF8".into())
    }
    fn end(&self) -> Result<(), String> {
        if self.pos != self.bytes.len() {
            Err("PD_TRAILING_BYTES".into())
        } else {
            Ok(())
        }
    }
}
pub fn encode_identity(identity: &PdIdentity) -> Result<Vec<u8>, String> {
    identity.validate()?;
    let mut out = b"PDI1".to_vec();
    identity_out(&mut out, identity)?;
    Ok(out)
}
pub fn decode_identity(bytes: &[u8]) -> Result<PdIdentity, String> {
    if bytes.len() > 8192 {
        return Err("PD_IDENTITY_LIMIT".into());
    }
    let mut r = Reader { bytes, pos: 0 };
    if r.take(4)? != b"PDI1" {
        return Err("PD_IDENTITY_MAGIC".into());
    }
    let identity = identity_in(&mut r)?;
    r.end()?;
    identity.validate()?;
    Ok(identity)
}
fn identity_in(r: &mut Reader<'_>) -> Result<PdIdentity, String> {
    Ok(PdIdentity {
        owner: r.text()?,
        obligation: r.text()?,
        case_id: r.text()?,
        scope: r.text()?,
        domain: r.text()?,
        lineage_root: r.text()?,
    })
}
pub fn encode_request(q: &PdOperationRequest) -> Result<Vec<u8>, String> {
    q.validate()?;
    encode_validated_request(q)
}
// Private canonical encoder for a request validated in this same call.
fn encode_validated_request(q: &PdOperationRequest) -> Result<Vec<u8>, String> {
    let mut out = b"PDQ1".to_vec();
    out.extend_from_slice(&PD_MEMORY_VERSION.to_le_bytes());
    identity_out(&mut out, &q.identity)?;
    put_text(&mut out, &q.operation_id)?;
    put_bytes(&mut out, &q.source_input, MAX_SOURCE_BYTES)?;
    out.extend_from_slice(&q.expected_revision.to_le_bytes());
    out.extend_from_slice(&q.previous_state);
    let w = &q.witness;
    for s in [
        &w.predecessor,
        &w.parent_authority,
        &w.failed_action,
        &w.condition_code,
        &w.reason,
        &w.source_component,
        &w.attempt_trace,
    ] {
        put_text(&mut out, s)?;
    }
    for h in [&w.source_sha256, &w.input_sha256, &w.lgc_parent] {
        out.extend_from_slice(h);
    }
    out.push(w.report_class as u8);
    out.push(w.permitted_next as u8);
    put_bytes(&mut out, &q.checkpoint, MAX_CHECKPOINT_BYTES)?;
    put_text(&mut out, &q.scoped_question)?;
    put_bytes(&mut out, &q.candidate, MAX_CANDIDATE_BYTES)?;
    out.extend_from_slice(&q.candidate_key);
    out.push(q.lgc_steps.len() as u8);
    for step in &q.lgc_steps {
        out.extend_from_slice(&step.sequence.to_le_bytes());
        out.extend_from_slice(&step.predecessor);
        put_text(&mut out, &step.site)?;
        put_text(&mut out, &step.action)?;
        out.extend_from_slice(&step.payload_digest);
        out.extend_from_slice(&step.step_digest);
    }
    if out.len() > MAX_REQUEST_BYTES {
        return Err("PD_REQUEST_LIMIT".into());
    }
    Ok(out)
}
pub fn decode_request(bytes: &[u8]) -> Result<PdOperationRequest, String> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err("PD_REQUEST_LIMIT".into());
    }
    let mut r = Reader { bytes, pos: 0 };
    if r.take(4)? != b"PDQ1" || r.u16()? != PD_MEMORY_VERSION {
        return Err("PD_REQUEST_VERSION".into());
    }
    let identity = identity_in(&mut r)?;
    let operation_id = r.text()?;
    let source_input = r.bytes(MAX_SOURCE_BYTES)?;
    let expected_revision = r.u64()?;
    let previous_state = r.hash()?;
    let predecessor = r.text()?;
    let parent_authority = r.text()?;
    let failed_action = r.text()?;
    let condition_code = r.text()?;
    let reason = r.text()?;
    let source_component = r.text()?;
    let attempt_trace = r.text()?;
    let source_sha256 = r.hash()?;
    let input_sha256 = r.hash()?;
    let lgc_parent = r.hash()?;
    let report_class = match r.u8()? {
        1 => ReportClass::MissingConditions,
        2 => ReportClass::AuthorityRefused,
        3 => ReportClass::TransportUncertain,
        _ => return Err("PD_REPORT_CLASS".into()),
    };
    let permitted_next = match r.u8()? {
        1 => NextAction::AcquireScopedEvidence,
        2 => NextAction::ExplainConditions,
        3 => NextAction::ResolveCommitStatus,
        _ => return Err("PD_NEXT_ACTION".into()),
    };
    let checkpoint = r.bytes(MAX_CHECKPOINT_BYTES)?;
    let scoped_question = r.text()?;
    let candidate = r.bytes(MAX_CANDIDATE_BYTES)?;
    let candidate_key = r.hash()?;
    let count = r.u8()? as usize;
    if count == 0 || count > 32 {
        return Err("PD_LGC_LIMIT".into());
    }
    let mut lgc_steps = Vec::new();
    for _ in 0..count {
        lgc_steps.push(LgcStep {
            sequence: u32::from_le_bytes(r.take(4)?.try_into().map_err(|_| "PD_TRUNCATED")?),
            predecessor: r.hash()?,
            site: r.text()?,
            action: r.text()?,
            payload_digest: r.hash()?,
            step_digest: r.hash()?,
        });
    }
    r.end()?;
    let q = PdOperationRequest {
        identity,
        operation_id,
        source_input,
        expected_revision,
        previous_state,
        witness: ConditionWitness {
            predecessor,
            parent_authority,
            failed_action,
            condition_code,
            reason,
            source_component,
            source_sha256,
            attempt_trace,
            input_sha256,
            lgc_parent,
            report_class,
            permitted_next,
        },
        checkpoint,
        scoped_question,
        candidate,
        candidate_key,
        lgc_steps,
    };
    q.validate()?;
    Ok(q)
}
pub fn encode_record(record: &PdOperationRecord) -> Result<Vec<u8>, String> {
    let request = record.validated_request_encoding()?;
    let mut out = b"PDR1".to_vec();
    put_bytes(&mut out, &request, MAX_REQUEST_BYTES)?;
    out.extend_from_slice(&record.request_digest);
    out.extend_from_slice(&record.revision.to_le_bytes());
    for h in [
        &record.previous_record,
        &record.state_digest,
        &record.record_id,
    ] {
        out.extend_from_slice(h);
    }
    Ok(out)
}
pub fn decode_record(bytes: &[u8]) -> Result<PdOperationRecord, String> {
    if bytes.len() > MAX_REQUEST_BYTES + 256 {
        return Err("PD_RECORD_LIMIT".into());
    }
    let mut r = Reader { bytes, pos: 0 };
    if r.take(4)? != b"PDR1" {
        return Err("PD_RECORD_MAGIC".into());
    }
    let request = decode_request(&r.bytes(MAX_REQUEST_BYTES)?)?;
    let record = PdOperationRecord {
        request,
        request_digest: r.hash()?,
        revision: r.u64()?,
        previous_record: r.hash()?,
        state_digest: r.hash()?,
        record_id: r.hash()?,
    };
    r.end()?;
    record.request_encoding_after_validation()?;
    Ok(record)
}
/// Explicit new operational input, not a GCL verdict request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuityInput {
    pub identity: PdIdentity,
    pub operation_id: String,
    pub user_text: String,
    pub science_query: Vec<u8>,
}
pub fn encode_continuity_input(q: &ContinuityInput) -> Result<Vec<u8>, String> {
    q.identity.validate()?;
    text_valid(&q.operation_id, 1024)?;
    text_valid(&q.user_text, MAX_TEXT_BYTES)?;
    if q.science_query.is_empty() {
        return Err("PD_SCIENCE_QUERY_MISSING".into());
    }
    let mut b = b"PDC1".to_vec();
    identity_out(&mut b, &q.identity)?;
    put_text(&mut b, &q.operation_id)?;
    put_text(&mut b, &q.user_text)?;
    put_bytes(&mut b, &q.science_query, 16 * 1024)?;
    Ok(b)
}
pub fn decode_continuity_input(bytes: &[u8]) -> Result<ContinuityInput, String> {
    if bytes.len() > 40 * 1024 {
        return Err("PD_CONTINUITY_LIMIT".into());
    }
    let mut r = Reader { bytes, pos: 0 };
    if r.take(4)? != b"PDC1" {
        return Err("PD_CONTINUITY_VERSION".into());
    }
    let q = ContinuityInput {
        identity: identity_in(&mut r)?,
        operation_id: r.text()?,
        user_text: r.text()?,
        science_query: r.bytes(16 * 1024)?,
    };
    r.end()?;
    if encode_continuity_input(&q)? != bytes {
        return Err("PD_CONTINUITY_CANONICAL".into());
    }
    Ok(q)
}

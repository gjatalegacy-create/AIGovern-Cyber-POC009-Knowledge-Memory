//! Standalone POC adapter. This is not the native KnowledgeVault backend.
use crate::framing::frame_crc;
use crate::knowledge::{self, KnowledgeIndex};
use crate::pd_memory::*;
use crate::progressive_debatic::{core::ProgressiveDebatic, types::DebateMode};
use crate::projection::{PdCommitPlan, PdProjection};
use crate::zero_noi::trigger;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

const MAGIC: &[u8; 8] = b"OPL2SPN9";
const USER_ONLY: &[u8] = b"PD_USER_ONLY_V2\0";
const MAX_RECORD: usize = MAX_REQUEST_BYTES + 256;

fn sha(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
}
fn io(error: std::io::Error) -> String {
    format!("POC_IO:{error}")
}

pub fn input(case_id:&str, operation_id:&str, user_text:&str, tick:u64, science:serde_json::Value)->Result<ContinuityInput,String> {
    let raw=serde_json::to_vec(&serde_json::json!({"case":case_id,"operation":operation_id,"text":user_text,"tick":tick,"science":science})).map_err(|e|e.to_string())?;
    Ok(knowledge::decode_input(&raw)?.0)
}

fn fresh_engine(
    identity: &PdIdentity,
    head: Option<&PdOperationRecord>,
    tick: u64,
) -> Result<ProgressiveDebatic, String> {
    let session = hex(&identity.key()?);
    let mut pd = ProgressiveDebatic::new();
    match head {
        Some(record) => {
            if record.request.identity != *identity {
                return Err("POC_HEAD_SCOPE".into());
            }
            let bytes = record
                .request
                .checkpoint
                .strip_prefix(USER_ONLY)
                .ok_or("POC_CHECKPOINT_POLICY")?;
            pd.import_checkpoint(&session, trigger(bytes).request())?;
        }
        None => pd.open_session(&session, &identity.scope, DebateMode::Scientific, tick),
    }
    Ok(pd)
}

fn trace(
    steps: &mut Vec<LgcStep>,
    initial: Hash,
    site: &str,
    action: &str,
    payload: &[u8],
) -> Result<(), String> {
    let predecessor = steps.last().map(|s| s.step_digest).unwrap_or(initial);
    steps.push(LgcStep::new(
        steps.len() as u32,
        predecessor,
        site,
        action,
        sha(payload),
    )?);
    Ok(())
}

fn build_request(input:&ContinuityInput,tick:u64,head:Option<&PdOperationRecord>,memory:&KnowledgeIndex)->Result<PdOperationRequest,String> {
    let (original,actual_tick)=knowledge::decode_input(&input.science_query)?;
    if original!=*input || tick!=actual_tick {return Err("POC009_EXACT_INPUT_BINDING".into());}
    let source_input=encode_continuity_input(input)?;
    let source_hash=sha(&source_input);
    let identity_key=input.identity.key()?;
    let head_binding=head.map(|r|r.record_id).unwrap_or(identity_key);
    let mut steps=Vec::new();
    trace(&mut steps,source_hash,"INPUT","bind exact input bytes; live FFI crossing is reported separately",&input.science_query)?;
    trace(&mut steps,source_hash,"D-BAB_MODEL_ADAPTER","new scoped witness of typed case before computation",&source_input)?;
    trace(&mut steps,source_hash,"PD_MAP","resolve exact scoped predecessor",&head_binding)?;
    let mut pd=fresh_engine(&input.identity,head,tick)?;
    trace(&mut steps,source_hash,"QUANTUM_PD","restore fresh engine or initialize absent context",&head_binding)?;
    let session_id=hex(&identity_key);
    pd.ingest(&session_id,trigger(input.user_text.as_str()).request(),tick).map_err(|e|e.to_string())?;
    trace(&mut steps,source_hash,"QUANTUM_PD","ingest exact user text with explicit typed science inputs",input.user_text.as_bytes())?;
    let result=knowledge::evaluate(trigger(input).request())?;
    trace(&mut steps,source_hash,"SCIENCE","compute source model support counterproof or unresolved condition",result.status.as_bytes())?;
    let prior=memory.lookup(identity_key,&result);
    trace(&mut steps,source_hash,"PD_MAP","retrieve exact context and candidate; historical result is not authority",&serde_json::to_vec(&prior).map_err(|e|e.to_string())?)?;
    trace(&mut steps,source_hash,"D-BAB_MODEL_ADAPTER","new scoped condition summary; no native D-BAB verdict",&[result.status.as_bytes(),result.question.as_bytes()].concat())?;
    let checkpoint=[USER_ONLY,pd.export_checkpoint(&session_id)?.as_slice()].concat();
    trace(&mut steps,source_hash,"QUANTUM_PD","export bounded PD checkpoint",&checkpoint)?;
    let scoped_question=format!("Case {}. {} Prior identical candidate found: {}. Native constitutional admission is outside this experiment.",input.identity.case_id,result.question,prior["found"]);
    let witness=ConditionWitness {
        predecessor:"POC009_TYPED_INPUT".into(),parent_authority:"POC009_SCOPED_MODEL_VERIFIER".into(),
        failed_action:"native_knowledge_admission_not_requested".into(),condition_code:"NATIVE_PIPELINE_OUT_OF_SCOPE".into(),
        reason:"The current model support/counterproof is recorded separately; no native Shadow verdict or physical applicability proof is issued.".into(),
        source_component:"POC009_ADAPTER".into(),source_sha256:sha(&[include_bytes!("continuity.rs").as_slice(),include_bytes!("knowledge.rs").as_slice()].concat()),
        attempt_trace:format!("explicit_tick:{tick}"),input_sha256:source_hash,lgc_parent:steps.last().ok_or("POC_TRACE_MISSING")?.step_digest,
        report_class:ReportClass::MissingConditions,permitted_next:NextAction::ExplainConditions,
    };
    let request=PdOperationRequest{identity:input.identity.clone(),operation_id:input.operation_id.clone(),source_input,
        expected_revision:head.map(|r|r.revision).unwrap_or(0),previous_state:head.map(|r|r.state_digest).unwrap_or([0;32]),
        witness,checkpoint,scoped_question,candidate:result.candidate,candidate_key:result.candidate_id,lgc_steps:steps};
    request.validate()?;Ok(request)
}

fn tick(request:&PdOperationRequest)->Result<u64,String>{
    request.witness.attempt_trace.strip_prefix("explicit_tick:").ok_or("POC_TICK_FORMAT")?.parse().map_err(|_|"POC_TICK_FORMAT".into())
}
fn validate_transition(request:&PdOperationRequest,head:Option<&PdOperationRecord>,memory:&KnowledgeIndex)->Result<(),String>{
    let source=decode_continuity_input(&request.source_input)?;
    let recomputed=build_request(&source,tick(request)?,head,memory)?;
    if recomputed!=*request{return Err("POC_RECOMPUTED_TRANSITION_MISMATCH".into());}Ok(())
}

/// One file owner, one mutable commit boundary. SHA-256 is not authentication.
pub struct Store {
    file: File,
    projection: PdProjection,
    memory: KnowledgeIndex,
    poisoned: bool,
    recovered_tail_bytes: u64,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(io)?;
        file.try_lock().map_err(|e| format!("POC_STORE_LOCK:{e}"))?;
        if file.metadata().map_err(io)?.len() == 0 {
            file.write_all(MAGIC).map_err(io)?;
            file.sync_all().map_err(io)?;
        }
        file.seek(SeekFrom::Start(0)).map_err(io)?;
        let length = file.metadata().map_err(io)?.len();
        let mut projection = PdProjection::default();
        let mut memory = KnowledgeIndex::default();
        let mut offset = 8u64;
        {
            let mut reader = BufReader::with_capacity(64 * 1024, &mut file);
            let mut magic = [0u8; 8];
            reader.read_exact(&mut magic).map_err(io)?;
            if &magic != MAGIC {
                return Err("POC_WAL_MAGIC".into());
            }
            while offset < length {
                let remaining = length - offset;
                if remaining < 12 {
                    let mut partial = [0u8; 12];
                    reader
                        .read_exact(&mut partial[..remaining as usize])
                        .map_err(io)?;
                    if remaining >= 4 {
                        let n =
                            u32::from_le_bytes(partial[..4].try_into().map_err(|_| "POC_HEADER")?);
                        if n == 0 || n as usize > MAX_RECORD {
                            return Err("POC_CORRUPT_FRAME_HEADER".into());
                        }
                        let present = (remaining as usize - 4).min(4);
                        if partial[4..4 + present] != (!n).to_le_bytes()[..present] {
                            return Err("POC_CORRUPT_FRAME_HEADER".into());
                        }
                    }
                    break;
                }
                let mut header = [0u8; 12];
                reader.read_exact(&mut header).map_err(io)?;
                let n = u32::from_le_bytes(header[0..4].try_into().map_err(|_| "POC_HEADER")?);
                let inverse =
                    u32::from_le_bytes(header[4..8].try_into().map_err(|_| "POC_HEADER")?);
                let checksum =
                    u32::from_le_bytes(header[8..12].try_into().map_err(|_| "POC_HEADER")?);
                if n == 0 || n as usize > MAX_RECORD || n != !inverse {
                    return Err("POC_CORRUPT_FRAME_HEADER".into());
                }
                if remaining < 12 + u64::from(n) {
                    break;
                }
                let mut payload = vec![0; n as usize];
                reader.read_exact(&mut payload).map_err(io)?;
                if frame_crc(n, &payload) != checksum {
                    return Err("POC_CORRUPT_FRAME_CRC".into());
                }
                let record = decode_record(&payload)?;
                let prior =
                    projection.operation(&record.request.identity, &record.request.operation_id)?;
                if prior.is_some() {
                    return Err("POC_DUPLICATE_DURABLE_OPERATION".into());
                }
                let head = projection.head(&record.request.identity)?;
                validate_transition(&record.request, head.as_ref(), &memory)?;
                memory.record(&record)?;
                projection.replay(record)?;
                offset += 12 + u64::from(n);
            }
        }
        // Do not repair anything until every complete committed frame has validated.
        let recovered_tail_bytes = length - offset;
        if recovered_tail_bytes > 0 {
            file.set_len(offset).map_err(io)?;
            file.sync_all().map_err(io)?;
        }
        file.seek(SeekFrom::End(0)).map_err(io)?;
        Ok(Self {
            file,
            projection,
            memory,
            poisoned: false,
            recovered_tail_bytes,
        })
    }

    pub fn memory(&self)->&KnowledgeIndex {&self.memory}

    pub fn recovered_tail_bytes(&self) -> u64 {
        self.recovered_tail_bytes
    }
    pub fn head(&self, identity: &PdIdentity) -> Result<Option<PdOperationRecord>, String> {
        self.projection.head(identity)
    }
    pub fn bytes(&self) -> Result<u64, String> {
        self.file.metadata().map(|m| m.len()).map_err(io)
    }

    pub fn prepare(
        &self,
        input: &ContinuityInput,
        tick_value: u64,
    ) -> Result<PdOperationRequest, String> {
        // Exact retry precedes current-head restoration, as in pd_operational.rs.
        if let Some(prior) = self
            .projection
            .operation(&input.identity, &input.operation_id)?
        {
            if prior.request.source_input != encode_continuity_input(input)?
                || tick(&prior.request)? != tick_value
            {
                return Err("PD_OPERATION_ID_CONFLICT".into());
            }
            return Ok(prior.request);
        }
        build_request(input, tick_value, self.head(&input.identity)?.as_ref(), &self.memory)
    }

    pub fn commit(&mut self, request: PdOperationRequest) -> Result<PdOperationRecord, String> {
        if self.poisoned {
            return Err("POC_STORE_POISONED_REOPEN_REQUIRED".into());
        }
        match self.projection.prepare(request.clone())? {
            PdCommitPlan::Existing(record) => Ok(record.as_ref().clone()),
            PdCommitPlan::Append(append) => {
                validate_transition(&request, self.head(&request.identity)?.as_ref(), &self.memory)?;
                let mut next_memory=self.memory.clone();
                next_memory.record(append.record())?;
                let payload = encode_record(append.record())?;
                let n = u32::try_from(payload.len()).map_err(|_| "POC_FRAME_LENGTH")?;
                let mut frame = Vec::with_capacity(12 + payload.len());
                frame.extend_from_slice(&n.to_le_bytes());
                frame.extend_from_slice(&(!n).to_le_bytes());
                frame.extend_from_slice(&frame_crc(n, &payload).to_le_bytes());
                frame.extend_from_slice(&payload);
                self.poisoned = true;
                self.file
                    .write_all(trigger(frame.as_slice()).request())
                    .map_err(io)?;
                self.file.sync_all().map_err(io)?;
                let record = self.projection.publish(append);
                self.memory=next_memory;
                self.poisoned = false;
                Ok(record.as_ref().clone())
            }
        }
    }

    pub fn step(
        &mut self,
        input: &ContinuityInput,
        tick_value: u64,
    ) -> Result<PdOperationRecord, String> {
        let request = self.prepare(input, tick_value)?;
        self.commit(request)
    }
}

/// Control path: the same original engine receives every turn without a reset.
pub fn uninterrupted(identity: &PdIdentity, turns: &[(String, u64)]) -> Result<Vec<u8>, String> {
    let first_tick = turns.first().ok_or("POC_EMPTY_CONTROL")?.1;
    let mut pd = fresh_engine(identity, None, first_tick)?;
    let session = hex(&identity.key()?);
    for (text, timestamp) in turns {
        pd.ingest(&session, text, *timestamp)
            .map_err(|e| e.to_string())?;
    }
    Ok([USER_ONLY, pd.export_checkpoint(&session)?.as_slice()].concat())
}

pub fn summary(record: &PdOperationRecord) -> Result<serde_json::Value, String> {
    record.validate()?;
    let pd = fresh_engine(
        &record.request.identity,
        Some(record),
        tick(&record.request)?,
    )?;
    let session = pd
        .session(&hex(&record.request.identity.key()?))
        .ok_or("POC_SESSION_MISSING")?;
    let last = session.turns.last().ok_or("POC_TURN_MISSING")?;
    Ok(
        serde_json::json!({"case":record.request.identity.case_id,"operation":record.request.operation_id,
        "revision":record.revision,"record_id":hex(&record.record_id),"previous_record":hex(&record.previous_record),
        "state_sha256":hex(&record.state_digest),"checkpoint_sha256":hex(&sha(&record.request.checkpoint)),
        "total_turns":session.total_turns,"challenges_issued":session.pressure_budget.challenges_issued,
        "budget_exhausted":session.pressure_budget.budget_exhausted,"returned_concepts":last.user_signal.returns_to,
        "question":record.request.scoped_question,"lgc_steps":record.request.lgc_steps.len(),
        "knowledge_verdict":null,"authority_transferred":false,"science_executed":!record.request.candidate.is_empty(),
        "model_result":knowledge::evaluate(&decode_continuity_input(&record.request.source_input)?)?.status,
        "candidate_sha256":hex(&record.request.candidate_key),
        "exact_input_sha256":hex(&sha(&decode_continuity_input(&record.request.source_input)?.science_query)),
        "lgc":record.request.lgc_steps.iter().map(|s|serde_json::json!({"site":s.site,"action":s.action,"digest":hex(&s.step_digest)})).collect::<Vec<_>>(),
        "user_turns":session.turns.iter().map(|t| t.user_signal.raw_content.clone()).collect::<Vec<_>>()}),
    )
}

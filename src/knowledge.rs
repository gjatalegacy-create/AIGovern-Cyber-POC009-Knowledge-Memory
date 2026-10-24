//! New POC009 scoped computational memory adapter. No native Shadow verdict.
use crate::pd_memory::{ContinuityInput, Hash, PdIdentity, PdOperationRecord};
use science_domains::{AdapterId, ModelRelation, ScienceQuery, ScienceTask, TimeScale};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn sha(bytes: &[u8]) -> Hash { Sha256::digest(bytes).into() }
pub fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn fail(e: impl std::fmt::Display) -> String { e.to_string() }

pub fn identity(case: &str) -> PdIdentity {
    PdIdentity { owner: "public-fixture-owner".into(), obligation: "compare-exact-UT1-model-claim".into(),
        case_id: case.into(), scope: "model-conditional-knowledge-only".into(), domain: "IAU2000-era00".into(),
        lineage_root: "ESS-MAI/AIGovern-Cyber/POC009".into() }
}

/// The exact incoming JSON bytes are retained in science_query, including punctuation.
/// JSON provides explicit typed inputs; this is not a free-text scientific parser.
pub fn decode_input(bytes: &[u8]) -> Result<(ContinuityInput,u64),String> {
    if bytes.len() > 16*1024 { return Err("POC009_INPUT_LIMIT".into()); }
    let value:Value=serde_json::from_slice(bytes).map_err(fail)?;
    let object=value.as_object().ok_or("POC009_EXPECT_OBJECT")?;
    if object.len()!=5 || object.keys().any(|k| !["case","operation","tick","text","science"].contains(&k.as_str())) {
        return Err("POC009_INPUT_FIELDS".into());
    }
    let s=|key| value[key].as_str().ok_or_else(||format!("POC009_FIELD:{key}"));
    let tick=value["tick"].as_u64().ok_or("POC009_TICK")?;
    let input=ContinuityInput { identity:identity(s("case")?), operation_id:s("operation")?.into(),
        user_text:s("text")?.into(), science_query:bytes.to_vec() };
    input.identity.validate()?;
    Ok((input,tick))
}

fn query(input:&ContinuityInput)->Result<Option<ScienceQuery>,String> {
    let (decoded,_)=decode_input(&input.science_query)?;
    if decoded!=*input {return Err("POC009_INPUT_BINDING".into());}
    let value:Value=serde_json::from_slice(&input.science_query).map_err(fail)?;
    let science=&value["science"];
    if science.is_null(){return Ok(None);}
    let object=science.as_object().ok_or("POC009_SCIENCE_OBJECT")?;
    if object.len()!=6 || object.keys().any(|k| !["jd_part1","jd_part2","time_scale","claim_radians","tolerance_radians","source"].contains(&k.as_str())) {
        return Err("POC009_SCIENCE_FIELDS".into());
    }
    let n=|key| science[key].as_f64().ok_or_else(||format!("POC009_NUMBER:{key}"));
    let time_scale=match science["time_scale"].as_str(){Some("UT1")=>TimeScale::Ut1,Some("UTC")=>TimeScale::Utc,_=>return Err("POC009_TIME_SCALE".into())};
    Ok(Some(ScienceQuery { adapter:AdapterId::Astronomy,session_id:hex(&input.identity.key()?),
        case_id:input.identity.case_id.clone(),scope_id:input.identity.scope.clone(),obligation_id:input.identity.obligation.clone(),
        query_id:input.operation_id.clone(), input_source_id:science["source"].as_str().ok_or("POC009_SOURCE")?.into(),
        task:ScienceTask::EarthRotationAngle{jd_part1:n("jd_part1")?,jd_part2:n("jd_part2")?,time_scale,
            claim_radians:n("claim_radians")?,tolerance_radians:n("tolerance_radians")?}}))
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ModelResult {
    pub status: &'static str,
    pub candidate:Vec<u8>,
    pub content:Vec<u8>,
    pub candidate_id:Hash,
    pub proposition_id:Hash,
    pub question:String,
}

/// A fresh computation owns its result. Neither a caller's label nor old history decides it.
pub fn evaluate(input:&ContinuityInput)->Result<ModelResult,String> {
    let Some(query)=query(input)? else {
        return Ok(ModelResult { status:"missing_evidence",candidate:Vec::new(),content:Vec::new(),candidate_id:[0;32],proposition_id:[0;32],
            question:"The claim is not yet computable. Which two-part UT1 Julian date, angle claim in radians, comparison tolerance and input source should this case use? Missing inputs are not a counterproof.".into() });
    };
    let candidate=science_domains::evaluate(&query).map_err(fail)?;
    let encoded=science_domains::encode_candidate(&candidate).map_err(fail)?;
    let checked=science_domains::kie::evaluate_kie(&science_domains::encode_query(&query).map_err(fail)?,&encoded).map_err(fail)?;
    let proposition=checked.canonical_proposition;
    let status=match candidate.relation { ModelRelation::SupportsClaim=>"model_support",ModelRelation::CountersClaim=>"model_counterproof",ModelRelation::Unresolved=>"unresolved_boundary" };
    let ScienceTask::EarthRotationAngle{claim_radians,tolerance_radians,..}=query.task;
    let question=match candidate.relation {
        ModelRelation::SupportsClaim=>format!("Under the declared IAU2000 model, angle {} rad supports claim {} rad: residual {} < tolerance {} rad. What independent UT1 source and uncertainty establish applicability to the physical case?",candidate.computed_radians,claim_radians,candidate.circular_residual_radians,tolerance_radians),
        ModelRelation::CountersClaim=>format!("Under the declared IAU2000 model, angle {} rad counters claim {} rad: residual {} > tolerance {} rad. Which exact claim, UT1 input or justified tolerance do you revise, and what source supports that revision?",candidate.computed_radians,claim_radians,candidate.circular_residual_radians,tolerance_radians),
        ModelRelation::Unresolved=>format!("Residual {} equals tolerance {} rad; neither strict support nor strict counterproof holds. What additional input precision or independently justified criterion resolves this boundary?",candidate.circular_residual_radians,tolerance_radians),
    };
    let proposition_id=sha(proposition.as_bytes());
    // Separate immutable proposition identity from occurrence/query ID. Context and declared
    // source remain part of content identity; repeat observations cannot inflate independence.
    let content=serde_json::to_vec(&json!({"schema":"POC009/MODEL-CONTENT/1","context":hex(&input.identity.key()?),
        "proposition":proposition,"status":status,"source":query.input_source_id,
        "computed_bits":format!("{:016x}",candidate.computed_radians.to_bits()),
        "residual_bits":format!("{:016x}",candidate.circular_residual_radians.to_bits())})).map_err(fail)?;
    let candidate_id=sha(&content);
    Ok(ModelResult {status,candidate:encoded,content,candidate_id,proposition_id,question})
}

#[derive(Debug,Clone,PartialEq,Eq)]
struct Entry { content:Vec<u8>,status:&'static str,proposition_id:Hash,origins:Vec<Hash> }

/// A rebuildable address index over verified WAL records, not a second persistence store.
#[derive(Debug,Clone,Default,PartialEq,Eq)]
pub struct KnowledgeIndex { entries:BTreeMap<(Hash,Hash),Vec<Entry>>, missing:BTreeMap<Hash,u64>, unresolved:BTreeMap<Hash,u64> }
impl KnowledgeIndex {
    /// Called by the storage owner after its current-transition check. Replay follows the same rule.
    pub(crate) fn record(&mut self,record:&PdOperationRecord)->Result<(),String> {
        let input=crate::pd_memory::decode_continuity_input(&record.request.source_input)?;
        let result=evaluate(&input)?;
        if result.candidate!=record.request.candidate || result.candidate_id!=record.request.candidate_key {
            return Err("POC009_MEMORY_COMPUTE_MISMATCH".into());
        }
        let context=input.identity.key()?;
        match result.status {
            "missing_evidence"=>{*self.missing.entry(context).or_default()+=1;},
            "unresolved_boundary"=>{*self.unresolved.entry(context).or_default()+=1;},
            _=>{
                let bucket=self.entries.entry((context,result.candidate_id)).or_default();
                match bucket.iter_mut().find(|entry|entry.content==result.content) {
                    Some(entry)=>{if !entry.origins.contains(&record.record_id){entry.origins.push(record.record_id);}},
                    None=>bucket.push(Entry{content:result.content,status:result.status,proposition_id:result.proposition_id,origins:vec![record.record_id]})
                }
            }
        }
        Ok(())
    }
    /// Exact content equality is required even within a SHA-256 bucket.
    pub fn lookup(&self,context:Hash,result:&ModelResult)->Value {
        let hit=self.entries.get(&(context,result.candidate_id)).and_then(|bucket|bucket.iter().find(|entry|entry.content==result.content));
        match hit {Some(entry)=>json!({"found":true,"candidate_sha256":hex(&result.candidate_id),"status":entry.status,
            "occurrences":entry.origins.len(),"origin_record_sha256":entry.origins.iter().map(|h|hex(h)).collect::<Vec<_>>()}),
            None=>json!({"found":false})}
    }
    pub fn view(&self,context:Hash)->Value {
        let mut rows=Vec::new();let mut positive=0;let mut negative=0;
        for ((ctx,id),bucket) in &self.entries {if *ctx!=context{continue;} for entry in bucket {
            positive+=usize::from(entry.status=="model_support");negative+=usize::from(entry.status=="model_counterproof");
            rows.push(json!({"candidate_sha256":hex(id),"proposition_sha256":hex(&entry.proposition_id),"status":entry.status,
                "canonical_content":String::from_utf8_lossy(&entry.content),"occurrences":entry.origins.len(),
                "origin_record_sha256":entry.origins.iter().map(|h|hex(h)).collect::<Vec<_>>() }));
        }}
        json!({"positive_candidates":positive,"negative_candidates":negative,"missing_evidence_events":self.missing.get(&context).copied().unwrap_or(0),
            "unresolved_events":self.unresolved.get(&context).copied().unwrap_or(0),"candidates":rows})
    }
}

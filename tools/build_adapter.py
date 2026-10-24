"""Documented adaptation of POC008's tested WAL owner for POC009.
This generates NEW adapter code, not an unchanged extraction from ESS-MAI.
"""
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parents[1]
base=root.parent/'poc008-continuity/src/continuity.rs'
text=base.read_text(encoding='utf-8')
text=text.replace('use crate::framing::frame_crc;', 'use crate::framing::frame_crc;\nuse crate::knowledge::{self, KnowledgeIndex};')
text=text.replace('b"OPL2SPD1"','b"OPL2SPN9"').replace('const NO_SCIENCE: &[u8] = b"POC008:SCIENCE_NOT_EXECUTED";\n','')
start=text.index('pub fn input(');end=text.index('fn fresh_engine(',start)
text=text[:start]+'''pub fn input(case_id:&str, operation_id:&str, user_text:&str, tick:u64, science:serde_json::Value)->Result<ContinuityInput,String> {
    let raw=serde_json::to_vec(&serde_json::json!({"case":case_id,"operation":operation_id,"text":user_text,"tick":tick,"science":science})).map_err(|e|e.to_string())?;
    Ok(knowledge::decode_input(&raw)?.0)
}

'''+text[end:]
start=text.index('fn build_request(');end=text.index('/// One file owner',start)
text=text[:start]+'''fn build_request(input:&ContinuityInput,tick:u64,head:Option<&PdOperationRecord>,memory:&KnowledgeIndex)->Result<PdOperationRequest,String> {
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

'''+text[end:]
text=text.replace('projection: PdProjection,','projection: PdProjection,\n    memory: KnowledgeIndex,')
text=text.replace('let mut projection = PdProjection::default();','let mut projection = PdProjection::default();\n        let mut memory = KnowledgeIndex::default();')
text=text.replace('validate_transition(&record.request, head.as_ref())?;','validate_transition(&record.request, head.as_ref(), &memory)?;\n                memory.record(&record)?;')
text=text.replace('            projection,\n','            projection,\n            memory,\n')
text=text.replace('    pub fn recovered_tail_bytes(&self) -> u64 {','    pub fn memory(&self)->&KnowledgeIndex {&self.memory}\n\n    pub fn recovered_tail_bytes(&self) -> u64 {')
text=text.replace('build_request(input, tick_value, self.head(&input.identity)?.as_ref())','build_request(input, tick_value, self.head(&input.identity)?.as_ref(), &self.memory)')
text=text.replace('validate_transition(&request, self.head(&request.identity)?.as_ref())?;','validate_transition(&request, self.head(&request.identity)?.as_ref(), &self.memory)?;\n                let mut next_memory=self.memory.clone();\n                next_memory.record(append.record())?;')
text=text.replace('let record = self.projection.publish(append);','let record = self.projection.publish(append);\n                self.memory=next_memory;')
text=text.replace('"knowledge_verdict":null,"authority_transferred":false,"science_executed":false,','"knowledge_verdict":null,"authority_transferred":false,"science_executed":!record.request.candidate.is_empty(),\n        "model_result":knowledge::evaluate(&decode_continuity_input(&record.request.source_input)?)?.status,\n        "candidate_sha256":hex(&record.request.candidate_key),\n        "exact_input_sha256":hex(&sha(&decode_continuity_input(&record.request.source_input)?.science_query)),\n        "lgc":record.request.lgc_steps.iter().map(|s|serde_json::json!({"site":s.site,"action":s.action,"digest":hex(&s.step_digest)})).collect::<Vec<_>>(),')
(root/'src/continuity.rs').write_text(text,encoding='utf-8',newline='\n')
(root/'adapter_provenance.json').write_text(json.dumps({'base':'https://github.com/gjatalegacy-create/AIGovern-Cyber-POC008-Continuity/blob/poc008-public-v0.1.0/src/continuity.rs','base_sha256':hashlib.sha256(base.read_bytes()).hexdigest(),'classification':'NEW POC009 adaptation; typed science inputs, current recomputation, derived scoped content index, changed WAL magic. Not original KnowledgeVault.','generator':'tools/build_adapter.py','output_sha256':hashlib.sha256((root/'src/continuity.rs').read_bytes()).hexdigest()},indent=2)+'\n',encoding='utf-8')
print('Generated POC009 adapter')

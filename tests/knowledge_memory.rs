use poc009_knowledge_memory::{input,knowledge,pd_memory::{encode_record,PdOperationRecord},summary,uninterrupted,Store};
use serde_json::{json,Value};
use std::{fs,path::PathBuf,process::{Command,Stdio},io::{BufRead,BufReader,Write},sync::atomic::{AtomicU64,Ordering}};
static NEXT:AtomicU64=AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {fn new()->Self{let p=std::env::temp_dir().join(format!("poc009-{}-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),NEXT.fetch_add(1,Ordering::SeqCst)));fs::create_dir(&p).unwrap();Self(p)}fn wal(&self)->PathBuf{self.0.join("memory.wal")}}
impl Drop for Temp {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn science(claim:f64)->Value{json!({"jd_part1":2400000.5,"jd_part2":54388.0,"time_scale":"UT1","claim_radians":claim,"tolerance_radians":1e-12,"source":"public-SOFA-conformance-vector; not observed UT1"})}
fn make(case:&str,op:&str,tick:u64,model:Value)->poc009_knowledge_memory::pd_memory::ContinuityInput{input(case,op,"Same Earth-rotation case, punctuation stays exact; π, shqiponjë.",tick,model).unwrap()}
fn fixtures()->Vec<Value>{
    let texts=["I want to check the Earth's rotation angle, but have not supplied the numerical inputs.",
        "Use UT1 JD 2400000.5 + 54388.0; my first claim is 0 radians, tolerance 1e-12.",
        "I revise that claim to 0.4022837240028158102 radians, with the same UT1 and tolerance.",
        "Recall my first 0-radian claim, and test it again without erasing its counterproof.",
        "Recall the corrected claim; preserve both histories, with their distinct propositions."];
    texts.iter().enumerate().map(|(i,text)|json!({"case":"earth-rotation","operation":format!("cycle-{}",i+1),"tick":1000+i,"text":text,
        "science":match i{0=>Value::Null,1|3=>science(0.0),_=>science(0.4022837240028158102)}})).collect()
}
fn run(t:&Temp,v:&Value)->Value{
    let p=t.0.join(format!("{}.json",v["operation"].as_str().unwrap()));let raw=serde_json::to_vec_pretty(v).unwrap();fs::write(&p,&raw).unwrap();
    let r=Command::new(env!("CARGO_BIN_EXE_poc009-memory")).arg("step").arg(t.wal()).arg(p).output().unwrap();
    assert!(r.status.success(),"{}",String::from_utf8_lossy(&r.stderr));let j:Value=serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(j["exact_input_sha256"],knowledge::hex(&knowledge::sha(&raw)));assert_eq!(j["ffi_same_pointer"],true);assert_eq!(j["ffi_exact_bytes"],true);assert_eq!(j["ffi_reuse_code"],-1);j
}

#[test] fn five_fresh_processes_preserve_both_polarities_and_exact_pd_prefixes(){
    let t=Temp::new();let control=Temp::new();let mut same_process=Store::open(control.wal()).unwrap();let mut turns=Vec::new();let mut outputs=Vec::new();
    for (i,v) in fixtures().iter().enumerate(){
        let j=run(&t,v);let raw=serde_json::to_vec_pretty(v).unwrap();let (request,tick)=knowledge::decode_input(&raw).unwrap();
        let expected=same_process.step(&request,tick).unwrap();let reopened=Store::open(t.wal()).unwrap();let actual=reopened.head(&request.identity).unwrap().unwrap();
        assert_eq!(actual,expected);assert_eq!(reopened.memory(),same_process.memory());
        turns.push((request.user_text.clone(),tick));assert_eq!(actual.request.checkpoint,uninterrupted(&request.identity,&turns).unwrap());
        assert_eq!(j["total_turns"],i+1);assert_eq!(j["revision"],i+1);assert_eq!(j["knowledge_verdict"],Value::Null);
        assert_eq!(j["model_result"],["missing_evidence","model_counterproof","model_support","model_counterproof","model_support"][i]);
        assert_eq!(j["memory_after"]["positive_candidates"],[0,0,1,1,1][i]);assert_eq!(j["memory_after"]["negative_candidates"],[0,1,1,1,1][i]);
        assert_eq!(j["memory_after"]["missing_evidence_events"],1);assert_eq!(j["retrieved_before"]["found"],i>=3);outputs.push(j);
    }
    assert_eq!(outputs[1]["candidate_sha256"],outputs[3]["candidate_sha256"]);assert_eq!(outputs[2]["candidate_sha256"],outputs[4]["candidate_sha256"]);
    assert_ne!(outputs[1]["candidate_sha256"],outputs[2]["candidate_sha256"]);
    for row in outputs[4]["memory_after"]["candidates"].as_array().unwrap(){assert_eq!(row["occurrences"],2);}
}
#[test] fn missing_input_is_a_question_not_a_semantic_negative(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let i=make("case","missing",1,Value::Null);let r=s.step(&i,1).unwrap();assert!(r.request.candidate.is_empty());let m=s.memory().view(i.identity.key().unwrap());assert_eq!(m["negative_candidates"],0);assert_eq!(m["missing_evidence_events"],1);assert!(r.request.scoped_question.contains("Which two-part UT1"));}
#[test] fn equality_is_unresolved_and_creates_neither_polarity(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let i=make("case","base",1,science(0.0));let result=knowledge::evaluate(&i).unwrap();let c=science_domains::decode_candidate(&result.candidate).unwrap();let mut q=science(0.0);q["tolerance_radians"]=c.circular_residual_radians.into();let i=make("case","boundary",1,q);let r=s.step(&i,1).unwrap();assert_eq!(summary(&r).unwrap()["model_result"],"unresolved_boundary");let m=s.memory().view(i.identity.key().unwrap());assert_eq!(m["positive_candidates"],0);assert_eq!(m["negative_candidates"],0);assert_eq!(m["unresolved_events"],1);}
#[test] fn current_counterproof_remains_negative_even_after_positive_memory(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();s.step(&make("case","positive",1,science(0.4022837240028158102)),1).unwrap();let n=s.step(&make("case","counter",2,science(0.0)),2).unwrap();assert_eq!(summary(&n).unwrap()["model_result"],"model_counterproof");assert!(n.request.scoped_question.contains("counters claim"));assert!(n.request.scoped_question.contains("Which exact claim"));}
#[test] fn candidate_retrieval_is_exact_scope_and_proposition(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let a=make("case-a","n",1,science(0.0));s.step(&a,1).unwrap();let n=knowledge::evaluate(&a).unwrap();assert_eq!(s.memory().lookup(a.identity.key().unwrap(),&n)["found"],true);assert_eq!(s.memory().lookup(knowledge::identity("case-b").key().unwrap(),&n)["found"],false);let p=make("case-a","p",2,science(0.4022837240028158102));assert_eq!(s.memory().lookup(p.identity.key().unwrap(),&knowledge::evaluate(&p).unwrap())["found"],false);}
#[test] fn public_prepared_fields_do_not_bypass_owned_recomputation(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let mut r=s.prepare(&make("case","n",1,science(0.0)),1).unwrap();r.scoped_question="Accepted without computation".into();let n=s.bytes().unwrap();assert!(s.commit(r).unwrap_err().contains("RECOMPUTED_TRANSITION"));assert_eq!(s.bytes().unwrap(),n);}
#[test] fn changed_candidate_bytes_and_rehashed_key_are_refused(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let mut r=s.prepare(&make("case","n",1,science(0.0)),1).unwrap();r.candidate[30]^=1;r.candidate_key=knowledge::sha(&r.candidate);assert!(s.commit(r).unwrap_err().contains("RECOMPUTED_TRANSITION"));assert_eq!(s.bytes().unwrap(),8);}
#[test] fn false_support_with_valid_recomputed_record_hash_is_refused_on_reopen(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let r=s.step(&make("case","n",1,science(0.0)),1).unwrap();drop(s);let mut request=r.request;request.scoped_question="This incorrect claim was supported".into();let forged=PdOperationRecord::construct(request,[0;32]).unwrap();let bytes=encode_record(&forged).unwrap();let n=bytes.len() as u32;let mut frame=b"OPL2SPN9".to_vec();frame.extend(n.to_le_bytes());frame.extend((!n).to_le_bytes());frame.extend(crc(n,&bytes).to_le_bytes());frame.extend(bytes);fs::write(t.wal(),&frame).unwrap();assert!(Store::open(t.wal()).err().unwrap().contains("RECOMPUTED_TRANSITION"));assert_eq!(fs::read(t.wal()).unwrap(),frame);}
fn crc(n:u32,bytes:&[u8])->u32{let mut c=!0u32;for byte in n.to_le_bytes().iter().chain(bytes){c^=u32::from(*byte);for _ in 0..8{c=(c>>1)^(0xedb88320u32.wrapping_mul(c&1));}}!c}
#[test] fn exact_old_retry_does_not_duplicate_evidence_after_head_advanced(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let n=make("case","n",1,science(0.0));let old=s.step(&n,1).unwrap();let p=s.step(&make("case","p",2,science(0.4022837240028158102)),2).unwrap();let size=s.bytes().unwrap();let memory=s.memory().clone();assert_eq!(s.step(&n,1).unwrap(),old);assert_eq!(s.bytes().unwrap(),size);assert_eq!(s.memory(),&memory);assert_eq!(s.head(&n.identity).unwrap().unwrap(),p);}
#[test] fn stale_and_changed_operations_are_refused(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let n=make("case","n",1,science(0.0));let stale=s.prepare(&n,1).unwrap();s.step(&make("case","p",2,science(0.4022837240028158102)),2).unwrap();assert!(s.commit(stale).unwrap_err().contains("CAS"));let p=make("case","p",2,science(0.0));assert!(s.step(&p,2).unwrap_err().contains("CONFLICT"));}
#[test] fn unsupported_timescale_does_not_append(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let mut q=science(0.0);q["time_scale"]="UTC".into();assert!(s.step(&make("case","bad",1,q),1).unwrap_err().contains("UT1"));assert_eq!(s.bytes().unwrap(),8);}

fn invalid_science_cli(t:&Temp)->std::process::Output {
    let mut v=fixtures()[1].clone(); v["science"]["time_scale"]="UTC".into();
    let path=t.0.join("invalid.json");fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_poc009-memory")).arg("step").arg(t.wal()).arg(path).output().unwrap()
}
#[test] fn invalid_science_cli_does_not_create_wal(){
    let t=Temp::new();let result=invalid_science_cli(&t);assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("UT1"));assert!(!t.wal().exists());
}
#[test] fn invalid_science_cli_does_not_repair_existing_tail(){
    let t=Temp::new();run(&t,&fixtures()[1]);
    let mut bytes=fs::read(t.wal()).unwrap();bytes.extend([0x20,0]);fs::write(t.wal(),&bytes).unwrap();
    let result=invalid_science_cli(&t);assert!(!result.status.success());assert_eq!(fs::read(t.wal()).unwrap(),bytes);
}
#[test] fn altered_lgc_is_refused_without_write(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();let mut r=s.prepare(&make("case","n",1,science(0.0)),1).unwrap();r.lgc_steps[2].payload_digest[0]^=1;assert!(s.commit(r).unwrap_err().contains("LGC"));assert_eq!(s.bytes().unwrap(),8);}
#[test] fn complete_crc_corruption_is_not_repaired(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();s.step(&make("case","n",1,science(0.0)),1).unwrap();drop(s);let mut bytes=fs::read(t.wal()).unwrap();*bytes.last_mut().unwrap()^=1;fs::write(t.wal(),&bytes).unwrap();assert!(Store::open(t.wal()).is_err());assert_eq!(fs::read(t.wal()).unwrap(),bytes);}
#[test] fn incomplete_tail_recovers_last_committed_both_polarity_state(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();s.step(&make("case","n",1,science(0.0)),1).unwrap();s.step(&make("case","p",2,science(0.4022837240028158102)),2).unwrap();let memory=s.memory().clone();let size=s.bytes().unwrap();drop(s);let mut f=fs::OpenOptions::new().append(true).open(t.wal()).unwrap();f.write_all(&[0x20,0]).unwrap();drop(f);let s=Store::open(t.wal()).unwrap();assert_eq!(s.recovered_tail_bytes(),2);assert_eq!(s.bytes().unwrap(),size);assert_eq!(s.memory(),&memory);}
#[test] fn whole_valid_old_prefix_documents_rollback_limit(){let t=Temp::new();let mut s=Store::open(t.wal()).unwrap();s.step(&make("case","n",1,science(0.0)),1).unwrap();drop(s);let old=fs::read(t.wal()).unwrap();let mut s=Store::open(t.wal()).unwrap();s.step(&make("case","p",2,science(0.4022837240028158102)),2).unwrap();drop(s);fs::write(t.wal(),old).unwrap();let s=Store::open(t.wal()).unwrap();let m=s.memory().view(knowledge::identity("case").key().unwrap());assert_eq!(m["negative_candidates"],1);assert_eq!(m["positive_candidates"],0);}
#[test] fn lock_is_exclusive_and_os_releases_it_after_process_kill(){let t=Temp::new();let mut child=Command::new(env!("CARGO_BIN_EXE_poc009-memory")).arg("lock-probe").arg(t.wal()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();let mut line=String::new();BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).unwrap();assert!(line.contains("LOCK_HELD"));assert!(Store::open(t.wal()).err().unwrap().contains("LOCK"));child.kill().unwrap();child.wait().unwrap();assert!(Store::open(t.wal()).is_ok());}

use poc009_knowledge_memory::{ffi_gate,knowledge,summary,Store};
use std::io::Write;

fn run()->Result<(),String>{
    let args:Vec<String>=std::env::args().collect();
    match args.get(1).map(String::as_str){
        Some("lock-probe") if args.len()==3=>{
            let _store=Store::open(&args[2])?;
            println!("LOCK_HELD");std::io::stdout().flush().map_err(|e|e.to_string())?;
            let mut line=String::new();std::io::stdin().read_line(&mut line).map_err(|e|e.to_string())?;Ok(())
        },
        Some("step") if args.len()==4=>{
            let bytes=std::fs::read(&args[3]).map_err(|e|e.to_string())?;
            let handle=ffi_gate::sovereign_issue_capability(ffi_gate::S_SHADOW_WRITE);
            // Owned Vec remains alive and immutably borrowed throughout the callback.
            let output=unsafe{ffi_gate::with_borrowed_payload(handle,bytes.as_ptr(),bytes.len(),|borrowed|->Result<serde_json::Value,String>{
                let same_pointer=borrowed.as_ptr()==bytes.as_ptr();
                if !same_pointer || borrowed!=bytes {return Err("POC009_FFI_BYTE_MISMATCH".into());}
                let (input,tick)=knowledge::decode_input(borrowed)?;
                let model=knowledge::evaluate(&input)?;
                let mut store=Store::open(&args[2])?;
                let before=store.memory().view(input.identity.key()?);
                let previous=store.memory().lookup(input.identity.key()?,&model);
                let record=store.step(&input,tick)?;
                let mut output=summary(&record)?;
                output["memory_before"]=before;output["memory_after"]=store.memory().view(input.identity.key()?);
                output["retrieved_before"]=previous;output["wal_bytes"]=store.bytes()?.into();
                output["ffi_same_pointer"]=same_pointer.into();output["ffi_exact_bytes"]=(borrowed==bytes).into();
                output["recovered_tail_bytes"]=store.recovered_tail_bytes().into();Ok(output)
            })}.map_err(|code|format!("POC009_FFI:{code}"))??;
            let replay=unsafe{ffi_gate::with_borrowed_payload(handle,bytes.as_ptr(),bytes.len(),|_|())};
            if replay.is_ok(){return Err("POC009_FFI_REUSE_NOT_REFUSED".into());}
            let mut output=output;output["ffi_reuse_code"]=replay.err().unwrap().into();
            println!("{}",serde_json::to_string(&output).map_err(|e|e.to_string())?);Ok(())
        },
        Some("inspect") if args.len()==4=>{
            let store=Store::open(&args[2])?;let identity=knowledge::identity(&args[3]);
            let record=store.head(&identity)?.ok_or("POC009_NO_HEAD")?;
            let mut value=summary(&record)?;value["memory"]=store.memory().view(identity.key()?);println!("{value}");Ok(())
        },
        _=>Err("usage: poc009-memory step STORE INPUT.json | inspect STORE CASE".into())
    }
}
fn main(){if let Err(error)=run(){eprintln!("{error}");std::process::exit(70);}}

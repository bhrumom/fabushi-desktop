#[cfg(unix)]
mod actions {
// Nonshipping adapter for real browser resource execution in disposable Actions.
use std::fs;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use mahayana_host_runtime::extensions::inference::provider_session::{ProviderSessionError, RoutedToolDefinition};
use mahayana_host_runtime::runner::box_tool_access::{RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::tools::sand_browser_tools::{browser_tool_specs, ProductionBrowserToolExecutor, SandBrowserToolBridge};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Default)]
struct ActionsResources {
    shells: AtomicUsize,
    writes: AtomicUsize,
    reads: AtomicUsize,
}
fn failure(message: &str) -> ProviderSessionError { ProviderSessionError::Tool(message.into()) }
fn driver_path(path: &str) -> Result<(), ProviderSessionError> {
    let path=Path::new(path);
    if !path.starts_with("/tmp/.sand-browser") || path.components().any(|part| matches!(part,std::path::Component::ParentDir)) {
        return Err(failure("Actions browser resource path denied"));
    }
    Ok(())
}
impl RunnerBoxResourcePort for ActionsResources {
    fn execute_shell(&self, request: RunnerBoxShellRequest) -> Result<Value, ProviderSessionError> {
        if request.working_directory!="/workspace" || request.is_background {return Err(failure("Actions browser shell scope denied"));}
        self.shells.fetch_add(1,Ordering::SeqCst);
        let mut child=Command::new("bash").args(["-lc",&request.command]).current_dir("/workspace")
            .process_group(0).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|_|failure("Actions browser shell failed"))?;
        let started=Instant::now();
        loop {
            if let Some(status)=child.try_wait().map_err(|_|failure("Actions shell status failed"))? {
                return Ok(json!({"kind":"success","exitCode":status.code().unwrap_or(1)}));
            }
            if started.elapsed()>Duration::from_secs(90) {
                unsafe {libc::kill(-(child.id() as i32),libc::SIGKILL);}
                let _=child.wait();return Err(failure("Actions browser shell timed out"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    fn execute_write(&self, request: RunnerBoxWriteRequest) -> Result<(), ProviderSessionError> {
        driver_path(&request.path)?;
        if request.data.len()>2*1024*1024 {return Err(failure("Actions write limit"));}
        fs::write(request.path,request.data).map_err(|_|failure("Actions driver write failed"))?;
        self.writes.fetch_add(1,Ordering::SeqCst);Ok(())
    }
    fn execute_read(&self, request: RunnerBoxReadRequest) -> Result<Value, ProviderSessionError> {
        driver_path(&request.path)?;
        let file=fs::File::open(request.path).map_err(|_|failure("Actions result open failed"))?;
        let mut bytes=Vec::new();file.take(8*1024*1024+1).read_to_end(&mut bytes).map_err(|_|failure("Actions result read failed"))?;
        if bytes.len()>8*1024*1024 {return Err(failure("Actions read limit"));}
        self.reads.fetch_add(1,Ordering::SeqCst);
        if request.encoding_hint.as_deref()==Some("utf-8") {
            let content=String::from_utf8(bytes).map_err(|_|failure("Actions textual result invalid"))?;
            Ok(json!({"kind":"success","output":{"content":content}}))
        } else {Ok(json!({"kind":"success","output":{"data":bytes}}))}
    }
    fn browser_window_index(&self)->Result<u32,ProviderSessionError> {Ok(77)}
}
struct NoDelegate;
impl RoutedToolBridge for NoDelegate {
    fn list_tools(&self)->Result<Vec<RoutedToolDefinition>,ProviderSessionError>{Ok(Vec::new())}
    fn call_tool(&self,_tool:&RoutedToolDefinition,_args:Value,_id:&str)->Result<Value,ProviderSessionError>{Err(failure("Actions unknown tool"))}
}
pub fn run()->Result<(),Box<dyn std::error::Error>> {
    if std::env::var("GITHUB_ACTIONS").as_deref()!=Ok("true") {return Err("Actions only".into());}
    let encoded=std::env::args().nth(1).ok_or("missing synthetic request")?;
    if encoded.len()>128*1024 {return Err("request limit".into());}
    let bytes=base64::engine::general_purpose::STANDARD.decode(encoded)?;
    let mut args:Value=serde_json::from_slice(&bytes)?;
    let operation=args.get("op").and_then(Value::as_str).ok_or("missing operation")?.to_owned();
    let spec=browser_tool_specs().into_iter().find(|spec|spec.op==operation).ok_or("unknown operation")?;
    args.as_object_mut().ok_or("request object required")?.remove("op");
    let port=Arc::new(ActionsResources::default());
    let safety=Arc::new(AtomicUsize::new(0));let observed_safety=safety.clone();
    let executor=ProductionBrowserToolExecutor::new(port.clone(),"primary")
        .with_shell_safety_callback(Arc::new(move |_request| {observed_safety.fetch_add(1,Ordering::SeqCst);Ok(())}));
    let bridge=SandBrowserToolBridge::new(Arc::new(NoDelegate),Arc::new(executor));
    let tool=bridge.list_tools()?.into_iter().find(|tool|tool.name==spec.name).ok_or("missing routed tool")?;
    let mut result=bridge.call_tool(&tool,args,"g11-actions")?;
    let stdout=fs::read_to_string("/tmp/.sand-browser/result-g11-actions.txt")?;
    let line=stdout.lines().rev().find_map(|line|line.strip_prefix("__SAND_BROWSER_RESULT__")).ok_or("missing actual driver result")?;
    let raw:Value=serde_json::from_str(line)?;
    let image=result.as_object_mut().ok_or("bridge result object")?.remove("imageB64");
    let image=image.as_ref().and_then(Value::as_str).map(|text|base64::engine::general_purpose::STANDARD.decode(text)).transpose()?;
    println!("{}",json!({"bridge":result,"driver":raw,"imageBytes":image.as_ref().map(Vec::len).unwrap_or(0),
        "imageSha256":image.as_ref().map(|bytes|format!("{:x}",Sha256::digest(bytes))),
        "io":{"shell":port.shells.load(Ordering::SeqCst),"write":port.writes.load(Ordering::SeqCst),"read":port.reads.load(Ordering::SeqCst),"safety":safety.load(Ordering::SeqCst)}}));
    Ok(())
}

}

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> { actions::run() }

#[cfg(not(unix))]
fn main() { panic!("managed Linux Actions environment required"); }

use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime,UNIX_EPOCH};

use mahayana_host_runtime::extensions::host_upgrade::host_bundle_source::*;
use mahayana_host_runtime::extensions::mcp::plugin_skills_cache::*;
use serde_json::json;

struct Fetcher{responses:Mutex<Vec<Result<HostBundleHttpResponse,String>>>,urls:Mutex<Vec<String>>}
impl HostBundleFetcher for Fetcher{
 fn fetch(&self,url:&str)->Result<HostBundleHttpResponse,String>{
  self.urls.lock().unwrap().push(url.into());
  self.responses.lock().unwrap().remove(0)
 }
}
fn scratch(label:&str)->std::path::PathBuf{
 let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
 std::env::temp_dir().join(format!("fabushi-cache-{label}-{}-{n}",std::process::id()))
}

#[test]
fn host_bundle_urls_validation_cache_and_tarball_fail_closed() {
 assert!(is_short_git_sha("abcdef0"));
 assert!(is_short_git_sha("0123456789abcdef0123456789abcdef01234567"));
 assert!(!is_short_git_sha("ABCDEF0"));
 assert!(!is_short_git_sha("abc"));
 assert_eq!(host_bundle_base_url_from(Some(" https://example.test/base/// ")),"https://example.test/base");
 assert_eq!(latest_host_bundle_version_url("https://x/"),"https://x/sand-host-bundle-latest.version");
 assert_eq!(host_bundle_tarball_url("abcdef0","https://x/"),"https://x/sand-host-bundle-abcdef0.tgz");

 let fetcher=Fetcher{responses:Mutex::new(vec![
  Ok(HostBundleHttpResponse{status:200,body:b"abcdef0\n".to_vec()}),
  Ok(HostBundleHttpResponse{status:200,body:b"bundle".to_vec()}),
 ]),urls:Mutex::new(Vec::new())};
 let cache=HostBundleVersionCache::default();
 let source=resolve_host_bundle_source_with(&fetcher,&cache,"https://x",100).unwrap();
 assert_eq!(source.version,"abcdef0");
 let cached=resolve_host_bundle_source_with(&fetcher,&cache,"https://x",200).unwrap();
 assert_eq!(cached.version,"abcdef0");
 assert_eq!(fetcher.urls.lock().unwrap().len(),1);
 assert_eq!(source.load_bundle_bytes(&fetcher).unwrap(),b"bundle");
 assert_eq!(fetcher.urls.lock().unwrap()[1],"https://x/sand-host-bundle-abcdef0.tgz");
 assert!(fetch_host_bundle_tarball_with_base(&fetcher,"bad","https://x").unwrap_err().to_string().contains("refusing malformed version"));
}

#[test]
fn plugin_skill_cache_reads_legacy_defaults_validates_and_writes_agent_readable_atomically() {
 assert!(is_safe_plugin_skill_id("linear-tools"));
 assert!(is_safe_plugin_skill_id("a1"));
 assert!(!is_safe_plugin_skill_id("-bad"));
 assert!(!is_safe_plugin_skill_id("Bad"));
 assert!(!is_safe_plugin_skill_id("bad--id"));

 let root=scratch("plugin");
 fs::create_dir_all(&root).unwrap();
 let cache_dir=get_plugin_skills_dir(&root);
 fs::create_dir_all(&cache_dir).unwrap();
 let path=get_plugin_skills_cache_path(&cache_dir);
 fs::write(&path,r#"{
   "fetchedAt": 12,
   "currentUserId": -1,
   "skills": [{
     "id": "linear-tools",
     "pluginId": "linear",
     "pluginName": "Linear",
     "name": "Issues",
     "description": "Manage issues",
     "filePath": "/tmp/SKILL.md",
     "publisherUserId": 7,
     "marketplaceTeamId": 0
   }],
   "authBlocked": [{"pluginId":"p","pluginName":"P"}]
 }"#).unwrap();
 let parsed=read_plugin_skills_cache(&cache_dir).unwrap();
 assert_eq!(parsed.fetched_at,12.0);
 assert_eq!(parsed.current_user_id,None);
 assert_eq!(parsed.skills[0].plugin_version,"");
 assert_eq!(parsed.skills[0].install_path,"");
 assert_eq!(parsed.skills[0].publisher_user_id,Some(7));
 assert_eq!(parsed.skills[0].marketplace_team_id,None);
 assert_eq!(parsed.auth_blocked.len(),1);

 write_plugin_skills_cache(&cache_dir,&PluginSkillsCacheWriteIndex{
  current_user_id:Some(Some(42)),
  skills:vec![json!({
   "id":"github","pluginId":"gh","pluginName":"GitHub","name":"GitHub",
   "description":"Use GitHub","filePath":"/tmp/GITHUB.md"
  })],
  auth_blocked:None,
 },||99.0).unwrap();
 let text=fs::read_to_string(&path).unwrap();
 assert_eq!(serde_json::from_str::<serde_json::Value>(&text).unwrap()["cacheFormatVersion"],json!(1));
 assert!(text.ends_with('\n'));
 assert!(text.contains("\"fetchedAt\": 99.0") || text.contains("\"fetchedAt\": 99"));
 assert!(!path.with_file_name("cache.json.tmp").exists());
 let parsed=read_plugin_skills_cache(&cache_dir).unwrap();
 assert_eq!(parsed.current_user_id,Some(42));
 assert!(parsed.auth_blocked.is_empty());
 #[cfg(unix)]
 {
  use std::os::unix::fs::PermissionsExt;
  assert_eq!(fs::metadata(&cache_dir).unwrap().permissions().mode()&0o777,0o755);
  assert_eq!(fs::metadata(&path).unwrap().permissions().mode()&0o777,0o644);
 }
 let _=fs::remove_dir_all(root);
}

#[test]
fn plugin_cache_rejects_malformed_denials_and_unknown_formats_without_rewriting() {
    let root = scratch("plugin-admission");
    fs::create_dir_all(&root).unwrap();
    let path = get_plugin_skills_cache_path(&root);
    let base = json!({
        "fetchedAt": 12,
        "skills": [{
            "id":"plugin-42-review", "pluginId":"42", "pluginName":"Review",
            "name":"Review", "description":"", "filePath":"/tmp/SKILL.md"
        }]
    });
    for denied in [
        json!(null), json!({}), json!("42"), json!(["42"]),
        json!([{"pluginId":"42"}]),
        json!([{"pluginId":"42","pluginName":"Review","marketplaceName":null}]),
    ] {
        for versioned in [false, true] {
            let mut value = base.clone();
            value["authBlocked"] = denied.clone();
            if versioned { value["cacheFormatVersion"] = json!(1); }
            let bytes = serde_json::to_vec(&value).unwrap();
            fs::write(&path, &bytes).unwrap();
            assert!(read_plugin_skills_cache(&root).is_none(), "{value}");
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    for version in [json!(0), json!(2), json!("1"), json!(null), json!({})] {
        let mut value = base.clone();
        value["cacheFormatVersion"] = version;
        value["authBlocked"] = json!([]);
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(read_plugin_skills_cache(&root).is_none(), "{value}");
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
    let mut missing = base.clone();
    missing["cacheFormatVersion"] = json!(1);
    fs::write(&path, serde_json::to_vec(&missing).unwrap()).unwrap();
    assert!(read_plugin_skills_cache(&root).is_none());
    for id in ["", " 42", "42 "] {
        let mut value = missing.clone();
        value["authBlocked"] = json!([{"pluginId":id,"pluginName":"Review"}]);
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(read_plugin_skills_cache(&root).is_none());
    }

    // Raw legacy diagnostics remain readable; executable consumers separately
    // refuse an ambiguous nonempty legacy block list.
    fs::write(&path, serde_json::to_vec(&base).unwrap()).unwrap();
    assert_eq!(read_plugin_skills_cache(&root).unwrap().skills.len(), 1);
    let mut legacy = base;
    legacy["authBlocked"] = json!([{"pluginId":"42","pluginName":"old slug"}]);
    fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(read_plugin_skills_cache(&root).unwrap().auth_blocked.len(), 1);
    let _ = fs::remove_dir_all(root);
}

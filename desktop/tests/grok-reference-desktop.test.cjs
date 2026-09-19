'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const {createReferenceDesktop}=require('../electron/grok-reference-desktop.cjs');

async function fixture(t){
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-ref-desktop-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const app={getPath(name){assert.equal(name,'userData');return root}};
  const win={isFullScreen:()=>false,isMaximized:()=>false};
  const desktop=createReferenceDesktop({
    app,
    BrowserWindow:{},
    shell:{},
    dialog:{showSaveDialog:async()=>({canceled:true})},
    safeStorage:{isEncryptionAvailable:()=>false},
    nativeTheme:{themeSource:'system',shouldUseDarkColors:true},
    getWindow:()=>win
  });
  Object.defineProperty(desktop,'__testRoot',{value:root});
  return desktop;
}

test('reference desktop exposes local window and theme state',async t=>{
  const desktop=await fixture(t);
  assert.deepEqual(await desktop.call('get-window-state'),{isFullscreen:false,isMaximized:false});
  assert.deepEqual(await desktop.call('theme-get'),{preference:'system',resolved:'dark'});
});

test('audio transcription derives OpenAI-compatible endpoint and returns measured text',async t=>{
  const desktop=await fixture(t);
  const before={
    url:process.env.FABUSHI_AGENT_API_URL,
    key:process.env.FABUSHI_AGENT_API_KEY,
    model:process.env.FABUSHI_TRANSCRIBE_MODEL,
    explicit:process.env.FABUSHI_TRANSCRIBE_URL
  };
  const originalFetch=global.fetch;
  t.after(()=>{
    for(const [key,value] of Object.entries({
      FABUSHI_AGENT_API_URL:before.url,
      FABUSHI_AGENT_API_KEY:before.key,
      FABUSHI_TRANSCRIBE_MODEL:before.model,
      FABUSHI_TRANSCRIBE_URL:before.explicit
    })){
      if(value===undefined)delete process.env[key];
      else process.env[key]=value;
    }
    global.fetch=originalFetch;
  });
  delete process.env.FABUSHI_TRANSCRIBE_URL;
  process.env.FABUSHI_AGENT_API_URL='https://example.invalid/v1/chat/completions';
  process.env.FABUSHI_AGENT_API_KEY='secret';
  process.env.FABUSHI_TRANSCRIBE_MODEL='whisper-test';
  let request=null;
  global.fetch=async(url,init)=>{
    request={url:String(url),init};
    return new Response(JSON.stringify({text:'spoken words'}),{status:200,headers:{'content-type':'application/json'}});
  };
  const result=await desktop.call('transcribe-audio',{audio:new Uint8Array([1,2,3]),mimeType:'audio/webm',language:'en'});
  assert.equal(request.url,'https://example.invalid/v1/audio/transcriptions');
  assert.equal(request.init.headers.authorization,'Bearer secret');
  assert.equal(result.text,'spoken words');
  assert.equal(Number.isFinite(result.transcriptionTimeMs),true);
});

test('audio transcription fails closed when no provider is configured',async t=>{
  const desktop=await fixture(t);
  const before={explicit:process.env.FABUSHI_TRANSCRIBE_URL,agent:process.env.FABUSHI_AGENT_API_URL};
  t.after(()=>{
    if(before.explicit===undefined)delete process.env.FABUSHI_TRANSCRIBE_URL;else process.env.FABUSHI_TRANSCRIBE_URL=before.explicit;
    if(before.agent===undefined)delete process.env.FABUSHI_AGENT_API_URL;else process.env.FABUSHI_AGENT_API_URL=before.agent;
  });
  delete process.env.FABUSHI_TRANSCRIBE_URL;
  delete process.env.FABUSHI_AGENT_API_URL;
  await assert.rejects(()=>desktop.call('transcribe-audio',{audio:new Uint8Array([1])}),/not configured/i);
});


test('local-tool approval history is persisted and can be cleared',async t=>{
  const desktop=await fixture(t);
  assert.equal(await desktop.call('local-tool-permission-ceiling'),null);
  await desktop.call('local-tool-approval-record',{approvalId:'approval-1',action:{kind:'shell'},target:{command:'pwd'}});
  let raw=JSON.parse(await fs.readFile(path.join(desktop.__testRoot,'grok-reference-desktop.json'),'utf8'));
  assert.equal(raw.localToolApprovals['approval-1'].action.kind,'shell');
  assert.equal(raw.localToolApprovals['approval-1'].target.command,'pwd');
  assert.ok(Number.isFinite(raw.localToolApprovals['approval-1'].recordedAt));
  await desktop.call('local-tool-approval-clear');
  raw=JSON.parse(await fs.readFile(path.join(desktop.__testRoot,'grok-reference-desktop.json'),'utf8'));
  assert.deepEqual(raw.localToolApprovals,{});
});


test('plugin logo bridge accepts bounded image responses and rejects non-images',async t=>{
  const desktop=await fixture(t),originalFetch=global.fetch;
  t.after(()=>{global.fetch=originalFetch});
  global.fetch=async()=>new Response(new Uint8Array([137,80,78,71]),{status:200,headers:{'content-type':'image/png'}});
  const logo=await desktop.call('plugin-logo',{url:'https://example.com/plugin.png'});
  assert.match(logo,/^data:image\/png;base64,/);
  global.fetch=async()=>new Response('not image',{status:200,headers:{'content-type':'text/plain'}});
  assert.equal(await desktop.call('plugin-logo',{url:'https://example.com/plugin.txt'}),null);
  assert.equal(await desktop.call('plugin-logo',{url:'file:///tmp/plugin.png'}),null);
});


test('reference telemetry stays local and produces an auditable NDJSON record',async t=>{
  const desktop=await fixture(t);
  const result=await desktop.call('telemetry-report',{name:'reportOnboardingStep',payload:{step:'done'}});
  assert.deepEqual(result,{ok:true,localOnly:true});
  const file=path.join(desktop.__testRoot,'grok-reference-telemetry.ndjson');
  const rows=(await fs.readFile(file,'utf8')).trim().split('\n').map(JSON.parse);
  assert.equal(rows.at(-1).name,'reportOnboardingStep');
  assert.deepEqual(rows.at(-1).payload,{step:'done'});
});

'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const fsp=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const {EventEmitter}=require('node:events');
const {PassThrough}=require('node:stream');
const {createLocalTeachRecording,SAND_TEACH_MAX_DURATION_MS}=require('../electron/grok-local-teach-recording.cjs');

class FakeChild extends EventEmitter{
  constructor(){
    super();this.stdout=new PassThrough();this.stderr=new PassThrough();this.killed=false;
  }
  kill(signal){
    if(this.killed)return false;
    this.killed=true;
    process.nextTick(()=>this.emit('exit',0,null));
    return true;
  }
}

test('local Teach recording creates a real session, exact status shape, and saved callback',async t=>{
  const root=await fsp.mkdtemp(path.join(os.tmpdir(),'fabushi-teach-'));
  t.after(()=>fsp.rm(root,{recursive:true,force:true}));
  let saved=null,spawned=null;
  const service=createLocalTeachRecording({
    app:{getPath(name){assert.equal(name,'userData');return root}},
    helperResolver:async()=>'/fake/fabushi-computer-helper',
    startupGraceMs:5,
    stopGraceMs:100,
    spawnImpl(_helper,args){
      spawned=args;
      fs.mkdirSync(path.dirname(args[1]),{recursive:true});
      fs.writeFileSync(args[1],Buffer.alloc(128,1));
      return new FakeChild();
    },
    onSaved:async record=>{saved=record}
  });

  assert.deepEqual(service.status(),{state:'idle',agentId:null,startedAtMs:null,maxDurationMs:SAND_TEACH_MAX_DURATION_MS});
  const active=await service.start({agentId:'agent-1',entryPoint:'fullscreen_title_bar'});
  assert.equal(active.state,'recording');
  assert.equal(active.agentId,'agent-1');
  assert.equal(active.maxDurationMs,600000);
  assert.equal(spawned[0],'record');
  assert.match(spawned[1],/demo\.mov$/);
  assert.equal(spawned[2],'600');

  const idle=await service.stop({agentId:'agent-1',save:true});
  assert.equal(idle.state,'idle');
  assert.equal(idle.agentId,null);
  assert.ok(saved);
  assert.equal(saved.agentId,'agent-1');
  assert.equal(saved.entryPoint,'fullscreen_title_bar');
  assert.ok(saved.videoBytes>=128);
  const metadata=JSON.parse(await fsp.readFile(path.join(saved.sessionDir,'session.json'),'utf8'));
  assert.equal(metadata.videoPath,saved.videoPath);
  assert.equal(metadata.maxDurationMs,600000);
  await service.dispose();
});

test('discarded Teach recording removes the local session and does not dispatch learning',async t=>{
  const root=await fsp.mkdtemp(path.join(os.tmpdir(),'fabushi-teach-discard-'));
  t.after(()=>fsp.rm(root,{recursive:true,force:true}));
  let savedCount=0;
  const service=createLocalTeachRecording({
    app:{getPath(){return root}},
    helperResolver:async()=>'/fake/fabushi-computer-helper',
    startupGraceMs:5,
    stopGraceMs:100,
    spawnImpl(_helper,args){
      fs.mkdirSync(path.dirname(args[1]),{recursive:true});
      fs.writeFileSync(args[1],Buffer.alloc(64,2));
      return new FakeChild();
    },
    onSaved:async()=>{savedCount+=1}
  });
  await service.start({agentId:'agent-2',entryPoint:'screen_hover'});
  const sessionRoot=service.root;
  assert.equal((await fsp.readdir(sessionRoot)).length,1);
  await service.stop({agentId:'agent-2',save:false});
  assert.equal(savedCount,0);
  assert.deepEqual(await fsp.readdir(sessionRoot),[]);
  assert.equal(service.status().state,'idle');
});

test('Teach recording fails closed when the native recorder exits during startup',async t=>{
  const root=await fsp.mkdtemp(path.join(os.tmpdir(),'fabushi-teach-fail-'));
  t.after(()=>fsp.rm(root,{recursive:true,force:true}));
  const service=createLocalTeachRecording({
    app:{getPath(){return root}},
    helperResolver:async()=>'/fake/fabushi-computer-helper',
    startupGraceMs:30,
    spawnImpl(){
      const child=new FakeChild();
      child.stderr.end('screen recording permission denied');
      process.nextTick(()=>child.emit('exit',5,null));
      return child;
    }
  });
  await assert.rejects(service.start({agentId:'agent-3'}),/could not start|permission denied/i);
  assert.equal(service.status().state,'idle');
});

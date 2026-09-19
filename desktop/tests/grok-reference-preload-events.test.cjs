'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');

const preload=fs.readFileSync(path.join(__dirname,'..','electron','preload.cjs'),'utf8');
const main=fs.readFileSync(path.join(__dirname,'..','electron','main.cjs'),'utf8');

test('reference desktop interaction hooks are not silent preload stubs',()=>{
  for(const forbidden of [
    "onOpenFeedback:()=>noopUnsubscribe",
    "onOpenAbout:()=>noopUnsubscribe",
    "onForceOnboarding:()=>noopUnsubscribe",
    "transcribeAudio:async()=>({text:''})",
    "onWindowStateEvent:()=>noopUnsubscribe",
    "onZoomFactorEvent:()=>noopUnsubscribe",
    "theme:{initial:{preference:'system',resolved:'dark'},get:()=>refDesktop('theme-get'),set:preference=>refDesktop('theme-set',{preference}),onChanged:()=>noopUnsubscribe}"
  ])assert.equal(preload.includes(forbidden),false,forbidden);
  assert.match(preload,/transcribeAudio:\(audio,mimeType,language\)=>refDesktop\('transcribe-audio'/);
  assert.match(preload,/onOpenAbout:listener=>refListen\('open-about'/);
  assert.match(preload,/onWindowStateEvent:listener=>refListen\('window-state'/);
  assert.match(preload,/onZoomFactorEvent:listener=>refListen\('zoom-factor-changed'/);
  assert.match(preload,/onChanged:listener=>refListen\('theme-changed'/);
});

test('Electron main emits the native events consumed by the exact renderer',()=>{
  assert.match(main,/installReferenceApplicationMenu/);
  assert.match(main,/sendReferenceEvent\('open-about'\)/);
  assert.match(main,/sendReferenceEvent\('open-feedback'\)/);
  assert.match(main,/sendReferenceEvent\('force-onboarding'\)/);
  assert.match(main,/sendReferenceEvent\('window-state'/);
  assert.match(main,/sendReferenceEvent\('zoom-factor-changed'/);
  assert.match(main,/sendReferenceEvent\('theme-changed'/);
});

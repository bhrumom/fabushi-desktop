'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {isTransientError,retryAfterMs,backoff,runWithTransientRetry}=require('../electron/grok-transient-retry.cjs');

test('transient classifier recognizes network/provider failures but not ordinary validation errors',()=>{
  assert.equal(isTransientError(Object.assign(Error('socket hang up'),{code:'ECONNRESET'})),true);
  assert.equal(isTransientError(Object.assign(Error('provider busy'),{retryable:true})),true);
  assert.equal(isTransientError(Error('invalid request')),false);
});
test('retry-after and bounded backoff follow reference retry shape',()=>{
  const error=Object.assign(Error('busy'),{metadata:new Map([['retry-after','2']])});
  assert.equal(retryAfterMs(error),2000);
  assert.equal(backoff(1,{baseDelayMs:1000,maxDelayMs:5000,random:()=>0}),500);
});
test('retry loop retries only bounded transient failures',async()=>{
  let calls=0;const delays=[];
  const value=await runWithTransientRetry(async()=>{calls++;if(calls<3)throw Object.assign(Error('reset'),{code:'ECONNRESET'});return'ok'},{maxAttempts:3,baseDelayMs:10,maxDelayMs:20,random:()=>0,sleep:async ms=>{delays.push(ms)}});
  assert.equal(value,'ok');assert.equal(calls,3);assert.deepEqual(delays,[5,10]);
});

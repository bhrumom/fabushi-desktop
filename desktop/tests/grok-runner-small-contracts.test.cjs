'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const {SEND_DISPATCH_MAX_PLAUSIBLE_MS,TTFT_MAX_PLAUSIBLE_MS,sanitizeCrossClockDurationMs,bucketClockSkewDeltaMs}=require('../electron/grok-clock-skew-guard.cjs');
const {bytesLookLikeVideoContainer}=require('../electron/grok-video-container.cjs');

test('clock-skew guard mirrors recovered duration ceilings and buckets',()=>{
 assert.equal(SEND_DISPATCH_MAX_PLAUSIBLE_MS,120000);assert.equal(TTFT_MAX_PLAUSIBLE_MS,1800000);
 assert.deepEqual(sanitizeCrossClockDurationMs(10.6,100),{ms:11});
 assert.deepEqual(sanitizeCrossClockDurationMs(-1,100),{skewReason:'negative'});
 assert.deepEqual(sanitizeCrossClockDurationMs(101,100),{skewReason:'too_large'});
 assert.equal(bucketClockSkewDeltaMs(-900),'neg_le_1s');assert.equal(bucketClockSkewDeltaMs(100000),'le_5m');
});
test('video-container probe recognizes recovered container signatures',()=>{
 const mp4=new Uint8Array(12);mp4.set([102,116,121,112],4);assert.equal(bytesLookLikeVideoContainer(mp4),true);
 assert.equal(bytesLookLikeVideoContainer(Uint8Array.from([26,69,223,163,0])),true);
 assert.equal(bytesLookLikeVideoContainer(Uint8Array.from([1,2,3,4,5])),false);
});

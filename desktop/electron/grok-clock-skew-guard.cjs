'use strict';

const SEND_DISPATCH_MAX_PLAUSIBLE_MS=120_000;
const TTFT_MAX_PLAUSIBLE_MS=1_800_000;
function sanitizeCrossClockDurationMs(rawDeltaMs,ceilingMs){
  if(!Number.isFinite(rawDeltaMs))return{skewReason:'too_large'};
  if(rawDeltaMs<0)return{skewReason:'negative'};
  if(rawDeltaMs>ceilingMs)return{skewReason:'too_large'};
  return{ms:Math.round(rawDeltaMs)};
}
function bucketClockSkewDeltaMs(raw){
  if(!Number.isFinite(raw))return'nonfinite';
  if(raw<0){const n=-raw;return n<=1_000?'neg_le_1s':n<=60_000?'neg_le_1m':'neg_gt_1m'}
  return raw<=60_000?'le_1m':raw<=300_000?'le_5m':raw<=900_000?'le_15m':'gt_15m';
}
module.exports={SEND_DISPATCH_MAX_PLAUSIBLE_MS,TTFT_MAX_PLAUSIBLE_MS,sanitizeCrossClockDurationMs,bucketClockSkewDeltaMs};

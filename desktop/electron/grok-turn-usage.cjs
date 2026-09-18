'use strict';

const MAX=Number.MAX_SAFE_INTEGER;
function safeCount(value){const number=Number(value);if(!Number.isFinite(number)||number<=0)return 0;return Math.min(MAX,Math.trunc(number))}
function addCounts(a,b){return Math.min(MAX,safeCount(a)+safeCount(b))}
function normalizeTurnUsage(value){
  if(!value||typeof value!=='object')return undefined;
  const input=value.inputTokens??value.prompt_tokens??value.input_tokens;
  const output=value.outputTokens??value.completion_tokens??value.output_tokens;
  const cacheRead=value.cacheReadTokens??value.prompt_tokens_details?.cached_tokens??value.input_tokens_details?.cached_tokens;
  const cacheWrite=value.cacheWriteTokens??value.prompt_tokens_details?.cache_creation_tokens??value.input_tokens_details?.cache_creation_tokens;
  const reasoning=value.reasoningTokens??value.completion_tokens_details?.reasoning_tokens??value.output_tokens_details?.reasoning_tokens;
  if([input,output,cacheRead,cacheWrite,reasoning].every(item=>item==null))return undefined;
  return{inputTokens:safeCount(input),outputTokens:safeCount(output),cacheReadTokens:safeCount(cacheRead),cacheWriteTokens:safeCount(cacheWrite),...(reasoning==null?{}:{reasoningTokens:safeCount(reasoning)})};
}
function mergeTurnUsage(a,b){
  if(!a)return b;if(!b)return a;
  const reasoning=a.reasoningTokens==null&&b.reasoningTokens==null?undefined:addCounts(a.reasoningTokens||0,b.reasoningTokens||0);
  return{inputTokens:addCounts(a.inputTokens,b.inputTokens),outputTokens:addCounts(a.outputTokens,b.outputTokens),cacheReadTokens:addCounts(a.cacheReadTokens,b.cacheReadTokens),cacheWriteTokens:addCounts(a.cacheWriteTokens,b.cacheWriteTokens),...(reasoning==null?{}:{reasoningTokens:reasoning})};
}
module.exports={safeCount,addCounts,normalizeTurnUsage,mergeTurnUsage};

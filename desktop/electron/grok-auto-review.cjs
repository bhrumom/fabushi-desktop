'use strict';

const crypto=require('node:crypto');

const MUTATING_SURFACE_TOOLS=new Set([
  'open_url',
  'browser_navigate','browser_click','browser_type','browser_key',
  'computer_click','computer_mouse_move','computer_drag','computer_scroll','computer_type','computer_key'
]);

function autoReviewSurface(name){
  const value=String(name||'');
  if(value==='open_url'||value.startsWith('browser_'))return'browser';
  if(value.startsWith('computer_'))return'computer';
  return null;
}
function requiresAutoReview(name){return MUTATING_SURFACE_TOOLS.has(String(name||''))}
function bounded(value,max){
  const text=String(value??'');
  return text.length>max?text.slice(0,max):text;
}
function canonicalAutoReviewTarget(name,args={}){
  const surface=autoReviewSurface(name);
  if(!surface)return null;
  const exact={};
  for(const [key,value] of Object.entries(args||{})){
    if(value==null)continue;
    if(typeof value==='string')exact[key]=bounded(value,key==='purpose'?500:key==='text'?2000:1000);
    else if(typeof value==='number'||typeof value==='boolean')exact[key]=value;
    else if(Array.isArray(value))exact[key]=value.slice(0,64);
  }
  return{
    surface,
    action:String(name),
    exactAction:exact,
    declaredPurpose:bounded(args?.purpose||'',500)||undefined,
    displayStateIdentity:bounded(args?.stateId||'',256)||undefined
  };
}
function fingerprintAutoReviewTarget(target){
  return crypto.createHash('sha256').update(JSON.stringify(target)).digest('hex');
}
function normalizeAutoReviewMode(value){
  return ['off','shadow','enforce'].includes(value)?value:'enforce';
}
function normalizeClassifierDecision(value){
  if(!value||typeof value!=='object')return null;
  const kind=value.kind==='allow'?'allow':value.kind==='block'?'block':null;
  if(!kind)return null;
  return{kind,reason:bounded(value.reason||'',1000)|| (kind==='allow'?'Action allowed by auto-review.':'Action needs manual review.')};
}
module.exports={
  autoReviewSurface,requiresAutoReview,canonicalAutoReviewTarget,fingerprintAutoReviewTarget,
  normalizeAutoReviewMode,normalizeClassifierDecision
};

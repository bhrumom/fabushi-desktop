'use strict';

const crypto=require('node:crypto');

const MUTATING_SURFACE_TOOLS=new Set([
  'browser_navigate','browser_click','browser_mouse_click_xy','browser_type','browser_fill','browser_select_option',
  'browser_press_key','browser_drag','browser_cdp','browser_tabs','Computer'
]);

function autoReviewSurface(name){
  const value=String(name||'');
  if(value.startsWith('browser_'))return'browser';
  if(value==='Computer'||value==='Screenshot')return'computer';
  return null;
}
function requiresAutoReview(name,args={}){
  const value=String(name||'');
  if(value==='Computer'){
    const actions=[args,...(Array.isArray(args?.then)?args.then:[])];
    return actions.some(row=>['click','move','drag','type','key','scroll'].includes(String(row?.action||'')));
  }
  if(value==='browser_tabs')return String(args?.action||'list')!=='list';
  return MUTATING_SURFACE_TOOLS.has(value);
}
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
    declaredPurpose:bounded(args?.description||args?.purpose||'',500)||undefined
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

'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');

const BOT_BLOCK_SIGNATURES=[
  {family:'google_sorry',confidence:'high',host:['google.com'],pathPrefix:'/sorry'},
  {family:'google_signin_rejected',confidence:'high',host:['accounts.google.com'],pathIncludes:'/signin/rejected'},
  {family:'cloudflare_challenge',confidence:'high',host:['challenges.cloudflare.com']},
  {family:'hcaptcha',confidence:'low',hostSuffix:['.hcaptcha.com']},
  {family:'arkose',confidence:'high',hostSuffix:['.arkoselabs.com','.funcaptcha.com']},
  {family:'linkedin_checkpoint',confidence:'high',host:['linkedin.com'],hostSuffix:['.linkedin.com'],pathPrefix:'/checkpoint/challenge'},
  {family:'datadome',confidence:'high',host:['captcha-delivery.com','captcha.datadome.co'],hostSuffix:['.captcha-delivery.com']},
  {family:'perimeterx',confidence:'high',pathIncludes:'/px/captcha'},
  {family:'vercel_checkpoint',confidence:'high',pathIncludes:'/.well-known/vercel/security/'}
];
function normalizeNavigationUrl(raw){
  let url;try{url=new URL(String(raw||'').trim())}catch{return undefined}
  if(!['http:','https:'].includes(url.protocol)||url.origin==='null')return undefined;
  return (url.origin+url.pathname).slice(0,1024);
}
function visitedSiteHost(raw){
  let url;try{url=new URL(String(raw||''))}catch{return undefined}
  return url.hostname.replace(/^www\./,'')||undefined;
}
function classifyBotBlockPage({url,title=''}) {
  let parsed;try{parsed=new URL(String(url||''))}catch{return undefined}
  const host=parsed.hostname.toLowerCase().replace(/^www\./,''),pathname=parsed.pathname,cleanTitle=String(title||'').trim();
  const titleHit=/^(just a moment|attention required! \| cloudflare|access to this page has been denied|pardon our interruption|access denied)/i.test(cleanTitle);
  if(titleHit)return{family:/cloudflare/i.test(cleanTitle)?'cloudflare_challenge':'generic_access_denied',confidence:/^access denied$/i.test(cleanTitle)?'low':'high',blockedHost:host.slice(0,100),blockedUrl:(parsed.origin+pathname).slice(0,1024)};
  for(const signature of BOT_BLOCK_SIGNATURES){
    const hostOk=(!signature.host&&!signature.hostSuffix)||(signature.host||[]).includes(host)||(signature.hostSuffix||[]).some(suffix=>host.endsWith(suffix));
    const pathOk=!signature.pathPrefix&&!signature.pathIncludes||(signature.pathPrefix&&pathname.startsWith(signature.pathPrefix))||(signature.pathIncludes&&pathname.includes(signature.pathIncludes));
    if(hostOk&&pathOk)return{family:signature.family,confidence:signature.confidence,blockedHost:host.slice(0,100),blockedUrl:(parsed.origin+pathname).slice(0,1024)};
  }
  return undefined;
}
function toolAuditAction(name,args,status,durationMs){
  const action={kind:'toolCall',toolName:String(name||'tool'),status,durationMs:Math.max(0,Math.round(Number(durationMs)||0))};
  if(String(name)==='Computer')action.computerAction=String(args?.action||'unknown');
  else if(String(name)==='Screenshot')action.computerAction='screenshot';
  else if(String(name).startsWith('computer_'))action.computerAction=String(name).slice('computer_'.length);
  if(String(name)==='browser_navigate'){
    const url=normalizeNavigationUrl(args?.url);if(url){action.kind='browserNavigation';action.url=url;const host=visitedSiteHost(url);if(host)action.host=host}
  }
  if(String(name).startsWith('browser_')&&action.kind==='toolCall')action.browserAction=String(name).slice('browser_'.length);
  return action;
}
function createActionAuditor({app}){
  const file=path.join(app.getPath('userData'),'audit','actions.ndjson');let writing=Promise.resolve();
  function record(record){
    const clean={...record,occurredAtMs:Number(record?.occurredAtMs)||Date.now(),action:record?.action&&typeof record.action==='object'?record.action:{kind:'unknown'}};
    const line=JSON.stringify(clean)+'\n';
    writing=writing.then(async()=>{await fs.mkdir(path.dirname(file),{recursive:true,mode:0o700});await fs.appendFile(file,line,{mode:0o600})}).catch(()=>{});
  }
  async function flush(){await writing}
  return{file,record,flush};
}
module.exports={normalizeNavigationUrl,visitedSiteHost,classifyBotBlockPage,toolAuditAction,createActionAuditor};

'use strict';

const SAND_MEDIA_SCHEME='sand-media';
const MEDIA_HOST='attachment';
const REMOTE_MEDIA_CHUNK_BYTES=4*1024*1024;
function buildSandMediaUrl(id){return SAND_MEDIA_SCHEME+'://'+MEDIA_HOST+'/'+encodeURIComponent(String(id||''))}
function parseSandMediaUrl(rawUrl){
  let url;try{url=new URL(String(rawUrl||''))}catch{return null}
  if(url.protocol!==SAND_MEDIA_SCHEME+':'||url.hostname!==MEDIA_HOST)return null;
  const segment=url.pathname.replace(/^\/+/, '');if(!segment)return null;
  try{return decodeURIComponent(segment)}catch{return null}
}
function parseRangeHeader(header,size){
  if(header==null)return null;const match=/^bytes=(\d*)-(\d*)$/.exec(String(header).trim());if(!match)return null;
  const startRaw=match[1]||'',endRaw=match[2]||'';if(!startRaw&&!endRaw)return null;
  let start,end;
  if(!startRaw){const suffix=Number.parseInt(endRaw,10);if(!Number.isFinite(suffix)||suffix<=0)return null;start=Math.max(0,size-suffix);end=size-1}
  else{start=Number.parseInt(startRaw,10);end=endRaw?Number.parseInt(endRaw,10):size-1}
  if(!Number.isFinite(start)||!Number.isFinite(end)||start>end||start<0||start>=size)return null;
  return{start,end:Math.min(end,size-1)};
}
function registerSandMediaScheme(protocol){
  protocol.registerSchemesAsPrivileged([{scheme:SAND_MEDIA_SCHEME,privileges:{standard:true,secure:true,supportFetchAPI:true,stream:true}}]);
}
function createSandMediaHandler({attachmentGateway}){
  if(!attachmentGateway||typeof attachmentGateway.readChunk!=='function')throw Error('Attachment gateway chunk reader is required.');
  return async function handle(request){
    const id=parseSandMediaUrl(request.url);if(!id)return new Response(null,{status:400});
    const meta=await attachmentGateway.readChunk(id,0,0);if(!meta||meta.totalSize<=0)return new Response(null,{status:404});
    const range=parseRangeHeader(request.headers.get('range'),meta.totalSize),start=range?.start??0,requestedEnd=range?.end??meta.totalSize-1;
    const chunk=await attachmentGateway.readChunk(id,start,Math.min(requestedEnd-start+1,REMOTE_MEDIA_CHUNK_BYTES));
    if(!chunk)return new Response(null,{status:404});
    if(chunk.totalSize<=0||start>=chunk.totalSize||chunk.data.length===0)return new Response(null,{status:416,headers:{'accept-ranges':'bytes','content-range':'bytes */'+Math.max(0,chunk.totalSize)}});
    const end=start+chunk.data.length-1,partial=range!=null||end<chunk.totalSize-1||start>0;
    const headers=new Headers({'content-type':chunk.mime||meta.mime||'application/octet-stream','content-length':String(chunk.data.length),'accept-ranges':'bytes','cache-control':'no-cache'});
    if(partial)headers.set('content-range','bytes '+start+'-'+end+'/'+chunk.totalSize);
    return new Response(new Uint8Array(chunk.data),{status:partial?206:200,headers});
  };
}
function registerSandMediaProtocol(protocol,attachmentGateway){protocol.handle(SAND_MEDIA_SCHEME,createSandMediaHandler({attachmentGateway}))}
module.exports={SAND_MEDIA_SCHEME,MEDIA_HOST,REMOTE_MEDIA_CHUNK_BYTES,buildSandMediaUrl,parseSandMediaUrl,parseRangeHeader,registerSandMediaScheme,createSandMediaHandler,registerSandMediaProtocol};

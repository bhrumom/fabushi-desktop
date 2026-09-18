'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');

const TOOL_OUTPUT_THRESHOLD_BYTES=40_000;
const MAX_OUTPUT_FILE_SIZE=1_000_000;

function enabled(env=process.env){
  return env.SAND_DISABLE_LARGE_OUTPUT_SPILL!=='1'&&env.FABUSHI_DISABLE_LARGE_OUTPUT_SPILL!=='1';
}
function safeSegment(value){
  return String(value||'agent').replace(/[^A-Za-z0-9._-]+/g,'-').replace(/^-+|-+$/g,'').slice(0,96)||'agent';
}
function describe(loc,lead='Content'){
  const size=loc.sizeBytes>=1024?(loc.sizeBytes/1024).toFixed(1)+' KB':loc.sizeBytes+' bytes';
  return lead+' written to file: '+loc.filePath+'\nSize: '+size+', '+loc.lineCount+' lines'+(loc.truncated?' (truncated to '+MAX_OUTPUT_FILE_SIZE+' characters)':'');
}
function createOutputSpiller({app,thresholdBytes=TOOL_OUTPUT_THRESHOLD_BYTES,maxFileSize=MAX_OUTPUT_FILE_SIZE}){
  const root=path.join(app.getPath('userData'),'agent-tools');
  async function spillText(text,{agentId='agent',toolCallId='tool',lead='Content'}={}){
    const source=String(text??'');
    const bytes=Buffer.byteLength(source,'utf8');
    if(!enabled()||thresholdBytes<=0||bytes<=thresholdBytes)return{text:source,outputLocation:null,spilled:false};
    const capped=source.length>maxFileSize?source.slice(0,maxFileSize):source;
    const dir=path.join(root,safeSegment(agentId));
    await fs.mkdir(dir,{recursive:true,mode:0o700});
    const filePath=path.join(dir,crypto.randomUUID()+'.txt');
    await fs.writeFile(filePath,capped,{mode:0o600});
    const location={
      filePath,
      sizeBytes:Buffer.byteLength(capped,'utf8'),
      lineCount:capped.split('\n').length,
      truncated:capped.length<source.length,
      originalSizeBytes:bytes,
      toolCallId:String(toolCallId||'')
    };
    return{text:describe(location,lead),outputLocation:location,spilled:true};
  }
  return{root,spillText};
}

module.exports={TOOL_OUTPUT_THRESHOLD_BYTES,MAX_OUTPUT_FILE_SIZE,createOutputSpiller,describe};

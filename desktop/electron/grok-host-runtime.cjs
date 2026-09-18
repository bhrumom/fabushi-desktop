'use strict';

const crypto=require('node:crypto');
const {toolDefinitions,executeTool,descriptor}=require('./local-tool-executor.cjs');

function abortError(message='Operation cancelled.'){const error=Error(message);error.name='AbortError';return error;}
function isAbort(error,signal){return signal?.aborted||error?.name==='AbortError'||error?.code==='ABORT_ERR';}

function createHostRuntime({shell,getLocalToolPermission,requestApproval,onToolState,onAgentStatus,getExternalTools=async()=>[],executeExternalTool=async()=>null,getWorkflowContext=async()=>'',subagents=null,browser=null}){
  function systemPrompt(agent,enabled,workflowContext){
    const parts=[
      `You are ${agent.name}, a Fabushi desktop agent following the Grok Bot host/coordinator execution model.`,
      'You operate the Mac where Fabushi is installed, not a cloud computer.',
      'Use the provided tools for computer work and report only results confirmed by tool output.',
      'Inspect before mutation. Mutating tools can be blocked or require explicit user approval.',
      'When a Computer click is needed, provide a concise purpose in the tool arguments.',
      `Enabled local capabilities: ${[...enabled].join(', ')||'none'}.`
    ];
    if(workflowContext)parts.push(workflowContext);
    return parts.join('\n');
  }

  function subagentDefinitions(){
    if(!subagents)return[];
    const fn=(name,description,properties,required=[])=>({type:'function',function:{name,description,parameters:{type:'object',properties,required,additionalProperties:false}}});
    return[
      fn('create_subagent','Create a delegated agent. It can run in the background or wait for a completed result.',{name:{type:'string'},prompt:{type:'string'},background:{type:'boolean'}},['name','prompt']),
      fn('check_subagent','Inspect a delegated agent status and recent transcript.',{agentId:{type:'string'}},['agentId']),
      fn('message_subagent','Send follow-up work to a delegated agent. Set interrupt=true to stop its current run before sending.',{agentId:{type:'string'},prompt:{type:'string'},interrupt:{type:'boolean'}},['agentId','prompt']),
      fn('stop_subagent','Abort a running delegated agent.',{agentId:{type:'string'}},['agentId'])
    ];
  }
  async function executeSubagentTool(name,args,signal,parentAgentId){
    if(!subagents)return null;
    if(name==='create_subagent')return{text:JSON.stringify(await subagents.create({parentAgentId,name:args.name,prompt:args.prompt,background:args.background===true,signal}),null,2)};
    if(name==='check_subagent')return{text:JSON.stringify(await subagents.check({agentId:args.agentId}),null,2)};
    if(name==='message_subagent')return{text:JSON.stringify(await subagents.message({parentAgentId,agentId:args.agentId,prompt:args.prompt,interrupt:args.interrupt===true,signal}),null,2)};
    if(name==='stop_subagent')return{text:JSON.stringify(await subagents.stop({agentId:args.agentId}),null,2)};
    return null;
  }

  async function chatRequest(messages,tools,signal){
    const endpoint=String(process.env.FABUSHI_AGENT_API_URL||'').trim();
    const key=String(process.env.FABUSHI_AGENT_API_KEY||'').trim();
    const model=String(process.env.FABUSHI_AGENT_MODEL||'gpt-5.6').trim();
    if(!endpoint||!key)return{offline:true};
    const response=await fetch(endpoint,{
      method:'POST',
      headers:{'content-type':'application/json',authorization:'Bearer '+key},
      body:JSON.stringify({model,messages,tools:tools.length?tools:undefined,tool_choice:tools.length?'auto':undefined,stream:false}),
      signal
    });
    if(!response.ok)throw Error('Inference HTTP '+response.status);
    const body=await response.json();
    const message=body?.choices?.[0]?.message;
    if(!message||typeof message!=='object')throw Error('Inference returned no assistant message');
    return{message};
  }

  async function updateTool(agentId,entry,patch){
    Object.assign(entry,patch,{updatedAt:Date.now()});
    await onToolState({agentId,entry});
  }

  async function authorize(agentId,entry,name,args,signal){
    const meta=descriptor(name,args);
    if(!meta.mutation)return true;
    const permission=await getLocalToolPermission();
    if(permission==='never'){
      await updateTool(agentId,entry,{status:'error',text:'Blocked by local tool permission policy.',errorCode:'permission-denied'});
      return false;
    }
    if(permission==='always')return true;
    await updateTool(agentId,entry,{status:'waiting-approval',text:meta.summary,approvalSummary:meta.summary});
    await onAgentStatus(agentId,'waiting');
    const approved=await requestApproval({agentId,messageId:entry.id,toolName:name,summary:meta.summary,args,signal});
    if(signal?.aborted)throw abortError();
    await onAgentStatus(agentId,'running');
    if(!approved){
      await updateTool(agentId,entry,{status:'cancelled',text:'Denied by user.',errorCode:'approval-denied'});
      return false;
    }
    return true;
  }

  async function runTurn({agent,history,transcript,enabled,signal}){
    const externalDefinitions=await getExternalTools();
    const externalNames=new Set(externalDefinitions.map(x=>x.function?.name).filter(Boolean));
    const subagentTools=subagentDefinitions();
    const subagentNames=new Set(subagentTools.map(x=>x.function.name));
    const browserTools=browser?.definitions(enabled)||[];
    const browserNames=new Set(browserTools.map(x=>x.function.name));
    const tools=[...toolDefinitions(enabled),...browserTools,...subagentTools,...externalDefinitions.map(({_mcp,...definition})=>definition)];
    const latestUser=[...history].reverse().find(x=>x.role==='user');
    const workflowContext=await getWorkflowContext(latestUser?.text||'');
    const messages=[
      {role:'system',content:systemPrompt(agent,enabled,workflowContext)},
      ...history.filter(x=>x.role==='user'||x.role==='assistant').slice(-40).map(x=>({role:x.role,content:x.text}))
    ];

    for(let round=0;round<12;round++){
      if(signal?.aborted)throw abortError();
      const result=await chatRequest(messages,tools,signal);
      if(result.offline){
        return 'Agent host is ready on this Mac. Configure FABUSHI_AGENT_API_URL, FABUSHI_AGENT_API_KEY, and optionally FABUSHI_AGENT_MODEL to connect a tool-calling model.';
      }
      const msg=result.message;
      const calls=Array.isArray(msg.tool_calls)?msg.tool_calls:[];
      if(!calls.length){
        const content=typeof msg.content==='string'?msg.content.trim():'';
        if(content)return content;
        throw Error('Inference returned neither text nor tool calls');
      }

      messages.push({role:'assistant',content:msg.content||null,tool_calls:calls});
      for(const call of calls){
        if(signal?.aborted)throw abortError();
        const name=String(call?.function?.name||'tool');
        let args={};
        try{args=JSON.parse(call?.function?.arguments||'{}')}catch{}
        const entry={
          id:crypto.randomUUID(),role:'tool',toolName:name,text:'Queued',createdAt:Date.now(),updatedAt:Date.now(),
          status:'queued',arguments:args
        };
        transcript.push(entry);
        await onToolState({agentId:agent.id,entry});

        let resultText='';
        try{
          const allowed=externalNames.has(name)||subagentNames.has(name)||(browserNames.has(name)&&browser?.isMutation(name)===false)?true:await authorize(agent.id,entry,name,args,signal);
          if(!allowed){
            resultText='ERROR: '+entry.text;
          }else{
            await updateTool(agent.id,entry,{status:'running',text:'Running…'});
            const execution=await executeTool(name,args,{
              enabled,shell,signal,
              onStarted:()=>{}
            });
            resultText=execution?.text||'(completed)';
            await updateTool(agent.id,entry,{status:'done',text:resultText,...(execution?.display?{display:execution.display}:{})});
          }
        }catch(error){
          if(isAbort(error,signal)){
            await updateTool(agent.id,entry,{status:'cancelled',text:'Cancelled.',errorCode:'cancelled'});
            throw abortError();
          }
          resultText='ERROR: '+(error instanceof Error?error.message:String(error));
          await updateTool(agent.id,entry,{status:'error',text:resultText,errorCode:'tool-error'});
        }
        messages.push({role:'tool',tool_call_id:call.id,content:resultText});
      }
    }
    throw Error('Agent exceeded the tool-call round limit');
  }

  return{runTurn};
}

module.exports={createHostRuntime};

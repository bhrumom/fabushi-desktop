'use strict';

function createResource(name){
  return Object.freeze({name:String(name),symbol:Symbol(String(name))});
}

class ResourceRegistry {
  constructor(parent=null){this.parent=parent;this.values=new Map()}
  register(resource,implementation){
    if(!resource?.symbol)throw Error('Invalid execution resource.');
    this.values.set(resource.symbol,{resource,implementation});
    return this;
  }
  has(resource){return this.values.has(resource.symbol)||(this.parent?.has(resource)??false)}
  get(resource){
    const local=this.values.get(resource.symbol);
    if(local)return local.implementation;
    if(this.parent)return this.parent.get(resource);
    throw Error('Execution resource is unavailable: '+resource.name);
  }
  entries(){return[...this.values.values()].map(value=>[value.resource,value.implementation])}
  child(){return new ResourceRegistry(this)}
}

const LOCAL_TOOL_EXECUTOR=createResource('local-tool-executor');
const BROWSER_TOOL_EXECUTOR=createResource('browser-tool-executor');
const EXTERNAL_TOOL_EXECUTOR=createResource('external-tool-executor');
const SUBAGENT_TOOL_EXECUTOR=createResource('subagent-tool-executor');

function createExecutionResources({executeLocal,executeBrowser,executeExternal,executeSubagent}){
  const registry=new ResourceRegistry();
  if(executeLocal)registry.register(LOCAL_TOOL_EXECUTOR,Object.freeze({execute:executeLocal}));
  if(executeBrowser)registry.register(BROWSER_TOOL_EXECUTOR,Object.freeze({execute:executeBrowser}));
  if(executeExternal)registry.register(EXTERNAL_TOOL_EXECUTOR,Object.freeze({execute:executeExternal}));
  if(executeSubagent)registry.register(SUBAGENT_TOOL_EXECUTOR,Object.freeze({execute:executeSubagent}));
  return registry;
}

module.exports={
  createResource,ResourceRegistry,
  LOCAL_TOOL_EXECUTOR,BROWSER_TOOL_EXECUTOR,EXTERNAL_TOOL_EXECUTOR,SUBAGENT_TOOL_EXECUTOR,
  createExecutionResources
};

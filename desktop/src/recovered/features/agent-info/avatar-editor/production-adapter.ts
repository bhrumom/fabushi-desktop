import type { GrokAgentBridge } from "../../../../grok-types";
import { createAvatarEditorController, type AvatarEditorAgent, type AvatarEditorController } from "./controller";
import type { AvatarImageCodec } from "./model";

export interface AvatarEditorProductionScope {
  readonly accountKey:string|null;
  readonly agent:AvatarEditorAgent|null;
}
export type AvatarEditorProductionStatus="ready"|"signed-out"|"missing-agent"|"bridge-unavailable";
export interface AvatarEditorProductionSnapshot {
  readonly accountKey:string|null;
  readonly agentId:string|null;
  readonly generation:number;
  readonly status:AvatarEditorProductionStatus;
  readonly controller:AvatarEditorController|null;
}
export interface AvatarEditorProductionAdapter {
  getSnapshot():AvatarEditorProductionSnapshot;
  subscribe(listener:()=>void):()=>void;
  setScope(scope:AvatarEditorProductionScope):void;
  reset():void;
  dispose():void;
}
export interface AvatarEditorProductionAdapterOptions {
  readonly bridge:Pick<GrokAgentBridge,"pickAvatarFile"|"generateAgentAvatarImage"|"setAgentAvatarBytes"|"updateAgent">;
  readonly codec?:AvatarImageCodec;
}
function callable(value:unknown):value is (...args:never[])=>unknown{return typeof value==="function"}
function validScope(scope:AvatarEditorProductionScope):boolean{
  return typeof scope.accountKey==="string"&&scope.accountKey.length>0&&scope.agent!=null&&typeof scope.agent.id==="string"&&scope.agent.id.length>0;
}
export function createAvatarEditorProductionAdapter(options:AvatarEditorProductionAdapterOptions):AvatarEditorProductionAdapter{
  let scope:AvatarEditorProductionScope={accountKey:null,agent:null};
  let generation=0,disposed=false,controller:AvatarEditorController|null=null,status:AvatarEditorProductionStatus="signed-out";
  let snapshot:AvatarEditorProductionSnapshot={accountKey:null,agentId:null,generation:0,status,controller:null};
  const listeners=new Set<()=>void>();
  const emit=()=>{if(!disposed)for(const listener of [...listeners])listener()};
  const disposeController=()=>{controller?.dispose();controller=null};
  const rebuild=(next:AvatarEditorProductionScope)=>{
    disposeController();generation+=1;scope=next;
    if(next.accountKey==null)status="signed-out";
    else if(!validScope(next)||next.agent==null)status="missing-agent";
    else if(!callable(options.bridge.pickAvatarFile)||!callable(options.bridge.generateAgentAvatarImage)||!callable(options.bridge.setAgentAvatarBytes)||!callable(options.bridge.updateAgent))status="bridge-unavailable";
    else{
      status="ready";
      controller=createAvatarEditorController({
        agent:next.agent,
        desktop:options.bridge,
        roster:{
          setAgentAvatarBytes:(args)=>options.bridge.setAgentAvatarBytes(args),
          updateAgent:(args)=>options.bridge.updateAgent(args)
        },
        codec:options.codec
      });
    }
    snapshot={accountKey:scope.accountKey,agentId:scope.agent?.id??null,generation,status,controller};emit();
  };
  return{
    getSnapshot:()=>snapshot,
    subscribe(listener){if(disposed)return()=>{};listeners.add(listener);return()=>listeners.delete(listener)},
    setScope(next){
      if(disposed)return;
      if(scope.accountKey===next.accountKey&&scope.agent?.id===next.agent?.id&&scope.agent?.isGroup===next.agent?.isGroup&&scope.agent?.avatarDataUrl===next.agent?.avatarDataUrl&&scope.agent?.avatarShape===next.agent?.avatarShape&&scope.agent?.avatarColor===next.agent?.avatarColor)return;
      rebuild(next);
    },
    reset(){if(!disposed)rebuild(scope)},
    dispose(){if(disposed)return;disposed=true;generation+=1;disposeController();snapshot={accountKey:scope.accountKey,agentId:scope.agent?.id??null,generation,status,controller:null};listeners.clear()}
  };
}

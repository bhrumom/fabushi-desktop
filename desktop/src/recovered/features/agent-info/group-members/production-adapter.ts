import type { GrokAgentBridge, AgentSummary } from "../../../../grok-types";
import type { GroupRosterSource, GroupRosterSourceSnapshot } from "./model";

export interface LocalGroupRosterSource extends GroupRosterSource {
  setAgents(agents: readonly AgentSummary[]): void;
  setAccountGeneration(generation:number): void;
  refresh():Promise<void>;
  dispose():void;
}
function project(agents:readonly AgentSummary[]):readonly unknown[]{
  return agents.map(agent=>({
    id:agent.id,
    name:agent.name,
    isGroup:agent.isGroup===true,
    memberIds:[...(agent.memberIds||[])],
    isSharedRoom:agent.isSharedRoom===true
  }));
}
export function createLocalGroupRosterSource(
  bridge:Pick<GrokAgentBridge,"listAgents"|"setGroupMembers">,
  initialAgents:readonly AgentSummary[]=[],
  initialGeneration=0
):LocalGroupRosterSource{
  let agents=[...initialAgents],accountGeneration=initialGeneration,disposed=false;
  const listeners=new Set<()=>void>();
  const emit=()=>{if(!disposed)for(const listener of [...listeners])listener()};
  return{
    getSnapshot:():GroupRosterSourceSnapshot=>({accountGeneration,agents:project(agents)}),
    subscribe(listener){if(disposed)return()=>{};listeners.add(listener);return()=>listeners.delete(listener)},
    setAgents(next){agents=[...next];emit()},
    setAccountGeneration(next){if(next===accountGeneration)return;accountGeneration=next;emit()},
    async refresh(){agents=await bridge.listAgents();emit()},
    async setGroupMembers(args){await bridge.setGroupMembers({id:args.id,memberAgentIds:[...args.memberAgentIds]});await this.refresh()},
    dispose(){disposed=true;listeners.clear()}
  };
}

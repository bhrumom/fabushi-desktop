import type { AgentEvent, GrokAgentBridge } from './grok-types';
export function getAgentBridge():GrokAgentBridge {
  if (!window.grokAgent) throw new Error('Agent bridge unavailable');
  return window.grokAgent;
}
export function subscribeAgentEvents(listener:(event:AgentEvent)=>void):()=>void {
  return getAgentBridge().subscribe(listener);
}
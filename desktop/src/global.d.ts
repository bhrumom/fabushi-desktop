import type { GrokAgentBridge } from './grok-types';
declare global { interface Window { grokAgent:GrokAgentBridge; } }
export {};
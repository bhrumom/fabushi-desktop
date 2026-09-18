import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { getAgentBridge, subscribeAgentEvents } from './grok-agent-client';
import type { AgentMessage, AgentSummary, AgentThread, PluginDescriptor } from './grok-types';
import './grok-app.css';

const bridge=getAgentBridge();
const initials=(name:string)=>name.trim().split(/\s+/).slice(0,2).map(x=>x[0]?.toUpperCase()||'').join('')||'A';
const formatTime=(v:number)=>new Intl.DateTimeFormat(undefined,{hour:'numeric',minute:'2-digit'}).format(new Date(v));
function Status({status}:{status:AgentSummary['status']}){return <span className={'status status-'+status} aria-label={status}/>;}

function Sidebar(p:{agents:AgentSummary[];selected:string|null;query:string;setQuery(v:string):void;select(id:string):void;create():void;plugins():void;settings():void;}){
 const rows=useMemo(()=>{const q=p.query.trim().toLowerCase();return q?p.agents.filter(a=>a.name.toLowerCase().includes(q)):p.agents},[p.agents,p.query]);
 return <aside className="sidebar">
  <div className="drag-region"/>
  <div className="brand"><span className="brand-mark">✣</span><strong>Fabushi</strong><button aria-label="New agent" onClick={p.create}>＋</button></div>
  <label className="search"><span>⌕</span><input value={p.query} onChange={e=>p.setQuery(e.target.value)} placeholder="Search"/><kbd>⌘K</kbd></label>
  <div className="section-label">Agents</div>
  <div className="agent-list">{rows.map(a=><button className={'agent-row '+(p.selected===a.id?'selected':'')} key={a.id} onClick={()=>p.select(a.id)}>
   <span className="avatar">{initials(a.name)}</span><span className="agent-copy"><strong>{a.name}</strong><small>{a.status==='idle'?'Ready':a.status}</small></span><Status status={a.status}/>
  </button>)}</div>
  <div className="grow"/>
  <div className="sidebar-footer">
   <button onClick={p.plugins}>◫ <span>Plugins</span></button>
   <button onClick={p.settings}>⚙ <span>Settings</span></button>
   <button className="account"><span className="avatar light">F</span><span><strong>Fabushi</strong><small>Local computer</small></span></button>
  </div>
 </aside>;
}

function Message({message}:{message:AgentMessage}){
 if(message.role==='tool') return <div className="tool-row"><span className="tool-icon">›_</span><div><strong>{message.toolName||'Tool'}</strong><p>{message.text}</p></div></div>;
 return <article className={'message '+message.role}><div className="message-meta"><strong>{message.role==='user'?'You':message.role==='assistant'?'Agent':'System'}</strong><time>{formatTime(message.createdAt)}</time></div><div className="message-text">{message.text}</div>{message.status==='streaming'?<span className="stream-caret"/>:null}</article>;
}

function Composer({disabled,onSend}:{disabled:boolean;onSend(text:string):Promise<void>}){
 const [text,setText]=useState(''); const [busy,setBusy]=useState(false);
 const submit=async()=>{const value=text.trim();if(!value||busy||disabled)return;setText('');setBusy(true);try{await onSend(value)}finally{setBusy(false)}};
 return <div className="composer"><textarea rows={1} value={text} disabled={disabled} placeholder="Message agent" onChange={e=>setText(e.target.value)} onKeyDown={e=>{if(e.key==='Enter'&&!e.shiftKey){e.preventDefault();void submit()}}}/>
  <div className="composer-bar"><div><button title="Attach file" onClick={async()=>{const f=await bridge.pickFile();if(f)setText(v=>v+(v?'\n':'')+'@file '+f.path)}}>＋</button><button title="Mention">＠</button><span className="model-chip">Auto</span></div>
  <button className="send-button" disabled={disabled||busy||!text.trim()} onClick={()=>void submit()}>{busy?'…':'↑'}</button></div>
 </div>;
}

function Workspace({agent,thread,refresh}:{agent:AgentSummary;thread:AgentThread|null;refresh():Promise<void>}){
 const scroller=useRef<HTMLDivElement|null>(null);
 useEffect(()=>{scroller.current?.scrollTo({top:scroller.current.scrollHeight})},[thread?.messages.length]);
 return <main className="workspace">
  <header className="chat-header"><div className="identity"><span className="avatar large">{initials(agent.name)}</span><span><strong>{agent.name}</strong><small><Status status={agent.status}/> {agent.status==='idle'?'Ready':agent.status}</small></span></div><nav><button>⌕</button><button>•••</button></nav></header>
  <div className="transcript" ref={scroller}><div className="transcript-column">{thread?.messages.length?thread.messages.map(m=><Message key={m.id} message={m}/>):<section className="welcome"><span className="avatar hero">{initials(agent.name)}</span><h2>{agent.name}</h2><p>This agent works directly on this Mac.</p></section>}</div></div>
  <Composer disabled={agent.status==='thinking'||agent.status==='running'} onSend={async text=>{await bridge.sendMessage({agentId:agent.id,text});await refresh()}}/>
 </main>;
}

function Plugins({items,onClose,reload}:{items:PluginDescriptor[];onClose():void;reload():Promise<void>}){
 const [query,setQuery]=useState(''); const rows=items.filter(x=>(x.name+' '+x.description).toLowerCase().includes(query.toLowerCase()));
 return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay plugins" role="dialog" aria-label="Plugins">
  <header><div><h2>Plugins</h2><p>Give agents access to more tools and services.</p></div><button onClick={onClose}>×</button></header>
  <label className="plugin-search">⌕<input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search plugins"/></label>
  <nav className="tabs"><button className="active">Browse</button><button>Installed</button></nav>
  <div className="plugin-rows">{rows.map(x=><article key={x.id}><span className="plugin-logo">{x.name[0]}</span><div className="plugin-copy"><strong>{x.name}</strong><p>{x.description}</p><small>{x.category}</small></div><div className="plugin-actions">{x.installed?<><button className={x.enabled?'primary compact':''} onClick={async()=>{await bridge.setPluginEnabled({pluginId:x.id,enabled:!x.enabled});await reload()}}>{x.enabled?'Enabled':'Enable'}</button><button onClick={async()=>{await bridge.setPluginInstalled({pluginId:x.id,installed:false});await reload()}}>Remove</button></>:<button className="primary compact" onClick={async()=>{await bridge.setPluginInstalled({pluginId:x.id,installed:true});await reload()}}>Install</button>}</div></article>)}</div>
 </section></div>;
}

function Settings({onClose}:{onClose():void}){
 return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay settings" role="dialog" aria-label="Settings"><header><div><h2>Settings</h2><p>Fabushi desktop agent runtime</p></div><button onClick={onClose}>×</button></header>
 <div className="setting-row"><div><strong>Computer</strong><p>Agents operate on the Mac where Fabushi is installed.</p></div><span className="local-pill">Local</span></div>
 <div className="setting-row"><div><strong>Agent runtime</strong><p>Electron/TypeScript coordinator path. No CLI wrapper.</p></div><span>TypeScript</span></div>
 <div className="setting-row"><div><strong>Inference</strong><p>Configure an OpenAI-compatible endpoint with FABUSHI_AGENT_API_URL, FABUSHI_AGENT_API_KEY and FABUSHI_AGENT_MODEL.</p></div><span>Environment</span></div>
 </section></div>;
}

function CreateAgent({onClose,onCreated}:{onClose():void;onCreated(a:AgentSummary):void}){
 const [name,setName]=useState('');
 const submit=async(e:FormEvent)=>{e.preventDefault();if(name.trim())onCreated(await bridge.createAgent({name:name.trim()}))};
 return <div className="shade"><form className="create-dialog" onSubmit={submit}><h2>New agent</h2><p>Create a focused agent that can use local tools on this Mac.</p><input autoFocus value={name} onChange={e=>setName(e.target.value)} placeholder="Agent name"/><div><button type="button" onClick={onClose}>Cancel</button><button className="primary" disabled={!name.trim()}>Create</button></div></form></div>;
}

export default function GrokApp(){
 const [agents,setAgents]=useState<AgentSummary[]>([]); const [selectedId,setSelectedId]=useState<string|null>(null); const [thread,setThread]=useState<AgentThread|null>(null);
 const [plugins,setPlugins]=useState<PluginDescriptor[]>([]); const [overlay,setOverlay]=useState<'plugins'|'settings'|null>(null); const [createOpen,setCreateOpen]=useState(false); const [query,setQuery]=useState('');
 const selected=agents.find(a=>a.id===selectedId)||null;
 const loadAgents=async()=>{const next=await bridge.listAgents();setAgents(next);setSelectedId(current=>current&&next.some(a=>a.id===current)?current:(next[0]?.id||null))};
 const loadThread=async(id=selectedId)=>setThread(id?await bridge.getThread({agentId:id}):null);
 const loadPlugins=async()=>setPlugins(await bridge.listPlugins());
 useEffect(()=>{void loadAgents();void loadPlugins();return subscribeAgentEvents(event=>{if(event.type==='agents.changed'||event.type==='agent.changed')void loadAgents();if(event.type==='plugins.changed')void loadPlugins();if(event.agentId&&event.agentId===selectedId)void loadThread(event.agentId)})},[selectedId]);
 useEffect(()=>{void loadThread(selectedId)},[selectedId]);
 return <div className="app-shell"><Sidebar agents={agents} selected={selectedId} query={query} setQuery={setQuery} select={setSelectedId} create={()=>setCreateOpen(true)} plugins={()=>setOverlay('plugins')} settings={()=>setOverlay('settings')}/>
 {selected?<Workspace agent={selected} thread={thread} refresh={()=>loadThread(selected.id)}/>:<main className="empty"><span>✣</span><h1>What should your agent do?</h1><p>Create an agent for a project, task, research thread, or workflow.</p><button className="primary" onClick={()=>setCreateOpen(true)}>New agent</button></main>}
 {overlay==='plugins'?<Plugins items={plugins} onClose={()=>setOverlay(null)} reload={loadPlugins}/>:null}{overlay==='settings'?<Settings onClose={()=>setOverlay(null)}/>:null}
 {createOpen?<CreateAgent onClose={()=>setCreateOpen(false)} onCreated={agent=>{setCreateOpen(false);void loadAgents().then(()=>setSelectedId(agent.id))}}/>:null}</div>;
}
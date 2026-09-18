import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { getAgentBridge, subscribeAgentEvents } from './grok-agent-client';
import type { AgentMessage, AgentSummary, AgentThread, McpToolDescriptor, PluginDescriptor, RoutineAutomationDescriptor, RuntimeSettings, WorkflowDescriptor } from './grok-types';
import './grok-app.css';

const bridge=getAgentBridge();
const initials=(name:string)=>name.trim().split(/\s+/).slice(0,2).map(x=>x[0]?.toUpperCase()||'').join('')||'A';
const formatTime=(v:number)=>new Intl.DateTimeFormat(undefined,{hour:'numeric',minute:'2-digit'}).format(new Date(v));
const busy=(status:AgentSummary['status'])=>status==='thinking'||status==='running'||status==='waiting';

function Status({status}:{status:AgentSummary['status']}) {
  return <span className={'status status-'+status} aria-label={status}/>;
}

function Sidebar(p:{
  agents:AgentSummary[];selected:string|null;query:string;setQuery(v:string):void;select(id:string):void;
  create():void;plugins():void;settings():void;rename(agent:AgentSummary):void;remove(agent:AgentSummary):void;openPalette():void;
}) {
  const rows=useMemo(()=>{
    const q=p.query.trim().toLowerCase();
    return q?p.agents.filter(a=>a.name.toLowerCase().includes(q)):p.agents;
  },[p.agents,p.query]);
  return <aside className="sidebar">
    <div className="drag-region"/>
    <div className="brand"><span className="brand-mark">✣</span><strong>Fabushi</strong><button aria-label="New agent" onClick={p.create}>＋</button></div>
    <label className="search" onClick={p.openPalette}><span>⌕</span><input value={p.query} onChange={e=>p.setQuery(e.target.value)} placeholder="Search agents"/><kbd>⌘K</kbd></label>
    <div className="section-label">Agents</div>
    <div className="agent-list">{rows.map(a=><div className={'agent-row '+(p.selected===a.id?'selected':'')} key={a.id}>
      <button className="agent-select" onClick={()=>p.select(a.id)}>
        <span className="avatar">{initials(a.name)}</span>
        <span className="agent-copy"><strong>{a.name}</strong><small>{a.status==='idle'?'Ready':a.status}</small></span>
        <Status status={a.status}/>
      </button>
      <span className="row-actions">
        <button aria-label={'Rename '+a.name} title="Rename" onClick={()=>p.rename(a)}>✎</button>
        <button aria-label={'Delete '+a.name} title="Delete" onClick={()=>p.remove(a)}>×</button>
      </span>
    </div>)}</div>
    <div className="grow"/>
    <div className="sidebar-footer">
      <button onClick={p.plugins}>◫ <span>Plugins</span></button>
      <button onClick={p.settings}>⚙ <span>Settings</span></button>
      <div className="account"><span className="avatar light">F</span><span><strong>Fabushi</strong><small>Local computer</small></span></div>
    </div>
  </aside>;
}

function ToolMessage({message,onResolve}:{message:AgentMessage;onResolve(approvalId:string,approved:boolean):Promise<void>}) {
  const [resolving,setResolving]=useState(false);
  const resolve=async(approved:boolean)=>{
    if(!message.approvalId||resolving)return;
    setResolving(true);
    try{await onResolve(message.approvalId,approved)}finally{setResolving(false)}
  };
  return <div className={'tool-row tool-'+(message.status||'done')}>
    <span className="tool-icon">›_</span>
    <div className="tool-body">
      <div className="tool-heading"><strong>{message.toolName||'Tool'}</strong><span>{message.status||'done'}</span></div>
      <p>{message.text}</p>
      {message.display?.kind==='image'?<img className="tool-image" src={message.display.dataUrl} alt="Computer screenshot"/>:null}
      {message.status==='waiting-approval'&&message.approvalId?<div className="approval-actions">
        <button disabled={resolving} onClick={()=>void resolve(false)}>Deny</button>
        <button className="primary compact" disabled={resolving} onClick={()=>void resolve(true)}>Allow once</button>
      </div>:null}
    </div>
  </div>;
}

function Message({message,onResolve}:{message:AgentMessage;onResolve(approvalId:string,approved:boolean):Promise<void>}) {
  if(message.role==='tool')return <ToolMessage message={message} onResolve={onResolve}/>;
  return <article className={'message '+message.role}>
    <div className="message-meta"><strong>{message.role==='user'?'You':message.role==='assistant'?'Agent':'System'}</strong><time>{formatTime(message.createdAt)}</time></div>
    <div className="message-text">{message.text}</div>
    {message.status==='streaming'?<span className="stream-caret"/>:null}
  </article>;
}

function Composer({running,onSend,onStop}:{running:boolean;onSend(text:string):Promise<void>;onStop():Promise<void>}) {
  const [text,setText]=useState('');
  const [submitting,setSubmitting]=useState(false);
  const submit=async()=>{
    const value=text.trim();if(!value||submitting||running)return;
    setText('');setSubmitting(true);
    try{await onSend(value)}finally{setSubmitting(false)}
  };
  return <div className="composer">
    <textarea rows={1} value={text} disabled={running} placeholder={running?'Agent is working…':'Message agent'} onChange={e=>setText(e.target.value)} onKeyDown={e=>{
      if(e.key==='Enter'&&!e.shiftKey){e.preventDefault();void submit()}
    }}/>
    <div className="composer-bar"><div>
      <button title="Attach file" disabled={running} onClick={async()=>{
        const file=await bridge.pickFile();if(file)setText(v=>v+(v?'\n':'')+'@file '+file.path);
      }}>＋</button>
      <span className="model-chip">Auto</span>
    </div>
    {running?<button className="stop-button" title="Stop agent" onClick={()=>void onStop()}>■</button>:<button className="send-button" disabled={submitting||!text.trim()} onClick={()=>void submit()}>{submitting?'…':'↑'}</button>}</div>
  </div>;
}

function Workspace({agent,thread,refresh,openAutomations}:{agent:AgentSummary;thread:AgentThread|null;refresh():Promise<void>;openAutomations():void}) {
  const scroller=useRef<HTMLDivElement|null>(null);
  useEffect(()=>{scroller.current?.scrollTo({top:scroller.current.scrollHeight})},[thread?.messages.length]);
  return <main className="workspace">
    <header className="chat-header"><div className="identity"><span className="avatar large">{initials(agent.name)}</span><span><strong>{agent.name}</strong><small><Status status={agent.status}/> {agent.status==='idle'?'Ready':agent.status}</small></span></div><button className="header-action" onClick={openAutomations}>Routines</button></header>
    <div className="transcript" ref={scroller}><div className="transcript-column">
      {thread?.messages.length?thread.messages.map(message=><Message key={message.id} message={message} onResolve={async(approvalId,approved)=>{
        await bridge.resolveApproval({approvalId,approved});await refresh();
      }}/>):<section className="welcome"><span className="avatar hero">{initials(agent.name)}</span><h2>{agent.name}</h2><p>This agent works directly on this Mac.</p></section>}
    </div></div>
    <Composer running={busy(agent.status)} onSend={async text=>{await bridge.sendMessage({agentId:agent.id,text});await refresh()}} onStop={async()=>{
      await bridge.stopAgent({agentId:agent.id});await refresh();
    }}/>
  </main>;
}

function Plugins({items,workflows,onClose,reload,reloadWorkflows}:{items:PluginDescriptor[];workflows:WorkflowDescriptor[];onClose():void;reload():Promise<void>;reloadWorkflows():Promise<void>}) {
  const [tab,setTab]=useState<'plugins'|'skills'>('plugins');
  const [query,setQuery]=useState('');
  const [addOpen,setAddOpen]=useState(false);
  const [name,setName]=useState('');
  const [transport,setTransport]=useState<'stdio'|'http'>('stdio');
  const [command,setCommand]=useState('');
  const [argsText,setArgsText]=useState('[]');
  const [url,setUrl]=useState('');
  const [customInstructions,setCustomInstructions]=useState('');
  const [expanded,setExpanded]=useState<string|null>(null);
  const [tools,setTools]=useState<Record<string,McpToolDescriptor[]>>({});
  const [skillEditor,setSkillEditor]=useState<WorkflowDescriptor|{id?:string;name:string;description:string;body:string;isEnabledForAgent:boolean}|null>(null);
  const [error,setError]=useState('');
  const rows=items.filter(x=>(x.name+' '+x.description+' '+x.category).toLowerCase().includes(query.toLowerCase()));
  const skillRows=workflows.filter(x=>(x.name+' '+x.description).toLowerCase().includes(query.toLowerCase()));
  const loadTools=async(serverId:string)=>{
    setError('');
    try{setTools(current=>({...current,[serverId]:[]}));const next=await bridge.listMcpServerTools({serverId});setTools(current=>({...current,[serverId]:next}))}
    catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  const addServer=async(e:FormEvent)=>{
    e.preventDefault();setError('');
    try{
      const parsed=JSON.parse(argsText||'[]');
      if(!Array.isArray(parsed)||parsed.some(value=>typeof value!=='string'))throw new Error('Args must be a JSON string array.');
      await bridge.addMcpServer({
        name:name.trim(),transport,
        ...(transport==='stdio'?{command:command.trim(),args:parsed}:{url:url.trim()}),
        ...(customInstructions.trim()?{customInstructions:customInstructions.trim()}:{})
      });
      setName('');setTransport('stdio');setCommand('');setArgsText('[]');setUrl('');setCustomInstructions('');setAddOpen(false);await reload();
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  const newSkill=()=>setSkillEditor({name:'',description:'',body:'',isEnabledForAgent:true});
  const saveSkill=async(e:FormEvent)=>{
    e.preventDefault();if(!skillEditor)return;setError('');
    try{
      await bridge.saveWorkflow({
        ...(skillEditor.id?{id:skillEditor.id}:{}),
        name:skillEditor.name,description:skillEditor.description,body:skillEditor.body,
        trigger:'trigger' in skillEditor?skillEditor.trigger:null,
        isEnabledForAgent:skillEditor.isEnabledForAgent,
        disableModelInvocation:'disableModelInvocation' in skillEditor?skillEditor.disableModelInvocation:false
      });
      setSkillEditor(null);await reloadWorkflows();
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay plugins" role="dialog" aria-label="Plugins">
    <header><div><h2>Plugins</h2><p>Executable local tools, MCP servers, and file-backed private skills.</p></div><div className="overlay-head-actions">
      {tab==='plugins'?<button className="add-server" onClick={()=>setAddOpen(value=>!value)}>＋ MCP</button>:<button className="add-server" onClick={newSkill}>＋ Skill</button>}
      <button onClick={onClose}>×</button>
    </div></header>
    {tab==='plugins'&&addOpen?<form className="mcp-add" onSubmit={e=>void addServer(e)}>
      <input value={name} onChange={e=>setName(e.target.value)} placeholder="Server name" required/>
      <select value={transport} onChange={e=>setTransport(e.target.value as 'stdio'|'http')}><option value="stdio">Local stdio</option><option value="http">Remote HTTP</option></select>
      {transport==='stdio'?<><input value={command} onChange={e=>setCommand(e.target.value)} placeholder="Command, e.g. npx" required/><input value={argsText} onChange={e=>setArgsText(e.target.value)} placeholder={'["-y","@vendor/server"]'} required/></>:<input className="mcp-url" value={url} onChange={e=>setUrl(e.target.value)} placeholder="https://server.example/mcp" required/>}
      <textarea value={customInstructions} onChange={e=>setCustomInstructions(e.target.value)} placeholder="Optional server instructions passed to the Agent with each routed tool"/>
      <div><button type="button" onClick={()=>setAddOpen(false)}>Cancel</button><button className="primary compact">Add server</button></div>
    </form>:null}
    {tab==='skills'&&skillEditor?<form className="skill-editor" onSubmit={e=>void saveSkill(e)}>
      <input value={skillEditor.name} onChange={e=>setSkillEditor({...skillEditor,name:e.target.value})} placeholder="Skill name" required/>
      <input value={skillEditor.description} onChange={e=>setSkillEditor({...skillEditor,description:e.target.value})} placeholder="When should the agent use this skill?"/>
      <textarea value={skillEditor.body} onChange={e=>setSkillEditor({...skillEditor,body:e.target.value})} placeholder="Skill instructions (stored in SKILL.md)" required/>
      <label><input type="checkbox" checked={skillEditor.isEnabledForAgent} onChange={e=>setSkillEditor({...skillEditor,isEnabledForAgent:e.target.checked})}/> Available to agents</label>
      <div><button type="button" onClick={()=>setSkillEditor(null)}>Cancel</button><button className="primary compact">Save skill</button></div>
    </form>:null}
    <label className="plugin-search">⌕<input value={query} onChange={e=>setQuery(e.target.value)} placeholder={tab==='plugins'?'Search plugins':'Search skills'}/></label>
    <nav className="tabs"><button className={tab==='plugins'?'active':''} onClick={()=>setTab('plugins')}>Installed</button><button className={tab==='skills'?'active':''} onClick={()=>setTab('skills')}>Private skills</button></nav>
    {error?<div className="plugin-error">{error}</div>:null}
    {tab==='plugins'?<div className="plugin-rows">{rows.length?rows.map(x=><article className="plugin-card" key={x.id}>
      <span className="plugin-logo">{x.name[0]}</span>
      <div className="plugin-copy"><strong>{x.name}</strong><p>{x.description}</p><small>{x.category} · {x.provider||'provider'}</small>
        {x.kind==='mcp'&&x.serverId&&expanded===x.serverId?<div className="mcp-tools">
          {(tools[x.serverId]||[]).length?(tools[x.serverId]||[]).map(tool=><label key={tool.name}><span><strong>{tool.name}</strong><small>{tool.description||'MCP tool'}</small></span><input type="checkbox" checked={!tool.isDisabled} onChange={async e=>{
            const next=await bridge.setMcpToolEnabled({serverId:x.serverId!,toolName:tool.name,enabled:e.target.checked});
            setTools(current=>({...current,[x.serverId!]:next}));
          }}/></label>):<span className="tool-loading">No tools loaded. If the server is starting, retry Configure.</span>}
        </div>:null}
      </div>
      <div className="plugin-actions">
        {x.kind==='mcp'&&x.serverId?<button onClick={async()=>{setExpanded(currentId=>currentId===x.serverId?null:x.serverId!);if(expanded!==x.serverId)await loadTools(x.serverId!)}}>Configure</button>:null}
        <button className={x.enabled?'primary compact':''} onClick={async()=>{await bridge.setPluginEnabled({pluginId:x.id,enabled:!x.enabled});await reload()}}>{x.enabled?'Enabled':'Enable'}</button>
        {x.removable?<button onClick={async()=>{await bridge.setPluginInstalled({pluginId:x.id,installed:false});if(x.serverId)setExpanded(null);await reload()}}>Remove</button>:null}
      </div>
    </article>):<div className="overlay-empty">No executable capability matches this search.</div>}</div>
    :<div className="skill-rows">{skillRows.length?skillRows.map(skill=><article key={skill.id}>
      <div className="plugin-copy"><strong>{skill.name}</strong><p>{skill.description||'No description'}</p><small>SKILL.md · mention @{skill.name.toLowerCase().replace(/\s+/g,'')} to load instructions</small></div>
      <div className="plugin-actions">
        <button onClick={()=>setSkillEditor(skill)}>Edit</button>
        <button className={skill.isEnabledForAgent?'primary compact':''} onClick={async()=>{await bridge.setWorkflowEnabled({id:skill.id,enabled:!skill.isEnabledForAgent});await reloadWorkflows()}}>{skill.isEnabledForAgent?'Enabled':'Enable'}</button>
        <button onClick={async()=>{await bridge.deleteWorkflow({id:skill.id});await reloadWorkflows()}}>Delete</button>
      </div>
    </article>):<div className="overlay-empty">No private skills yet. Create one to store a real SKILL.md under the Fabushi profile.</div>}</div>}
  </section></div>;
}

function Automations({agent,onClose}:{agent:AgentSummary;onClose():void}) {
  const [rows,setRows]=useState<RoutineAutomationDescriptor[]>([]);
  const [loading,setLoading]=useState(true);
  const [error,setError]=useState('');
  const [pending,setPending]=useState<string|null>(null);
  const [editor,setEditor]=useState<null|{id?:string;name:string;prompt:string;schedule:string;isEnabled:boolean}>(null);
  const load=async()=>{setLoading(true);setError('');try{setRows(await bridge.getAgentAutomations({id:agent.id}))}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setLoading(false)}};
  useEffect(()=>{void load();return subscribeAgentEvents(event=>{if(event.type==='automations.changed'&&event.agentId===agent.id)void load()})},[agent.id]);
  const save=async(e:FormEvent)=>{
    e.preventDefault();if(!editor)return;setPending(editor.id||'create');setError('');
    const spec={name:editor.name.trim(),prompt:editor.prompt.trim(),trigger:{type:'cron' as const,schedule:editor.schedule.trim()},isEnabled:editor.isEnabled};
    try{
      if(editor.id)await bridge.updateAgentAutomation({id:agent.id,automationId:editor.id,spec});
      else await bridge.createAgentAutomation({id:agent.id,spec});
      setEditor(null);await load();
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setPending(null)}
  };
  const fmt=(value:number|null)=>value?new Intl.DateTimeFormat(undefined,{dateStyle:'medium',timeStyle:'short'}).format(new Date(value)):'—';
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay routines" role="dialog" aria-label="Routines">
    <header><div><h2>Routines</h2><p>Automations owned by {agent.name}. Runs execute through the same Agent host and local-tool permission model.</p></div><div className="overlay-head-actions"><button className="add-server" onClick={()=>setEditor({name:'',prompt:'',schedule:'0 9 * * 1-5',isEnabled:true})}>＋ Routine</button><button onClick={onClose}>×</button></div></header>
    {editor?<form className="routine-editor" onSubmit={e=>void save(e)}>
      <input value={editor.name} onChange={e=>setEditor({...editor,name:e.target.value})} placeholder="Routine name" required/>
      <input value={editor.schedule} onChange={e=>setEditor({...editor,schedule:e.target.value})} placeholder="0 9 * * 1-5 or @every 2h" required/>
      <textarea value={editor.prompt} onChange={e=>setEditor({...editor,prompt:e.target.value})} placeholder="What should this agent do?" required/>
      <label><input type="checkbox" checked={editor.isEnabled} onChange={e=>setEditor({...editor,isEnabled:e.target.checked})}/> Enabled</label>
      <small>Supports the reference schedule forms: 5-field cron, @hourly/@daily/etc, @every Ns/m/h/d, and CRON_TZ/TZ prefixes.</small>
      <div><button type="button" onClick={()=>setEditor(null)}>Cancel</button><button className="primary compact" disabled={pending!==null}>Save</button></div>
    </form>:null}
    {error?<div className="plugin-error">{error}</div>:null}
    <div className="routine-list">{loading&&!rows.length?<div className="overlay-empty">Loading routines…</div>:rows.length?rows.map(row=><article key={row.id}>
      <div className="routine-main"><div className="routine-title"><strong>{row.name}</strong><span>{row.isEnabled?'Enabled':'Paused'}</span></div><p>{row.prompt}</p><small>{row.triggerDescription} · next {fmt(row.nextRunAt)} · last {fmt(row.lastRunAt)}</small>
        {row.runs.length?<div className="run-history">{row.runs.slice(0,5).map(run=><div key={run.id}><span className={'run-status '+run.status}>{run.status}</span><time>{fmt(run.startedAt)}</time><span>{run.event||'run'}</span>{run.detail?<em>{run.detail}</em>:null}</div>)}</div>:null}
      </div>
      <div className="plugin-actions">
        <button onClick={()=>setEditor({id:row.id,name:row.name,prompt:row.prompt,schedule:row.trigger.schedule,isEnabled:row.isEnabled})}>Edit</button>
        <button onClick={async()=>{setPending(row.id);try{await bridge.setAgentAutomationEnabled({id:agent.id,automationId:row.id,isEnabled:!row.isEnabled});await load()}finally{setPending(null)}}}>{row.isEnabled?'Pause':'Enable'}</button>
        <button className="primary compact" disabled={pending!==null||busy(agent.status)} onClick={async()=>{setPending(row.id);try{await bridge.runAgentAutomationNow({id:agent.id,automationId:row.id});await load()}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setPending(null)}}}>Run now</button>
        <button onClick={async()=>{setPending(row.id);try{await bridge.deleteAgentAutomation({id:agent.id,automationId:row.id});await load()}finally{setPending(null)}}}>Delete</button>
      </div>
    </article>):<div className="overlay-empty">No routines yet. Create one to schedule real work for this agent.</div>}</div>
  </section></div>;
}

function Settings({onClose}:{onClose():void}) {
  const [settings,setSettings]=useState<RuntimeSettings|null>(null);
  const [error,setError]=useState('');
  useEffect(()=>{void bridge.getRuntimeSettings().then(setSettings,e=>setError(String(e)))},[]);
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay settings" role="dialog" aria-label="Settings">
    <header><div><h2>Settings</h2><p>Agent execution and local-computer permissions</p></div><button onClick={onClose}>×</button></header>
    <div className="setting-row"><div><strong>Computer</strong><p>Agents operate on the Mac where Fabushi is installed.</p></div><span className="local-pill">Local Mac</span></div>
    <div className="setting-row"><div><strong>Local tool permission</strong><p>Mutating Files, Terminal, Browser and Computer actions pass through this policy.</p></div>
      {settings?<select value={settings.localToolPermission} onChange={e=>{
        const permission=e.target.value as RuntimeSettings['localToolPermission'];
        void bridge.setLocalToolPermission({permission}).then(setSettings,reason=>setError(String(reason)));
      }}><option value="ask">Ask every time</option><option value="always">Always allow</option><option value="never">Never allow</option></select>:<span>Loading…</span>}
    </div>
    <div className="setting-row"><div><strong>Auto-review</strong><p>Classifier review for Browser and Computer mutations. Enforce blocks or asks before execution; shadow records decisions without changing execution.</p></div>
      {settings?<select value={settings.autoReviewMode} onChange={e=>{
        const mode=e.target.value as RuntimeSettings['autoReviewMode'];
        void bridge.setAutoReviewMode({mode}).then(setSettings,reason=>setError(String(reason)));
      }}><option value="enforce">Enforce</option><option value="shadow">Shadow</option><option value="off">Off</option></select>:<span>Loading…</span>}
    </div>
    <div className="setting-row"><div><strong>Agent runtime</strong><p>Renderer → preload → coordinator → host → local execution.</p></div><span>Coordinator/Host</span></div>
    <div className="setting-row"><div><strong>Inference</strong><p>OpenAI-compatible endpoint configured through Fabushi agent environment variables.</p></div><span>External model</span></div>
    {error?<div className="settings-error">{error}</div>:null}
  </section></div>;
}

function CreateAgent({onClose,onCreated}:{onClose():void;onCreated(a:AgentSummary):void}) {
  const [name,setName]=useState('');
  const submit=async(e:FormEvent)=>{e.preventDefault();if(name.trim())onCreated(await bridge.createAgent({name:name.trim()}))};
  return <div className="shade"><form className="create-dialog" onSubmit={submit}><h2>New agent</h2><p>Create a focused agent that can use local tools on this Mac.</p><input autoFocus value={name} onChange={e=>setName(e.target.value)} placeholder="Agent name"/><div><button type="button" onClick={onClose}>Cancel</button><button className="primary" disabled={!name.trim()}>Create</button></div></form></div>;
}

function RenameAgent({agent,onClose,onSaved}:{agent:AgentSummary;onClose():void;onSaved():Promise<void>}) {
  const [name,setName]=useState(agent.name);
  const submit=async(e:FormEvent)=>{e.preventDefault();if(!name.trim())return;await bridge.renameAgent({agentId:agent.id,name:name.trim()});await onSaved();onClose()};
  return <div className="shade"><form className="create-dialog" onSubmit={submit}><h2>Rename agent</h2><p>Choose the name shown in the agent list and conversation.</p><input autoFocus value={name} onChange={e=>setName(e.target.value)}/><div><button type="button" onClick={onClose}>Cancel</button><button className="primary" disabled={!name.trim()}>Save</button></div></form></div>;
}

function DeleteAgent({agent,onClose,onDeleted}:{agent:AgentSummary;onClose():void;onDeleted():Promise<void>}) {
  return <div className="shade"><section className="create-dialog"><h2>Delete {agent.name}?</h2><p>This removes its local conversation transcript from this Fabushi profile.</p><div><button onClick={onClose}>Cancel</button><button className="danger" onClick={async()=>{await bridge.deleteAgent({agentId:agent.id});await onDeleted();onClose()}}>Delete</button></div></section></div>;
}

function CommandPalette({agents,onClose,onSelect,onCreate,onPlugins,onSettings}:{agents:AgentSummary[];onClose():void;onSelect(id:string):void;onCreate():void;onPlugins():void;onSettings():void}) {
  const [query,setQuery]=useState('');
  const actions=[
    {id:'new',label:'New agent',run:onCreate},
    {id:'plugins',label:'Open Plugins',run:onPlugins},
    {id:'settings',label:'Open Settings',run:onSettings},
    ...agents.map(agent=>({id:'agent:'+agent.id,label:'Open '+agent.name,run:()=>onSelect(agent.id)}))
  ].filter(item=>item.label.toLowerCase().includes(query.toLowerCase()));
  return <div className="shade palette-shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="command-palette" role="dialog" aria-label="Command palette">
    <input autoFocus value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search commands and agents" onKeyDown={e=>{if(e.key==='Escape')onClose()}}/>
    <div>{actions.length?actions.map(item=><button key={item.id} onClick={()=>{item.run();onClose()}}>{item.label}</button>):<p>No commands found.</p>}</div>
  </section></div>;
}

export default function GrokApp(){
  const [agents,setAgents]=useState<AgentSummary[]>([]);
  const [selectedId,setSelectedId]=useState<string|null>(null);
  const [thread,setThread]=useState<AgentThread|null>(null);
  const [plugins,setPlugins]=useState<PluginDescriptor[]>([]);
  const [workflows,setWorkflows]=useState<WorkflowDescriptor[]>([]);
  const [overlay,setOverlay]=useState<'plugins'|'settings'|'automations'|null>(null);
  const [createOpen,setCreateOpen]=useState(false);
  const [renameAgent,setRenameAgent]=useState<AgentSummary|null>(null);
  const [deleteAgent,setDeleteAgent]=useState<AgentSummary|null>(null);
  const [paletteOpen,setPaletteOpen]=useState(false);
  const [query,setQuery]=useState('');
  const [loading,setLoading]=useState(true);
  const [failure,setFailure]=useState('');
  const selected=agents.find(a=>a.id===selectedId)||null;

  const loadAgents=async()=>{
    const next=await bridge.listAgents();setAgents(next);
    setSelectedId(current=>current&&next.some(a=>a.id===current)?current:(next[0]?.id||null));
  };
  const loadThread=async(id=selectedId)=>setThread(id?await bridge.getThread({agentId:id}):null);
  const loadPlugins=async()=>setPlugins(await bridge.listPlugins());
  const loadWorkflows=async()=>setWorkflows(await bridge.listWorkflows());
  const refreshAll=async()=>{try{setFailure('');await Promise.all([loadAgents(),loadPlugins(),loadWorkflows()])}catch(error){setFailure(error instanceof Error?error.message:String(error))}finally{setLoading(false)}};

  useEffect(()=>{void refreshAll()},[]);
  useEffect(()=>{if(selectedId)void loadThread(selectedId).catch(error=>setFailure(error instanceof Error?error.message:String(error)));else setThread(null)},[selectedId]);
  useEffect(()=>subscribeAgentEvents(event=>{
    if(event.type==='agents.changed'||event.type==='agent.changed')void loadAgents().catch(()=>{});
    if(event.type==='plugins.changed')void loadPlugins().catch(()=>{});
    if(event.type==='workflows.changed')void loadWorkflows().catch(()=>{});
    if(event.agentId&&event.agentId===selectedId)void loadThread(event.agentId).catch(()=>{});
  }),[selectedId]);
  useEffect(()=>{
    const onKey=(event:KeyboardEvent)=>{
      if((event.metaKey||event.ctrlKey)&&event.key.toLowerCase()==='k'){event.preventDefault();setPaletteOpen(true)}
      if((event.metaKey||event.ctrlKey)&&event.key.toLowerCase()==='n'){event.preventDefault();setCreateOpen(true)}
      if(event.key==='Escape'){setPaletteOpen(false);setOverlay(null)}
    };
    window.addEventListener('keydown',onKey);return()=>window.removeEventListener('keydown',onKey);
  },[]);

  if(loading)return <div className="root-state"><span className="spinner"/><strong>Loading agents…</strong></div>;
  if(failure&&!agents.length)return <div className="root-state error-state"><strong>Fabushi could not load the agent runtime.</strong><p>{failure}</p><button className="primary" onClick={()=>void refreshAll()}>Retry</button></div>;

  return <div className="app-shell">
    <Sidebar agents={agents} selected={selectedId} query={query} setQuery={setQuery} select={setSelectedId} create={()=>setCreateOpen(true)}
      plugins={()=>setOverlay('plugins')} settings={()=>setOverlay('settings')} rename={setRenameAgent} remove={setDeleteAgent} openPalette={()=>setPaletteOpen(true)}/>
    {selected?<Workspace agent={selected} thread={thread} refresh={()=>loadThread(selected.id)} openAutomations={()=>setOverlay('automations')}/>:<main className="empty"><span>✣</span><h1>What should your agent do?</h1><p>Create an agent for a project, task, research thread, or workflow.</p><button className="primary" onClick={()=>setCreateOpen(true)}>New agent</button></main>}
    {failure?<div className="toast-error">{failure}<button onClick={()=>setFailure('')}>×</button></div>:null}
    {overlay==='plugins'?<Plugins items={plugins} workflows={workflows} onClose={()=>setOverlay(null)} reload={loadPlugins} reloadWorkflows={loadWorkflows}/>:null}
    {overlay==='settings'?<Settings onClose={()=>setOverlay(null)}/>:null}
    {overlay==='automations'&&selected?<Automations agent={selected} onClose={()=>setOverlay(null)}/>:null}
    {createOpen?<CreateAgent onClose={()=>setCreateOpen(false)} onCreated={agent=>{setCreateOpen(false);void loadAgents().then(()=>setSelectedId(agent.id))}}/>:null}
    {renameAgent?<RenameAgent agent={renameAgent} onClose={()=>setRenameAgent(null)} onSaved={loadAgents}/>:null}
    {deleteAgent?<DeleteAgent agent={deleteAgent} onClose={()=>setDeleteAgent(null)} onDeleted={async()=>{await loadAgents();setThread(null)}}/>:null}
    {paletteOpen?<CommandPalette agents={agents} onClose={()=>setPaletteOpen(false)} onSelect={setSelectedId} onCreate={()=>setCreateOpen(true)} onPlugins={()=>setOverlay('plugins')} onSettings={()=>setOverlay('settings')}/>:null}
  </div>;
}

import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { getAgentBridge, subscribeAgentEvents } from './grok-agent-client';
import type { AgentMessage, AgentSummary, AgentThread, AttachmentDescriptor, AttachmentPreview, MarketplaceCatalogDescriptor, McpAccountStatus, McpServerDescriptor, McpToolDescriptor, PluginDescriptor, RoutineAutomationDescriptor, RuntimeSettings, WorkflowDescriptor } from './grok-types';
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
  create():void;plugins():void;settings():void;orgChart():void;hiddenChats():void;rename(agent:AgentSummary):void;hide(agent:AgentSummary):void;remove(agent:AgentSummary):void;openPalette():void;
}) {
  const rows=useMemo(()=>{
    const visible=p.agents.filter(a=>!a.hidden);
    const q=p.query.trim().toLowerCase();
    return q?visible.filter(a=>a.name.toLowerCase().includes(q)):visible;
  },[p.agents,p.query]);
  return <aside className="sidebar sand-agents-sidebar">
    <div className="drag-region"/>
    <div className="brand"><span className="brand-mark">✣</span><strong>Fabushi</strong><button aria-label="New agent" onClick={p.create}>＋</button></div>
    <label className="search" onClick={p.openPalette}><span>⌕</span><input value={p.query} onChange={e=>p.setQuery(e.target.value)} placeholder="Search agents"/><kbd>⌘K</kbd></label>
    <div className="section-label">Agents</div>
    <div className="agent-list">{rows.map(a=><div className={'agent-row sand-agent-item '+(p.selected===a.id?'selected':'')} key={a.id}>
      <button className="agent-select" onClick={()=>p.select(a.id)}>
        <span className="avatar">{initials(a.name)}</span>
        <span className="agent-copy"><strong>{a.name}</strong><small>{a.status==='idle'?'Ready':a.status}</small></span>
        <Status status={a.status}/>
      </button>
      <span className="row-actions">
        <button aria-label={'Rename '+a.name} title="Rename" onClick={()=>p.rename(a)}>✎</button>
        <button aria-label={'Hide '+a.name} title="Hide" onClick={()=>p.hide(a)}>◌</button>
        <button aria-label={'Delete '+a.name} title="Delete" onClick={()=>p.remove(a)}>×</button>
      </span>
    </div>)}</div>
    <div className="grow"/>
    <div className="sidebar-footer">
      <button onClick={p.orgChart}>⌘ <span>Org chart</span></button>
      <button onClick={p.hiddenChats}>◉ <span>Hidden Bots</span></button>
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

function AttachmentCards({items}:{items:AttachmentDescriptor[]|undefined}) {
  const [preview,setPreview]=useState<AttachmentPreview|null>(null),[error,setError]=useState('');
  if(!items?.length)return null;
  const open=async(item:AttachmentDescriptor)=>{setError('');try{setPreview(await bridge.readAttachment({id:item.id}))}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}};
  return <><div className="attachment-cards">{items.map(item=><button key={item.id} className="attachment-card" onClick={()=>void open(item)}><span>{item.kind==='image'?'▧':item.kind==='pdf'?'▤':'▱'}</span><span><strong>{item.name}</strong><small>{item.mime} · {Math.max(1,Math.ceil(item.size/1024))} KB</small></span></button>)}</div>
    {error?<div className="attachment-error">{error}</div>:null}
    {preview?<div className="shade attachment-preview-shade" onMouseDown={e=>e.currentTarget===e.target&&setPreview(null)}><section className="attachment-preview" role="dialog" aria-label={preview.name}><header><strong>{preview.name}</strong><button aria-label="Close" onClick={()=>setPreview(null)}>×</button></header><div>{preview.kind==='image'?<img src={preview.dataUrl} alt={preview.name}/>:preview.kind==='pdf'||preview.kind==='text'?<iframe sandbox="" src={preview.dataUrl} title={preview.name}/>:<div className="attachment-no-preview">Preview unavailable for {preview.mime}</div>}</div></section></div>:null}
  </>;
}

function Message({message,onResolve}:{message:AgentMessage;onResolve(approvalId:string,approved:boolean):Promise<void>}) {
  if(message.role==='tool')return <div data-entry-id={message.id}><ToolMessage message={message} onResolve={onResolve}/></div>;
  return <article className={'message '+message.role} data-entry-id={message.id}>
    <div className="message-meta"><strong>{message.role==='user'?'You':message.role==='assistant'?'Agent':'System'}</strong><time>{formatTime(message.createdAt)}</time></div>
    <div className="message-text">{message.text}</div>
    <AttachmentCards items={message.attachments}/>
    {message.status==='streaming'?<span className="stream-caret"/>:null}
  </article>;
}

function Composer({running,onSend,onStop}:{running:boolean;onSend(text:string,attachmentIds:string[]):Promise<void>;onStop():Promise<void>}) {
  const [text,setText]=useState('');
  const [attachments,setAttachments]=useState<AttachmentDescriptor[]>([]);
  const [submitting,setSubmitting]=useState(false);
  const submit=async()=>{
    const value=text.trim();if((!value&&!attachments.length)||submitting||running)return;
    setSubmitting(true);
    try{await onSend(value,attachments.map(item=>item.id));setText('');setAttachments([])}finally{setSubmitting(false)}
  };
  return <div className="composer sand-prompt-shell sand-prompt-form">
    {attachments.length?<div className="composer-attachments">{attachments.map(item=><span key={item.id}><span>{item.name}</span><button aria-label={'Remove '+item.name} onClick={()=>setAttachments(rows=>rows.filter(row=>row.id!==item.id))}>×</button></span>)}</div>:null}
    <textarea rows={1} value={text} disabled={running} placeholder={running?'Agent is working…':'Message agent'} onChange={e=>setText(e.target.value)} onKeyDown={e=>{
      if(e.key==='Enter'&&!e.shiftKey){e.preventDefault();void submit()}
    }}/>
    <div className="composer-bar"><div>
      <button title="Attach file" disabled={running} onClick={async()=>{const file=await bridge.pickFile();if(file)setAttachments(rows=>rows.some(row=>row.id===file.id)?rows:[...rows,file])}}>＋</button>
      <span className="model-chip">Auto</span>
    </div>
    {running?<button className="stop-button" title="Stop agent" onClick={()=>void onStop()}>■</button>:<button className="send-button" disabled={submitting||(!text.trim()&&!attachments.length)} onClick={()=>void submit()}>{submitting?'…':'↑'}</button>}</div>
  </div>;
}

function ConversationOutline({thread,onClose}:{thread:AgentThread|null;onClose():void}) {
  const turns=thread?.outline||[];
  return <aside className="conversation-outline" aria-label="Conversation outline">
    <header><strong>Outline</strong><button onClick={onClose} aria-label="Close outline">×</button></header>
    <div className="conversation-outline-list">{turns.length?turns.map((turn,index)=><section key={turn.userMessageId||String(index)}>
      <div className="outline-user">{turn.rawUserText||'Turn '+(index+1)}</div>
      {turn.items.filter(item=>item.kind!=='user').map(item=><div className={'outline-item outline-'+item.kind} key={item.id}>
        {item.kind==='assistant-text'?<><span>Reply</span><p>{item.text}</p></>:<><span>{item.name} · {item.status}</span><p>{item.summary||item.outputLocation?.filePath||'Tool activity'}</p></>}
      </div>)}
    </section>):<div className="overlay-empty">No conversation activity yet.</div>}</div>
  </aside>;
}

function OrgChart({agents,onClose,onSelect}:{agents:AgentSummary[];onClose():void;onSelect(id:string):void}) {
  const ids=new Set(agents.map(agent=>agent.id));
  const roots=agents.filter(agent=>!agent.parentAgentId||!ids.has(agent.parentAgentId));
  const renderNode=(agent:AgentSummary,depth:number):React.ReactNode=><div className="org-node" key={agent.id} style={{marginLeft:depth*18}}>
    <button onClick={()=>{onSelect(agent.id);onClose()}}><span className="avatar">{initials(agent.name)}</span><span><strong>{agent.name}</strong><small>{agent.purpose==='subagent'?'Delegated agent':'Agent'} · {agent.status}</small></span></button>
    {agents.filter(child=>child.parentAgentId===agent.id).map(child=>renderNode(child,depth+1))}
  </div>;
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay org-chart" role="dialog" aria-label="Agent org chart">
    <header><div><h2>Org chart</h2><p>Real parent and delegated-agent relationships for this workspace</p></div><button onClick={onClose}>×</button></header>
    <div className="org-chart-list">{roots.length?roots.map(root=>renderNode(root,0)):<div className="overlay-empty">No agents yet.</div>}</div>
  </section></div>;
}

function ComputerInfoPane({agent,thread,onClose}:{agent:AgentSummary;thread:AgentThread|null;onClose():void}) {
  const latest=[...(thread?.messages||[])].reverse().find(message=>message.role==='tool'&&message.display?.kind==='image');
  const active=(thread?.messages||[]).some(message=>message.role==='tool'&&['queued','waiting-approval','running','streaming'].includes(message.status||'done')&&String(message.toolName||'').startsWith('computer_'));
  return <aside className="computer-info-pane sand-info-pane" aria-label="Conversation details">
    <header><div><strong>Computer</strong><small>{active?'In use':'Local Mac'}</small></div><button aria-label="Close details" onClick={onClose}>×</button></header>
    <div className="sand-computer-monitor-strip"><span className={active?'computer-live':'computer-idle'}/><span>{agent.name}'s screen</span></div>
    <div className="sand-computer-preview">
      {latest?.display?.kind==='image'?<img src={latest.display.dataUrl} alt="Latest local computer screenshot"/>:<div className="computer-preview-empty"><span>▣</span><strong>Screen preview unavailable</strong><p>A real preview appears here after this agent captures the local Mac screen.</p></div>}
    </div>
    <div className="sand-computer-banner"><strong>Installed computer</strong><p>Computer tools operate this Mac through the coordinator → host → local-exec boundary. Mutations still pass permission and auto-review checks.</p></div>
  </aside>;
}

function AgentSettings({agent,onClose,onChanged}:{agent:AgentSummary;onClose():void;onChanged():Promise<void>}) {
  const [name,setName]=useState(agent.name),[title,setTitle]=useState(agent.title||''),[description,setDescription]=useState(agent.description||'');
  const [pending,setPending]=useState(false),[error,setError]=useState('');
  const save=async()=>{const next=name.trim();if(!next||pending)return;setPending(true);setError('');try{await bridge.updateAgent({id:agent.id,profile:{name:next,title:title.trim(),description:description.trim()}});await onChanged();onClose()}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setPending(false)}};
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay agent-settings-overlay" role="dialog" aria-label="Agent settings">
    <header><div><h2>Agent settings</h2><p>Profile and notifications for this agent</p></div><button aria-label="Close" onClick={onClose}>×</button></header>
    <div className="sand-agent-settings">
      <label><span>Name</span><input aria-label="Agent name" value={name} disabled={pending} onChange={e=>setName(e.target.value)} placeholder="Bob"/></label>
      <label><span>Title</span><input aria-label="Agent title" value={title} disabled={pending} onChange={e=>setTitle(e.target.value)} placeholder="Describe what your agent does"/></label>
      <label><span>Description</span><textarea aria-label="Agent description" value={description} disabled={pending} onChange={e=>setDescription(e.target.value)} placeholder="What this agent is for"/></label>
      <div className="agent-setting-switch"><span><strong>Notifications</strong><small>Get notified when this agent finishes or needs input</small></span><button role="switch" aria-checked={agent.notifyOnUpdatesEnabled===true} disabled={pending} onClick={async()=>{setPending(true);setError('');try{await bridge.setAgentNotifyOnUpdates({id:agent.id,isEnabled:!agent.notifyOnUpdatesEnabled});await onChanged()}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setPending(false)}}}>{agent.notifyOnUpdatesEnabled?'On':'Off'}</button></div>
      {error?<div className="settings-error">{error}</div>:null}
      <footer><button onClick={onClose}>Cancel</button><button className="primary" disabled={pending||!name.trim()} onClick={()=>void save()}>{pending?'Saving…':'Save'}</button></footer>
    </div>
  </section></div>;
}

function FindInChat({thread,onClose}:{thread:AgentThread|null;onClose():void}) {
  const [query,setQuery]=useState(''),[index,setIndex]=useState(0);
  const matches=useMemo(()=>{const q=query.trim().toLowerCase();if(!q)return[];return (thread?.messages||[]).flatMap(message=>{const text=(message.text||'').toLowerCase();const rows:{id:string;occurrence:number}[]=[];let at=text.indexOf(q),occurrence=0;while(at>=0){rows.push({id:message.id,occurrence});occurrence++;at=text.indexOf(q,at+q.length)}return rows})},[thread?.messages,query]);
  useEffect(()=>{setIndex(0)},[query]);
  const step=(delta:number)=>{if(!matches.length)return;const next=(index+delta+matches.length)%matches.length;setIndex(next);const id=matches[next]?.id;id&&document.querySelector<HTMLElement>(`[data-entry-id="${CSS.escape(id)}"]`)?.scrollIntoView({block:'center',behavior:'smooth'})};
  return <div className="sand-chat-find"><div className="sand-chat-find-bar"><span>⌕</span><input autoFocus aria-label="Find in chat" value={query} onChange={e=>setQuery(e.target.value)} placeholder="Find in chat" onKeyDown={e=>{if(e.key==='Escape'){e.preventDefault();onClose()}else if(e.key==='Enter'){e.preventDefault();step(e.shiftKey?-1:1)}}}/>{query.trim()?<span role="status">{matches.length?Math.min(index+1,matches.length):0}/{matches.length}</span>:null}<button aria-label="Previous match" disabled={!matches.length} onClick={()=>step(-1)}>↑</button><button aria-label="Next match" disabled={!matches.length} onClick={()=>step(1)}>↓</button><button aria-label="Close find" onClick={onClose}>×</button></div></div>;
}

function Workspace({agent,thread,refresh,openAutomations,openAgentSettings}:{agent:AgentSummary;thread:AgentThread|null;refresh():Promise<void>;openAutomations():void;openAgentSettings():void}) {
  const scroller=useRef<HTMLDivElement|null>(null);
  const [outlineOpen,setOutlineOpen]=useState(false);
  const [computerOpen,setComputerOpen]=useState(false);
  const [findOpen,setFindOpen]=useState(false);
  useEffect(()=>{scroller.current?.scrollTo({top:scroller.current.scrollHeight})},[thread?.messages.length]);
  useEffect(()=>{const onKey=(event:KeyboardEvent)=>{if((event.metaKey||event.ctrlKey)&&event.key.toLowerCase()==='f'){event.preventDefault();setFindOpen(true)}else if(event.key==='Escape'&&findOpen)setFindOpen(false)};window.addEventListener('keydown',onKey);return()=>window.removeEventListener('keydown',onKey)},[findOpen]);
  return <main className="workspace">
    <header className="chat-header"><div className="identity"><span className="avatar large">{initials(agent.name)}</span><span><strong>{agent.name}</strong><small><Status status={agent.status}/> {agent.status==='idle'?'Ready':agent.status}</small></span></div><nav className="header-actions"><button className="header-action sand-chat-header__computer" data-computer-active={busy(agent.status)||undefined} onClick={()=>setComputerOpen(value=>!value)}>Computer</button><button className="header-action" onClick={()=>setFindOpen(true)}>Find</button><button className="header-action" onClick={()=>setOutlineOpen(value=>!value)}>Outline</button><button className="header-action" onClick={openAgentSettings}>Agent</button><button className="header-action" onClick={openAutomations}>Routines</button></nav></header>
    {findOpen?<FindInChat thread={thread} onClose={()=>setFindOpen(false)}/>:null}
    <div className="transcript sand-virtual-transcript" ref={scroller}><div className="transcript-column">
      {thread?.messages.length?thread.messages.map(message=><Message key={message.id} message={message} onResolve={async(approvalId,approved)=>{
        await bridge.resolveApproval({approvalId,approved});await refresh();
      }}/>):<section className="welcome"><span className="avatar hero">{initials(agent.name)}</span><h2>{agent.name}</h2><p>This agent works directly on this Mac.</p></section>}
    </div></div>
    {outlineOpen?<ConversationOutline thread={thread} onClose={()=>setOutlineOpen(false)}/>:null}
    {computerOpen?<ComputerInfoPane agent={agent} thread={thread} onClose={()=>setComputerOpen(false)}/>:null}
    <Composer running={busy(agent.status)} onSend={async(text,attachmentIds)=>{await bridge.sendMessage({agentId:agent.id,text,attachmentIds});await refresh()}} onStop={async()=>{
      await bridge.stopAgent({agentId:agent.id});await refresh();
    }}/>
  </main>;
}

function HiddenChats({agents,onClose,onOpen,onChanged}:{agents:AgentSummary[];onClose():void;onOpen(id:string):void;onChanged():Promise<void>}) {
  const hidden=agents.filter(agent=>agent.hidden);
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay hidden-chats sand-hidden-chats-dialog" role="dialog" aria-modal="true" aria-label="Hidden Bots">
    <header><div><h2>Hidden Bots</h2><p>Hidden Bots stay active and keep their history, they just don't show in the sidebar.</p></div><button aria-label="Close" onClick={onClose}>×</button></header>
    <div className="hidden-chats-list">{hidden.length?hidden.map(agent=><div className="hidden-chat-row sand-hidden-chats__row" key={agent.id}>
      <button className="hidden-chat-open sand-hidden-chats__open" onClick={()=>onOpen(agent.id)}><span className="avatar">{initials(agent.name)}</span><span>{agent.name}</span></button>
      <button className="hidden-chat-unhide sand-hidden-chats__unhide" onClick={async()=>{await bridge.setAgentHidden({agentId:agent.id,hidden:false});await onChanged()}}>Unhide</button>
    </div>):<div className="hidden-chats-empty sand-hidden-chats__empty"><span>◉</span><span>No hidden bots</span></div>}</div>
  </section></div>;
}

function Plugins({items,workflows,onClose,reload,reloadWorkflows}:{items:PluginDescriptor[];workflows:WorkflowDescriptor[];onClose():void;reload():Promise<void>;reloadWorkflows():Promise<void>}) {
  const [tab,setTab]=useState<'plugins'|'marketplace'|'skills'>('plugins');
  const [query,setQuery]=useState('');
  const [addOpen,setAddOpen]=useState(false);
  const [name,setName]=useState('');
  const [transport,setTransport]=useState<'stdio'|'http'>('stdio');
  const [command,setCommand]=useState('');
  const [argsText,setArgsText]=useState('[]');
  const [url,setUrl]=useState('');
  const [accountKey,setAccountKey]=useState('default');
  const [oauthClientId,setOauthClientId]=useState('');
  const [oauthScopes,setOauthScopes]=useState('');
  const [customInstructions,setCustomInstructions]=useState('');
  const [expanded,setExpanded]=useState<string|null>(null);
  const [tools,setTools]=useState<Record<string,McpToolDescriptor[]>>({});
  const [accounts,setAccounts]=useState<Record<string,McpAccountStatus>>({});
  const [accountSlots,setAccountSlots]=useState<Record<string,McpAccountStatus[]>>({});
  const [accountDraft,setAccountDraft]=useState<Record<string,string>>({});
  const [serverConfigs,setServerConfigs]=useState<Record<string,McpServerDescriptor>>({});
  const [serverEditor,setServerEditor]=useState<null|{serverId:string;name:string;transport:'stdio'|'http';command:string;argsText:string;url:string;accountKey:string;oauthClientId:string;oauthScopes:string;customInstructions:string}>(null);
  const [authPending,setAuthPending]=useState<string|null>(null);
  const [marketplace,setMarketplace]=useState<MarketplaceCatalogDescriptor|null>(null);
  const [marketPending,setMarketPending]=useState<string|null>(null);
  const [marketValues,setMarketValues]=useState<Record<string,Record<string,string>>>({});
  const [skillEditor,setSkillEditor]=useState<WorkflowDescriptor|{id?:string;name:string;description:string;body:string;isEnabledForAgent:boolean}|null>(null);
  const [error,setError]=useState('');
  const rows=items.filter(x=>(x.name+' '+x.description+' '+x.category).toLowerCase().includes(query.toLowerCase()));
  const skillRows=workflows.filter(x=>(x.name+' '+x.description).toLowerCase().includes(query.toLowerCase()));
  const marketRows=(marketplace?.plugins||[]).filter(x=>(x.displayName+' '+x.description+' '+x.category).toLowerCase().includes(query.toLowerCase()));
  const loadAccount=async(serverId:string,key='default')=>{
    try{
      const slots=await bridge.listMcpAccounts({serverId});
      const active=slots.find(slot=>slot.active)||slots.find(slot=>slot.accountKey===key)||await bridge.getMcpAccountStatus({serverId,accountKey:key});
      setAccountSlots(current=>({...current,[serverId]:slots}));
      setAccounts(current=>({...current,[serverId]:active}));
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  const loadServerConfigs=async()=>{
    try{
      const servers=await bridge.listMcpServers();
      setServerConfigs(Object.fromEntries(servers.map(server=>[server.id,server])));
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  useEffect(()=>{
    void loadServerConfigs();
    for(const item of items)if(item.kind==='mcp'&&item.transport==='http'&&item.serverId)void loadAccount(item.serverId,item.accountKey||'default');
  },[items]);
  const loadMarketplace=async()=>{
    setError('');
    try{setMarketplace(await bridge.listMarketplacePlugins())}
    catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  useEffect(()=>{if(tab==='marketplace')void loadMarketplace()},[tab]);
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
        ...(transport==='stdio'?{command:command.trim(),args:parsed}:{url:url.trim(),accountKey:accountKey.trim()||'default',...(oauthClientId.trim()?{oauthClientId:oauthClientId.trim()}:{}),...(oauthScopes.trim()?{oauthScopes:oauthScopes.split(/[\\s,]+/).filter(Boolean)}:{})}),
        ...(customInstructions.trim()?{customInstructions:customInstructions.trim()}:{})
      });
      setName('');setTransport('stdio');setCommand('');setArgsText('[]');setUrl('');setAccountKey('default');setOauthClientId('');setOauthScopes('');setCustomInstructions('');setAddOpen(false);await reload();
    }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
  };
  const editServer=(serverId:string)=>{
    const server=serverConfigs[serverId];if(!server){void loadServerConfigs();setError('Server configuration is still loading.');return}
    setServerEditor({
      serverId,name:server.name,transport:server.transport,command:server.command,argsText:JSON.stringify(server.args||[]),
      url:server.url,accountKey:server.accountKey||'default',oauthClientId:server.oauthClientId||'',oauthScopes:(server.oauthScopes||[]).join(' '),
      customInstructions:server.customInstructions||''
    });
  };
  const saveServer=async(e:FormEvent)=>{
    e.preventDefault();if(!serverEditor)return;setError('');
    try{
      const args=JSON.parse(serverEditor.argsText||'[]');
      if(!Array.isArray(args)||args.some(value=>typeof value!=='string'))throw new Error('Args must be a JSON string array.');
      await bridge.updateMcpServer({
        serverId:serverEditor.serverId,name:serverEditor.name.trim(),transport:serverEditor.transport,
        ...(serverEditor.transport==='stdio'?{command:serverEditor.command.trim(),args}:{url:serverEditor.url.trim(),accountKey:serverEditor.accountKey.trim()||'default',oauthClientId:serverEditor.oauthClientId.trim(),oauthScopes:serverEditor.oauthScopes.split(/[\\s,]+/).filter(Boolean)}),
        customInstructions:serverEditor.customInstructions.trim()
      });
      setServerEditor(null);setExpanded(null);await Promise.all([reload(),loadServerConfigs()]);
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
      {tab==='plugins'?<button className="add-server" onClick={()=>setAddOpen(value=>!value)}>＋ MCP</button>:tab==='skills'?<button className="add-server" onClick={newSkill}>＋ Skill</button>:null}
      <button onClick={onClose}>×</button>
    </div></header>
    {tab==='plugins'&&addOpen?<form className="mcp-add" onSubmit={e=>void addServer(e)}>
      <input value={name} onChange={e=>setName(e.target.value)} placeholder="Server name" required/>
      <select value={transport} onChange={e=>setTransport(e.target.value as 'stdio'|'http')}><option value="stdio">Local stdio</option><option value="http">Remote HTTP</option></select>
      {transport==='stdio'?<><input value={command} onChange={e=>setCommand(e.target.value)} placeholder="Command, e.g. npx" required/><input value={argsText} onChange={e=>setArgsText(e.target.value)} placeholder={'["-y","@vendor/server"]'} required/></>:<><input className="mcp-url" value={url} onChange={e=>setUrl(e.target.value)} placeholder="https://server.example/mcp" required/><input value={accountKey} onChange={e=>setAccountKey(e.target.value)} placeholder="Account key (default)"/><input value={oauthClientId} onChange={e=>setOauthClientId(e.target.value)} placeholder="OAuth client ID (optional; dynamic registration is preferred)"/><input value={oauthScopes} onChange={e=>setOauthScopes(e.target.value)} placeholder="OAuth scopes (optional, comma/space separated)"/></>}
      <textarea value={customInstructions} onChange={e=>setCustomInstructions(e.target.value)} placeholder="Optional server instructions passed to the Agent with each routed tool"/>
      <div><button type="button" onClick={()=>setAddOpen(false)}>Cancel</button><button className="primary compact">Add server</button></div>
    </form>:null}
    {tab==='plugins'&&serverEditor?<form className="mcp-add" onSubmit={e=>void saveServer(e)}>
      <input value={serverEditor.name} onChange={e=>setServerEditor({...serverEditor,name:e.target.value})} placeholder="Server name" required/>
      <select value={serverEditor.transport} onChange={e=>setServerEditor({...serverEditor,transport:e.target.value as 'stdio'|'http'})}><option value="stdio">Local stdio</option><option value="http">Remote HTTP</option></select>
      {serverEditor.transport==='stdio'?<><input value={serverEditor.command} onChange={e=>setServerEditor({...serverEditor,command:e.target.value})} placeholder="Command" required/><input value={serverEditor.argsText} onChange={e=>setServerEditor({...serverEditor,argsText:e.target.value})} placeholder={'["-y","@vendor/server"]'} required/></>:<><input value={serverEditor.url} onChange={e=>setServerEditor({...serverEditor,url:e.target.value})} placeholder="https://server.example/mcp" required/><input value={serverEditor.accountKey} onChange={e=>setServerEditor({...serverEditor,accountKey:e.target.value})} placeholder="Account key"/><input value={serverEditor.oauthClientId} onChange={e=>setServerEditor({...serverEditor,oauthClientId:e.target.value})} placeholder="OAuth client ID (optional)"/><input value={serverEditor.oauthScopes} onChange={e=>setServerEditor({...serverEditor,oauthScopes:e.target.value})} placeholder="OAuth scopes"/></>}
      <textarea value={serverEditor.customInstructions} onChange={e=>setServerEditor({...serverEditor,customInstructions:e.target.value})} placeholder="Agent instructions for this server"/>
      <div><button type="button" onClick={()=>setServerEditor(null)}>Cancel</button><button className="primary compact">Save server</button></div>
    </form>:null}
    {tab==='skills'&&skillEditor?<form className="skill-editor" onSubmit={e=>void saveSkill(e)}>
      <input value={skillEditor.name} onChange={e=>setSkillEditor({...skillEditor,name:e.target.value})} placeholder="Skill name" required/>
      <input value={skillEditor.description} onChange={e=>setSkillEditor({...skillEditor,description:e.target.value})} placeholder="When should the agent use this skill?"/>
      <textarea value={skillEditor.body} onChange={e=>setSkillEditor({...skillEditor,body:e.target.value})} placeholder="Skill instructions (stored in SKILL.md)" required/>
      <label><input type="checkbox" checked={skillEditor.isEnabledForAgent} onChange={e=>setSkillEditor({...skillEditor,isEnabledForAgent:e.target.checked})}/> Available to agents</label>
      <div><button type="button" onClick={()=>setSkillEditor(null)}>Cancel</button><button className="primary compact">Save skill</button></div>
    </form>:null}
    <label className="plugin-search">⌕<input value={query} onChange={e=>setQuery(e.target.value)} placeholder={tab==='plugins'?'Search installed plugins':tab==='marketplace'?'Search marketplace':'Search skills'}/></label>
    <nav className="tabs"><button className={tab==='plugins'?'active':''} onClick={()=>setTab('plugins')}>Installed</button><button className={tab==='marketplace'?'active':''} onClick={()=>setTab('marketplace')}>Marketplace</button><button className={tab==='skills'?'active':''} onClick={()=>setTab('skills')}>Private skills</button></nav>
    {error?<div className="plugin-error">{error}</div>:null}
    {tab==='plugins'?<div className="plugin-rows">{rows.length?rows.map(x=><article className="plugin-card" key={x.id}>
      <span className="plugin-logo">{x.name[0]}</span>
      <div className="plugin-copy"><strong>{x.name}</strong><p>{x.description}</p><small>{x.category} · {x.provider||'provider'}{x.transport==='http'&&x.serverId?` · ${accounts[x.serverId]?.connected?'connected':'not connected'} · ${x.accountKey||'default'}`:''}</small>
        {x.kind==='mcp'&&x.transport==='http'&&x.serverId?<div className="mcp-account-slots">
          {(accountSlots[x.serverId!]||[]).map(slot=><div className={'account-slot '+(slot.active?'active':'')} key={slot.accountKey}>
            <button className="account-key" disabled={slot.active} onClick={async()=>{await bridge.setMcpActiveAccount({serverId:x.serverId!,accountKey:slot.accountKey});await Promise.all([loadAccount(x.serverId!,slot.accountKey),reload()])}}>{slot.accountKey}{slot.active?' · active':''}</button>
            <span>{slot.connected?'Connected':'Disconnected'}</span>
            <button onClick={async()=>{setAuthPending(x.serverId!);try{slot.connected?await bridge.disconnectMcpAccount({serverId:x.serverId!,accountKey:slot.accountKey}):await bridge.connectMcpAccount({serverId:x.serverId!,accountKey:slot.accountKey});await Promise.all([loadAccount(x.serverId!,slot.accountKey),reload()])}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setAuthPending(null)}}}>{slot.connected?'Disconnect':'Connect'}</button>
            {(accountSlots[x.serverId!]||[]).length>1?<button onClick={async()=>{await bridge.removeMcpAccount({serverId:x.serverId!,accountKey:slot.accountKey});await Promise.all([loadAccount(x.serverId!),reload()])}}>Remove</button>:null}
          </div>)}
          <div className="account-slot add"><input value={accountDraft[x.serverId!]||''} onChange={e=>setAccountDraft(current=>({...current,[x.serverId!]:e.target.value}))} placeholder="New account key"/><button disabled={!String(accountDraft[x.serverId!]||'').trim()||authPending===x.serverId} onClick={async()=>{const key=String(accountDraft[x.serverId!]||'').trim();if(!key)return;setAuthPending(x.serverId!);try{await bridge.connectMcpAccount({serverId:x.serverId!,accountKey:key});setAccountDraft(current=>({...current,[x.serverId!]:''}));await Promise.all([loadAccount(x.serverId!,key),reload()])}catch(reason){setError(reason instanceof Error?reason.message:String(reason))}finally{setAuthPending(null)}}}>Connect account</button></div>
        </div>:null}
        {x.kind==='mcp'&&x.serverId&&expanded===x.serverId?<div className="mcp-tools">
          {(tools[x.serverId]||[]).length?(tools[x.serverId]||[]).map(tool=><label key={tool.name}><span><strong>{tool.name}</strong><small>{tool.description||'MCP tool'}</small></span><input type="checkbox" checked={!tool.isDisabled} onChange={async e=>{
            const next=await bridge.setMcpToolEnabled({serverId:x.serverId!,toolName:tool.name,enabled:e.target.checked});
            setTools(current=>({...current,[x.serverId!]:next}));
          }}/></label>):<span className="tool-loading">No tools loaded. If the server is starting, retry Tools.</span>}
        </div>:null}
      </div>
      <div className="plugin-actions">
        {x.kind==='mcp'&&x.transport==='http'&&x.serverId?<button disabled={authPending===x.serverId} onClick={async()=>{
          setAuthPending(x.serverId!);setError('');
          try{
            const current=accounts[x.serverId!];
            const next=current?.connected
              ?await bridge.disconnectMcpAccount({serverId:x.serverId!,accountKey:x.accountKey||'default'})
              :await bridge.connectMcpAccount({serverId:x.serverId!,accountKey:x.accountKey||'default'});
            setAccounts(value=>({...value,[x.serverId!]:next}));
            if(next.connected){setExpanded(x.serverId!);await loadTools(x.serverId!)}else setTools(value=>({...value,[x.serverId!]:[]}));
          }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
          finally{setAuthPending(null)}
        }}>{authPending===x.serverId?'…':accounts[x.serverId]?.connected?'Disconnect':'Connect'}</button>:null}
        {x.kind==='mcp'&&x.serverId?<button onClick={()=>editServer(x.serverId!)}>Edit</button>:null}
        {x.kind==='mcp'&&x.serverId?<button onClick={async()=>{setExpanded(currentId=>currentId===x.serverId?null:x.serverId!);if(expanded!==x.serverId)await loadTools(x.serverId!)}}>Tools</button>:null}
        <button className={x.enabled?'primary compact':''} onClick={async()=>{await bridge.setPluginEnabled({pluginId:x.id,enabled:!x.enabled});await reload()}}>{x.enabled?'Enabled':'Enable'}</button>
        {x.removable?<button onClick={async()=>{await bridge.setPluginInstalled({pluginId:x.id,installed:false});if(x.serverId)setExpanded(null);await reload()}}>Remove</button>:null}
      </div>
    </article>):<div className="overlay-empty">No executable capability matches this search.</div>}</div>
    :tab==='marketplace'?<div className="plugin-rows">
      {!marketplace?<div className="overlay-empty">Loading marketplace…</div>
      :!marketplace.available?<div className="overlay-empty">{marketplace.reason||'Marketplace provider is unavailable.'}</div>
      :marketRows.length?marketRows.map(plugin=><article className="plugin-card marketplace-card" key={plugin.id}>
        <span className="plugin-logo">{plugin.displayName[0]||'P'}</span>
        <div className="plugin-copy"><strong>{plugin.displayName}</strong><p>{plugin.description}</p><small>{plugin.category}{plugin.publisher?.displayName?' · '+plugin.publisher.displayName:''}</small>
          {plugin.connectors.length?<div className="market-meta"><span>MCP</span>{plugin.connectors.map(row=><em key={row.name}>{row.name}</em>)}</div>:null}
          {plugin.skills.length?<div className="market-meta"><span>Skills</span>{plugin.skills.map(row=><em key={row.name}>{row.name}</em>)}</div>:null}
          {!plugin.installed&&plugin.fields.length?<div className="market-fields">{plugin.fields.map(field=><label key={field.key}><span>{field.label}{field.isRequired?' *':''}</span><input type={field.isSecret?'password':'text'} value={marketValues[plugin.id]?.[field.key]??field.defaultValue??''} placeholder={field.placeholder} onChange={e=>setMarketValues(current=>({...current,[plugin.id]:{...(current[plugin.id]||{}),[field.key]:e.target.value}}))}/>{field.hint?<small>{field.hint}</small>:null}</label>)}</div>:null}
        </div>
        <div className="plugin-actions">
          {plugin.homepage?<button onClick={()=>void window.open(plugin.homepage!,'_blank','noopener,noreferrer')}>Homepage</button>:null}
          <button className={plugin.installed?'':'primary compact'} disabled={marketPending===plugin.id} onClick={async()=>{
            setMarketPending(plugin.id);setError('');
            try{
              const next=plugin.installed
                ?await bridge.uninstallMarketplacePlugin({entryId:plugin.id})
                :await bridge.installMarketplacePlugin({entryId:plugin.id,values:marketValues[plugin.id]||{}});
              setMarketplace(next);await Promise.all([reload(),reloadWorkflows(),loadServerConfigs()]);
            }catch(reason){setError(reason instanceof Error?reason.message:String(reason))}
            finally{setMarketPending(null)}
          }}>{marketPending===plugin.id?'…':plugin.installed?'Uninstall':'Install'}</button>
        </div>
      </article>):<div className="overlay-empty">No marketplace plugin matches this search.</div>}
    </div>
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

function AutoReviewRules({settings,onChange}:{settings:RuntimeSettings;onChange(next:RuntimeSettings):void}) {
  const [draft,setDraft]=useState('');
  const [behavior,setBehavior]=useState<'allow'|'ask'>('allow');
  const [pending,setPending]=useState(false);
  const rows=[
    ...settings.autoReviewAllowInstructions.map((text,index)=>({behavior:'allow' as const,index,text})),
    ...settings.autoReviewBlockInstructions.map((text,index)=>({behavior:'ask' as const,index,text}))
  ];
  const commit=async(allowInstructions:string[],blockInstructions:string[])=>{
    setPending(true);
    try{onChange(await bridge.setAutoReviewInstructions({allowInstructions,blockInstructions}))}finally{setPending(false)}
  };
  const add=async()=>{
    const value=draft.trim();if(!value||pending)return;
    const allow=[...settings.autoReviewAllowInstructions],block=[...settings.autoReviewBlockInstructions];
    (behavior==='allow'?allow:block).push(value.slice(0,1000));
    await commit(allow,block);setDraft('');setBehavior('allow');
  };
  const remove=async(row:{behavior:'allow'|'ask';index:number})=>{
    const allow=[...settings.autoReviewAllowInstructions],block=[...settings.autoReviewBlockInstructions];
    if(row.behavior==='allow')allow.splice(row.index,1);else block.splice(row.index,1);
    await commit(allow,block);
  };
  return <section className="sand-auto-review">
    <div className="auto-review-rule-entry"><input value={draft} disabled={pending} maxLength={1000} onChange={e=>setDraft(e.target.value)} placeholder="e.g. reply to emails for me" onKeyDown={e=>{if(e.key==='Enter'&&(e.metaKey||e.ctrlKey))void add()}}/><select value={behavior} disabled={pending} onChange={e=>setBehavior(e.target.value as 'allow'|'ask')}><option value="allow">Allow automatically</option><option value="ask">Ask first</option></select><button disabled={pending||!draft.trim()} onClick={()=>void add()}>Add Rule</button></div>
    {rows.length?<div className="auto-review-rules" role="table" aria-label="Auto-review rules">{rows.map((row,i)=><div className="auto-review-rule" role="row" key={row.behavior+':'+row.index+':'+row.text}><span role="cell" title={row.text}>{row.text}</span><span role="cell">{row.behavior==='allow'?'Allow automatically':'Ask first'}</span><button aria-label={'Delete rule '+(i+1)} disabled={pending} onClick={()=>void remove(row)}>Delete</button></div>)}</div>:null}
    <small>Ask first takes priority if rules conflict. Built-in safety checks always apply.</small>
  </section>;
}

function Settings({onClose}:{onClose():void}) {
  const [settings,setSettings]=useState<RuntimeSettings|null>(null);
  const [error,setError]=useState('');
  useEffect(()=>{void bridge.getRuntimeSettings().then(setSettings,e=>setError(String(e)))},[]);
  return <div className="shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="overlay settings sand-settings-dialog" role="dialog" aria-label="Settings">
    <header><div><h2>Settings</h2><p>Agent execution and local-computer permissions</p></div><button onClick={onClose}>×</button></header>
    <div className="setting-row sand-settings-general"><div><strong>Computer</strong><p>Agents operate on the Mac where Fabushi is installed.</p></div><span className="local-pill">Local Mac</span></div>
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
    {settings?<div className="setting-stack"><div><strong>Auto-review Rules</strong><p>Customize which ordinary actions can run automatically and which must ask first.</p></div><AutoReviewRules settings={settings} onChange={setSettings}/></div>:null}
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

function CommandPalette({agents,onClose,onSelect,onCreate,onOrgChart,onHiddenChats,onPlugins,onSettings}:{agents:AgentSummary[];onClose():void;onSelect(id:string):void;onCreate():void;onOrgChart():void;onHiddenChats():void;onPlugins():void;onSettings():void}) {
  const [query,setQuery]=useState('');
  const actions=[
    {id:'new',label:'New agent',run:onCreate},
    {id:'orgchart',label:'Open Org chart',run:onOrgChart},
    {id:'hidden',label:'Open Hidden Bots',run:onHiddenChats},
    {id:'plugins',label:'Open Plugins',run:onPlugins},
    {id:'settings',label:'Open Settings',run:onSettings},
    ...agents.map(agent=>({id:'agent:'+agent.id,label:'Open '+agent.name,run:()=>onSelect(agent.id)}))
  ].filter(item=>item.label.toLowerCase().includes(query.toLowerCase()));
  return <div className="shade palette-shade" onMouseDown={e=>e.currentTarget===e.target&&onClose()}><section className="command-palette sand-command-palette" role="dialog" aria-label="Command palette">
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
  const [overlay,setOverlay]=useState<'plugins'|'settings'|'automations'|'orgchart'|'hidden'|'agent-settings'|null>(null);
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
  if(failure&&!agents.length)return <div className="root-state error-state sand-error-boundary--app"><strong>Fabushi could not load the agent runtime.</strong><p>{failure}</p><button className="primary" onClick={()=>void refreshAll()}>Retry</button></div>;

  return <div className="app-shell">
    <Sidebar agents={agents} selected={selectedId} query={query} setQuery={setQuery} select={setSelectedId} create={()=>setCreateOpen(true)}
      plugins={()=>setOverlay('plugins')} settings={()=>setOverlay('settings')} orgChart={()=>setOverlay('orgchart')} hiddenChats={()=>setOverlay('hidden')} rename={setRenameAgent} hide={agent=>{void bridge.setAgentHidden({agentId:agent.id,hidden:true}).then(loadAgents).catch(error=>setFailure(error instanceof Error?error.message:String(error)))}} remove={setDeleteAgent} openPalette={()=>setPaletteOpen(true)}/>
    {selected?<Workspace agent={selected} thread={thread} refresh={()=>loadThread(selected.id)} openAutomations={()=>setOverlay('automations')} openAgentSettings={()=>setOverlay('agent-settings')}/ >:<main className="empty"><span>✣</span><h1>What should your agent do?</h1><p>Create an agent for a project, task, research thread, or workflow.</p><button className="primary" onClick={()=>setCreateOpen(true)}>New agent</button></main>}
    {failure?<div className="toast-error">{failure}<button onClick={()=>setFailure('')}>×</button></div>:null}
    {overlay==='plugins'?<Plugins items={plugins} workflows={workflows} onClose={()=>setOverlay(null)} reload={loadPlugins} reloadWorkflows={loadWorkflows}/>:null}
    {overlay==='settings'?<Settings onClose={()=>setOverlay(null)}/>:null}
    {overlay==='automations'&&selected?<Automations agent={selected} onClose={()=>setOverlay(null)}/>:null}
    {overlay==='orgchart'?<OrgChart agents={agents} onClose={()=>setOverlay(null)} onSelect={setSelectedId}/>:null}
    {overlay==='hidden'?<HiddenChats agents={agents} onClose={()=>setOverlay(null)} onOpen={id=>{setSelectedId(id);setOverlay(null)}} onChanged={loadAgents}/>:null}
    {overlay==='agent-settings'&&selected?<AgentSettings agent={selected} onClose={()=>setOverlay(null)} onChanged={loadAgents}/>:null}
    {createOpen?<CreateAgent onClose={()=>setCreateOpen(false)} onCreated={agent=>{setCreateOpen(false);void loadAgents().then(()=>setSelectedId(agent.id))}}/>:null}
    {renameAgent?<RenameAgent agent={renameAgent} onClose={()=>setRenameAgent(null)} onSaved={loadAgents}/>:null}
    {deleteAgent?<DeleteAgent agent={deleteAgent} onClose={()=>setDeleteAgent(null)} onDeleted={async()=>{await loadAgents();setThread(null)}}/>:null}
    {paletteOpen?<CommandPalette agents={agents} onClose={()=>setPaletteOpen(false)} onSelect={setSelectedId} onCreate={()=>setCreateOpen(true)} onOrgChart={()=>setOverlay('orgchart')} onHiddenChats={()=>setOverlay('hidden')} onPlugins={()=>setOverlay('plugins')} onSettings={()=>setOverlay('settings')}/>:null}
  </div>;
}

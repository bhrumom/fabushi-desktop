'use strict';

function isLoopback(hostname){
  return hostname==='localhost'||hostname==='127.0.0.1'||hostname==='::1'||hostname==='[::1]';
}
function normalizeProviderUrl(value){
  const raw=String(value||'').trim();
  if(!raw)return null;
  let url;try{url=new URL(raw)}catch{throw Error('Plugin marketplace URL is invalid.')}
  if(url.protocol!=='https:'&&!(url.protocol==='http:'&&isLoopback(url.hostname)))throw Error('Plugin marketplace URL must use HTTPS (HTTP is allowed only for loopback development).');
  url.pathname=url.pathname.replace(/\/+$/,'')+'/';
  url.search='';url.hash='';
  return url;
}
function asArray(value){return Array.isArray(value)?value:[]}
function cleanText(value,max=2000){return String(value??'').trim().slice(0,max)}
function normalizeField(value){
  if(!value||typeof value!=='object')return null;
  const key=cleanText(value.key,120);if(!key)return null;
  return{
    key,
    label:cleanText(value.label||key,160),
    placeholder:cleanText(value.placeholder||key,300),
    isRequired:value.isRequired===true,
    isSecret:value.isSecret===true,
    ...(value.defaultValue==null?{}:{defaultValue:cleanText(value.defaultValue,4000)}),
    ...(value.hint==null?{}:{hint:cleanText(value.hint,1000)})
  };
}
function normalizeCatalogEntry(value){
  if(!value||typeof value!=='object')return null;
  const id=cleanText(value.id||value.pluginId,180);
  const name=cleanText(value.name||value.displayName,160);
  if(!id||!name)return null;
  const connectors=asArray(value.connectors).map(row=>({
    name:cleanText(row?.name,160),description:cleanText(row?.description,1000)
  })).filter(row=>row.name);
  const skills=asArray(value.skills).map(row=>({
    name:cleanText(row?.name,160),description:cleanText(row?.description,1000)
  })).filter(row=>row.name);
  return{
    id,name,
    displayName:cleanText(value.displayName||name,160),
    description:cleanText(value.description,3000),
    category:cleanText(value.category||'MCP',120),
    homepage:cleanText(value.homepage,1000)||null,
    iconUrl:cleanText(value.iconUrl||value.logoUrl,2000)||null,
    connectors,skills,
    fields:asArray(value.fields||value.variableFields).map(normalizeField).filter(Boolean),
    publisher:value.publisher&&typeof value.publisher==='object'?{
      name:cleanText(value.publisher.name,160),
      displayName:cleanText(value.publisher.displayName||value.publisher.name,160),
      isUserOwned:value.publisher.isUserOwned===true
    }:null,
    marketplace:value.marketplace&&typeof value.marketplace==='object'?{
      name:cleanText(value.marketplace.name,160),
      displayName:cleanText(value.marketplace.displayName||value.marketplace.name,160),
      ownership:value.marketplace.ownership==='team'?'team':'user'
    }:null
  };
}
function normalizeInstallPayload(value){
  if(!value||typeof value!=='object')throw Error('Marketplace install response is invalid.');
  const servers=asArray(value.servers||value.mcpServers).filter(row=>row&&typeof row==='object');
  const skills=asArray(value.skills).map(row=>({
    name:cleanText(row?.name,160),
    description:cleanText(row?.description,1000),
    body:String(row?.body??row?.instructions??'').slice(0,200000),
    isEnabledForAgent:row?.isEnabledForAgent!==false,
    disableModelInvocation:row?.disableModelInvocation===true
  })).filter(row=>row.name&&row.body);
  if(!servers.length&&!skills.length)throw Error('Marketplace plugin did not return any executable MCP server or private skill.');
  return{servers,skills};
}
function createPluginMarketplace({env=process.env,fetchImpl=globalThis.fetch}={}){
  const base=normalizeProviderUrl(env.FABUSHI_PLUGIN_MARKETPLACE_URL);
  const token=cleanText(env.FABUSHI_PLUGIN_MARKETPLACE_TOKEN,16000);
  const headers=()=>({
    accept:'application/json',
    ...(token?{authorization:'Bearer '+token}:{})
  });
  async function request(path,options={}){
    if(!base)throw Error('Plugin marketplace provider is not configured.');
    const url=new URL(path.replace(/^\//,''),base);
    const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),12000);
    try{
      const response=await fetchImpl(url,{
        ...options,
        headers:{...headers(),...(options.body?{'content-type':'application/json'}:{}),...(options.headers||{})},
        signal:controller.signal
      });
      const raw=await response.text();
      if(!response.ok)throw Error('Plugin marketplace HTTP '+response.status+': '+raw.slice(0,1200));
      if(!raw.trim())return{};
      try{return JSON.parse(raw)}catch{throw Error('Plugin marketplace returned invalid JSON.')}
    }finally{clearTimeout(timer)}
  }
  return{
    configured:Boolean(base),
    providerUrl:base?.toString()||null,
    async list(){
      if(!base)return{
        available:false,
        reason:'The reference marketplace is backed by an authenticated external DashboardService and is not self-contained in the reconstructed repository. Configure FABUSHI_PLUGIN_MARKETPLACE_URL to use a compatible executable provider.',
        plugins:[],
        includesPrivateMarketplaces:false
      };
      const payload=await request('plugins');
      const rows=Array.isArray(payload)?payload:asArray(payload.plugins);
      return{
        available:true,reason:null,
        includesPrivateMarketplaces:payload?.includesPrivateMarketplaces===true,
        plugins:rows.map(normalizeCatalogEntry).filter(Boolean).sort((a,b)=>a.displayName.localeCompare(b.displayName))
      };
    },
    async install(entryId,values={}){
      const id=cleanText(entryId,180);if(!id)throw Error('Marketplace plugin id is required.');
      const safeValues={};
      for(const [key,value] of Object.entries(values||{}))safeValues[cleanText(key,120)]=String(value??'').slice(0,20000);
      return normalizeInstallPayload(await request('plugins/'+encodeURIComponent(id)+'/install',{
        method:'POST',body:JSON.stringify({values:safeValues})
      }));
    }
  };
}

module.exports={createPluginMarketplace,normalizeProviderUrl,normalizeCatalogEntry,normalizeInstallPayload};

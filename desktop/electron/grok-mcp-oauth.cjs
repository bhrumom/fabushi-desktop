'use strict';

const crypto=require('node:crypto');
const http=require('node:http');

const OAUTH_TIMEOUT_MS=15*60*1000;

function normalizeAccountKey(value){
  const key=String(value||'default').trim().slice(0,80)||'default';
  if(!/^[A-Za-z0-9._-]+$/.test(key))throw Error('MCP account key may contain only letters, numbers, dot, underscore, and dash.');
  return key;
}
function oauthSecretKey(serverId,accountKey){
  return 'mcp-oauth:'+String(serverId)+':'+normalizeAccountKey(accountKey);
}
function base64url(value){
  return Buffer.from(value).toString('base64').replace(/=/g,'').replace(/\+/g,'-').replace(/\//g,'_');
}
function parseChallenge(header){
  const value=String(header||'');
  const read=name=>{
    const match=value.match(new RegExp(name+'\\s*=\\s*"([^"]+)"','i'));
    return match?.[1]||null;
  };
  return{resourceMetadata:read('resource_metadata'),scope:read('scope')};
}
async function fetchWithTimeout(url,options={},timeoutMs=15000){
  const controller=new AbortController();
  const upstream=options.signal;
  const abort=()=>controller.abort();
  if(upstream){if(upstream.aborted)controller.abort();else upstream.addEventListener('abort',abort,{once:true})}
  const timer=setTimeout(()=>controller.abort(),timeoutMs);
  try{return await fetch(url,{...options,signal:controller.signal})}
  finally{clearTimeout(timer);upstream?.removeEventListener('abort',abort)}
}
async function jsonResponse(url,options,description){
  const response=await fetchWithTimeout(url,options);
  if(!response.ok)throw Error(description+' HTTP '+response.status+': '+(await response.text()).slice(0,1000));
  const value=await response.json();
  if(!value||typeof value!=='object')throw Error(description+' returned invalid JSON.');
  return value;
}
function metadataCandidates(issuer){
  const url=new URL(issuer);
  const path=url.pathname==='/'?'':url.pathname.replace(/\/$/,'');
  return[
    new URL('/.well-known/oauth-authorization-server'+path,url.origin).toString(),
    new URL('/.well-known/oauth-authorization-server',url.origin).toString()
  ];
}
async function probeProtectedResource(server,signal){
  const body={jsonrpc:'2.0',id:'fabushi-oauth-probe',method:'initialize',params:{protocolVersion:'2025-06-18',capabilities:{},clientInfo:{name:'Fabushi',version:'2.0.0-alpha.1'}}};
  const response=await fetchWithTimeout(server.url,{
    method:'POST',
    headers:{'content-type':'application/json',accept:'application/json, text/event-stream'},
    body:JSON.stringify(body),
    signal
  });
  if(response.status!==401){
    if(response.ok)throw Error('This MCP server did not request OAuth authentication.');
    throw Error('MCP auth probe HTTP '+response.status+': '+(await response.text()).slice(0,1000));
  }
  return parseChallenge(response.headers.get('www-authenticate'));
}
async function discoverOAuth(server,signal){
  if(server.oauthAuthorizationUrl&&server.oauthTokenUrl){
    return{
      authorizationEndpoint:server.oauthAuthorizationUrl,
      tokenEndpoint:server.oauthTokenUrl,
      registrationEndpoint:server.oauthRegistrationUrl||null,
      scopes:Array.isArray(server.oauthScopes)?server.oauthScopes:[],
      clientId:server.oauthClientId||null
    };
  }
  const challenge=await probeProtectedResource(server,signal);
  const target=new URL(server.url);
  const resourceMetadataUrl=challenge.resourceMetadata||new URL('/.well-known/oauth-protected-resource',target.origin).toString();
  const resource=await jsonResponse(resourceMetadataUrl,{signal},'OAuth protected-resource metadata');
  const issuer=Array.isArray(resource.authorization_servers)?String(resource.authorization_servers[0]||''):String(resource.authorization_server||'');
  if(!issuer)throw Error('OAuth protected-resource metadata did not identify an authorization server.');
  let metadata=null,lastError=null;
  for(const candidate of metadataCandidates(issuer)){
    try{metadata=await jsonResponse(candidate,{signal},'OAuth authorization-server metadata');break}
    catch(error){lastError=error}
  }
  if(!metadata)throw lastError||Error('OAuth authorization-server metadata is unavailable.');
  const authorizationEndpoint=String(metadata.authorization_endpoint||'');
  const tokenEndpoint=String(metadata.token_endpoint||'');
  if(!authorizationEndpoint||!tokenEndpoint)throw Error('OAuth metadata is missing authorization_endpoint or token_endpoint.');
  const scopes=Array.isArray(resource.scopes_supported)?resource.scopes_supported.map(String):Array.isArray(metadata.scopes_supported)?metadata.scopes_supported.map(String):challenge.scope?challenge.scope.split(/\s+/).filter(Boolean):[];
  return{
    authorizationEndpoint,
    tokenEndpoint,
    registrationEndpoint:metadata.registration_endpoint?String(metadata.registration_endpoint):null,
    scopes,
    clientId:server.oauthClientId||null
  };
}
async function registerClient(endpoint,redirectUri,signal){
  const response=await jsonResponse(endpoint,{
    method:'POST',
    headers:{'content-type':'application/json'},
    body:JSON.stringify({
      client_name:'Fabushi',
      redirect_uris:[redirectUri],
      grant_types:['authorization_code','refresh_token'],
      response_types:['code'],
      token_endpoint_auth_method:'none'
    }),
    signal
  },'OAuth dynamic client registration');
  const clientId=String(response.client_id||'');
  if(!clientId)throw Error('OAuth client registration returned no client_id.');
  return clientId;
}
function startLoopback(){
  let settle;
  const pending=new Promise((resolve,reject)=>{settle={resolve,reject}});
  let expectedState=null,done=false,timer=null;
  const server=http.createServer((req,res)=>{
    let url;
    try{url=new URL(req.url||'/', 'http://127.0.0.1')}catch{res.writeHead(400).end('Bad request');return}
    if(req.method!=='GET'||url.pathname!=='/callback'){res.writeHead(404).end('Not found');return}
    if(done){res.writeHead(410).end('OAuth flow already completed');return}
    const state=url.searchParams.get('state');
    if(!expectedState||state!==expectedState){res.writeHead(400).end('Invalid OAuth state');return}
    const error=url.searchParams.get('error');
    if(error){
      done=true;res.writeHead(400,{'content-type':'text/html; charset=utf-8'}).end('<h1>Fabushi connector authorization failed</h1>');
      settle.reject(Error('OAuth provider returned '+error));return;
    }
    const code=url.searchParams.get('code');
    if(!code){res.writeHead(400).end('Missing authorization code');return}
    done=true;res.writeHead(200,{'content-type':'text/html; charset=utf-8'}).end('<h1>Fabushi connector authorized</h1><p>You can close this window.</p>');
    settle.resolve(code);
  });
  return new Promise((resolve,reject)=>{
    server.once('error',reject);
    server.listen(0,'127.0.0.1',()=>{
      server.off('error',reject);server.unref();
      const address=server.address();
      if(!address||typeof address==='string'){server.close();reject(Error('OAuth loopback listener failed.'));return}
      const redirectUri='http://127.0.0.1:'+address.port+'/callback';
      resolve({
        redirectUri,
        setState(value){expectedState=String(value);timer=setTimeout(()=>{if(done)return;done=true;settle.reject(Error('OAuth authorization timed out.'))},OAUTH_TIMEOUT_MS);timer.unref?.()},
        wait:()=>pending,
        close:async()=>{if(timer)clearTimeout(timer);await new Promise(doneClose=>server.close(()=>doneClose())).catch(()=>{})}
      });
    });
  });
}
async function exchangeToken({tokenEndpoint,clientId,code,redirectUri,verifier,resource,signal}){
  const body=new URLSearchParams({
    grant_type:'authorization_code',client_id:clientId,code,redirect_uri:redirectUri,code_verifier:verifier,resource
  });
  const response=await jsonResponse(tokenEndpoint,{
    method:'POST',
    headers:{'content-type':'application/x-www-form-urlencoded'},
    body:body.toString(),
    signal
  },'OAuth token exchange');
  const accessToken=String(response.access_token||'');
  if(!accessToken)throw Error('OAuth token exchange returned no access_token.');
  return{
    accessToken,
    refreshToken:response.refresh_token?String(response.refresh_token):null,
    tokenType:String(response.token_type||'Bearer'),
    expiresAt:Number.isFinite(Number(response.expires_in))?Date.now()+Number(response.expires_in)*1000:null,
    scope:String(response.scope||''),
    tokenEndpoint,clientId,resource
  };
}
async function refreshToken(record,signal){
  if(!record?.refreshToken)return record;
  const body=new URLSearchParams({
    grant_type:'refresh_token',
    refresh_token:record.refreshToken,
    client_id:record.clientId,
    resource:record.resource
  });
  const response=await jsonResponse(record.tokenEndpoint,{
    method:'POST',
    headers:{'content-type':'application/x-www-form-urlencoded'},
    body:body.toString(),
    signal
  },'OAuth token refresh');
  const accessToken=String(response.access_token||'');
  if(!accessToken)throw Error('OAuth token refresh returned no access_token.');
  return{
    ...record,
    accessToken,
    refreshToken:response.refresh_token?String(response.refresh_token):record.refreshToken,
    tokenType:String(response.token_type||record.tokenType||'Bearer'),
    expiresAt:Number.isFinite(Number(response.expires_in))?Date.now()+Number(response.expires_in)*1000:null,
    scope:String(response.scope||record.scope||'')
  };
}
function createMcpOAuthManager({getServer,tokenStore,openExternal,onChanged=()=>{}}){
  async function status(serverId,accountKey){
    const key=normalizeAccountKey(accountKey);
    const token=await tokenStore.get(oauthSecretKey(serverId,key));
    return{serverId:String(serverId),accountKey:key,connected:Boolean(token?.accessToken),expiresAt:Number.isFinite(token?.expiresAt)?token.expiresAt:null,scope:String(token?.scope||'')};
  }
  async function authorizationHeader(server,signal){
    const accountKey=normalizeAccountKey(server.accountKey);
    const key=oauthSecretKey(server.id,accountKey);
    let token=await tokenStore.get(key);
    if(!token?.accessToken)return null;
    if(Number.isFinite(token.expiresAt)&&token.expiresAt<=Date.now()+60_000&&token.refreshToken){
      token=await refreshToken(token,signal);await tokenStore.set(key,token);onChanged(server.id);
    }
    return String(token.tokenType||'Bearer')+' '+String(token.accessToken);
  }
  async function connect(serverId,accountKey='default'){
    const server=await getServer(String(serverId));
    if(!server)throw Error('MCP server not found.');
    if(server.transport!=='http')throw Error('Only HTTP MCP servers support OAuth accounts.');
    const key=normalizeAccountKey(accountKey);
    const controller=new AbortController();
    const oauth=await discoverOAuth(server,controller.signal);
    const loopback=await startLoopback();
    try{
      const clientId=oauth.clientId|| (oauth.registrationEndpoint?await registerClient(oauth.registrationEndpoint,loopback.redirectUri,controller.signal):null);
      if(!clientId)throw Error('OAuth server requires a client_id and does not expose dynamic client registration. Configure oauthClientId for this MCP server.');
      const verifier=base64url(crypto.randomBytes(48));
      const challenge=base64url(crypto.createHash('sha256').update(verifier).digest());
      const state=base64url(crypto.randomBytes(24));
      loopback.setState(state);
      const authorization=new URL(oauth.authorizationEndpoint);
      authorization.searchParams.set('response_type','code');
      authorization.searchParams.set('client_id',clientId);
      authorization.searchParams.set('redirect_uri',loopback.redirectUri);
      authorization.searchParams.set('code_challenge',challenge);
      authorization.searchParams.set('code_challenge_method','S256');
      authorization.searchParams.set('state',state);
      authorization.searchParams.set('resource',server.url);
      if(oauth.scopes.length)authorization.searchParams.set('scope',oauth.scopes.join(' '));
      await openExternal(authorization.toString());
      const code=await loopback.wait();
      const token=await exchangeToken({tokenEndpoint:oauth.tokenEndpoint,clientId,code,redirectUri:loopback.redirectUri,verifier,resource:server.url,signal:controller.signal});
      await tokenStore.set(oauthSecretKey(server.id,key),token);
      onChanged(server.id);
      return await status(server.id,key);
    }finally{controller.abort();await loopback.close()}
  }
  async function disconnect(serverId,accountKey='default'){
    const key=normalizeAccountKey(accountKey);
    await tokenStore.remove(oauthSecretKey(serverId,key));
    onChanged(String(serverId));
    return await status(serverId,key);
  }
  async function rename(serverId,from,to){
    const source=normalizeAccountKey(from),target=normalizeAccountKey(to);
    await tokenStore.move(oauthSecretKey(serverId,source),oauthSecretKey(serverId,target));
    onChanged(String(serverId));
    return await status(serverId,target);
  }
  return{connect,disconnect,rename,status,authorizationHeader,discoverOAuth};
}

module.exports={createMcpOAuthManager,normalizeAccountKey,oauthSecretKey,parseChallenge,discoverOAuth};

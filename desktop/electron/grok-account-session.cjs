'use strict';

const crypto=require('node:crypto');
const http=require('node:http');

const SESSION_KEY='fabushi-account-session';
function isLoopback(host){return host==='127.0.0.1'||host==='localhost'||host==='::1'||host==='[::1]'}
function endpoint(value,name){
  const raw=String(value||'').trim();if(!raw)return null;
  const url=new URL(raw);if(url.protocol!=='https:'&&!(url.protocol==='http:'&&isLoopback(url.hostname)))throw Error(name+' must use HTTPS (HTTP is allowed only for loopback development).');
  return url.toString();
}
function b64url(value){return Buffer.from(value).toString('base64').replace(/=/g,'').replace(/\+/g,'-').replace(/\//g,'_')}
function decodeJwtPayload(token){
  try{const part=String(token||'').split('.')[1];if(!part)return{};const normalized=part.replace(/-/g,'+').replace(/_/g,'/');return JSON.parse(Buffer.from(normalized,'base64').toString('utf8'))}catch{return{}}
}
function normalizeProfile(value={}){
  const claims=value&&typeof value==='object'?value:{};
  return{
    authId:String(claims.sub||claims.id||claims.authId||'').slice(0,300)||null,
    email:String(claims.email||'').slice(0,500)||null,
    displayName:String(claims.name||claims.displayName||claims.preferred_username||'').slice(0,300)||null,
    avatarUrl:String(claims.picture||claims.avatar_url||'').slice(0,3000)||null
  };
}
function startLoopback(){
  let state=null,settled=false,resolveCode,rejectCode;
  const pending=new Promise((resolve,reject)=>{resolveCode=resolve;rejectCode=reject});
  const server=http.createServer((req,res)=>{
    const url=new URL(req.url||'/','http://127.0.0.1');
    if(req.method!=='GET'||url.pathname!=='/callback'){res.writeHead(404).end('Not found');return}
    if(settled){res.writeHead(410).end('Already completed');return}
    if(!state||url.searchParams.get('state')!==state){res.writeHead(400).end('Invalid state');return}
    const error=url.searchParams.get('error');if(error){settled=true;res.writeHead(400,{'content-type':'text/html'}).end('<h1>Fabushi sign-in failed</h1>');rejectCode(Error('OAuth provider returned '+error));return}
    const code=url.searchParams.get('code');if(!code){res.writeHead(400).end('Missing code');return}
    settled=true;res.writeHead(200,{'content-type':'text/html'}).end('<h1>Fabushi sign-in complete</h1><p>You can close this window.</p>');resolveCode(code);
  });
  return new Promise((resolve,reject)=>{
    server.once('error',reject);
    server.listen(0,'127.0.0.1',()=>{
      server.off('error',reject);server.unref();
      const addr=server.address();if(!addr||typeof addr==='string'){server.close();reject(Error('OAuth loopback failed.'));return}
      resolve({
        redirectUri:'http://127.0.0.1:'+addr.port+'/callback',
        setState(value){state=String(value)},
        wait:()=>pending,
        cancel(reason='Sign-in cancelled.'){if(settled)return;settled=true;rejectCode(Error(reason))},
        close:()=>new Promise(done=>server.close(()=>done())).catch(()=>{})
      });
    });
  });
}
function createAccountSession({env=process.env,secretStore,openExternal,fetchImpl=globalThis.fetch,onChanged=()=>{}}){
  const authorizationUrl=endpoint(env.FABUSHI_ACCOUNT_AUTHORIZATION_URL,'Account authorization URL');
  const tokenUrl=endpoint(env.FABUSHI_ACCOUNT_TOKEN_URL,'Account token URL');
  const userinfoUrl=endpoint(env.FABUSHI_ACCOUNT_USERINFO_URL,'Account userinfo URL');
  const clientId=String(env.FABUSHI_ACCOUNT_CLIENT_ID||'').trim();
  const scopes=String(env.FABUSHI_ACCOUNT_SCOPES||'openid profile email').trim().split(/\s+/).filter(Boolean);
  const configured=Boolean(authorizationUrl&&tokenUrl&&clientId);
  let active=null;
  async function record(){return await secretStore.get(SESSION_KEY)}
  async function status(){
    const saved=await record();
    if(active)return{kind:'logging-in',available:configured};
    if(!saved?.accessToken)return{kind:'logged-out',available:configured,reason:configured?null:'Account OAuth provider is not configured.'};
    return{kind:'logged-in',available:true,authId:saved.profile?.authId||null,email:saved.profile?.email||null,displayName:saved.profile?.displayName||null,avatarUrl:saved.profile?.avatarUrl||null};
  }
  async function login(){
    if(!configured)throw Error('Account OAuth provider is not configured.');
    if(active)return status();
    const loopback=await startLoopback(),controller=new AbortController();
    active={loopback,controller};onChanged(await status());
    try{
      const verifier=b64url(crypto.randomBytes(48)),challenge=b64url(crypto.createHash('sha256').update(verifier).digest()),state=b64url(crypto.randomBytes(24));
      loopback.setState(state);
      const auth=new URL(authorizationUrl);auth.searchParams.set('response_type','code');auth.searchParams.set('client_id',clientId);auth.searchParams.set('redirect_uri',loopback.redirectUri);auth.searchParams.set('code_challenge',challenge);auth.searchParams.set('code_challenge_method','S256');auth.searchParams.set('state',state);if(scopes.length)auth.searchParams.set('scope',scopes.join(' '));
      await openExternal(auth.toString());
      const code=await loopback.wait();
      const body=new URLSearchParams({grant_type:'authorization_code',client_id:clientId,code,redirect_uri:loopback.redirectUri,code_verifier:verifier});
      const response=await fetchImpl(tokenUrl,{method:'POST',headers:{'content-type':'application/x-www-form-urlencoded'},body:body.toString(),signal:controller.signal});
      if(!response.ok)throw Error('Account token exchange HTTP '+response.status+': '+(await response.text()).slice(0,800));
      const token=await response.json();const accessToken=String(token.access_token||'');if(!accessToken)throw Error('Account token exchange returned no access_token.');
      let profile=normalizeProfile(decodeJwtPayload(token.id_token));
      if(userinfoUrl){
        const info=await fetchImpl(userinfoUrl,{headers:{authorization:(token.token_type||'Bearer')+' '+accessToken},signal:controller.signal});
        if(info.ok)profile=normalizeProfile({...decodeJwtPayload(token.id_token),...(await info.json())});
      }
      await secretStore.set(SESSION_KEY,{accessToken,refreshToken:token.refresh_token?String(token.refresh_token):null,tokenType:String(token.token_type||'Bearer'),expiresAt:Number.isFinite(Number(token.expires_in))?Date.now()+Number(token.expires_in)*1000:null,profile});
      return await status();
    }finally{controller.abort();await loopback.close();active=null;onChanged(await status())}
  }
  async function cancelLogin(){if(active){active.loopback.cancel();active.controller.abort()}return status()}
  async function logout(){if(active)await cancelLogin();await secretStore.remove(SESSION_KEY);const next=await status();onChanged(next);return next}
  async function updateName(name){const saved=await record();if(!saved?.accessToken)throw Error('Sign in first.');saved.profile={...(saved.profile||{}),displayName:String(name||'').trim().slice(0,300)||null};await secretStore.set(SESSION_KEY,saved);const next=await status();onChanged(next);return next}
  async function getAvatar(){
    const saved=await record(),url=String(saved?.profile?.avatarUrl||'');if(!url)return null;
    try{const response=await fetchImpl(url,{headers:{authorization:saved.accessToken?'Bearer '+saved.accessToken:undefined}});if(!response.ok)return null;const bytes=Buffer.from(await response.arrayBuffer());if(bytes.length>5*1024*1024)return null;return'data:'+(response.headers.get('content-type')||'image/png')+';base64,'+bytes.toString('base64')}catch{return null}
  }
  return{configured,status,login,cancelLogin,logout,updateName,getAvatar};
}
module.exports={createAccountSession,decodeJwtPayload,normalizeProfile};

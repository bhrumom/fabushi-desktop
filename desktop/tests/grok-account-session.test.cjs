'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {createAccountSession}=require('../electron/grok-account-session.cjs');

function memorySecretStore(){
  const rows=new Map();
  return{
    async get(key){return rows.get(String(key))??null},
    async set(key,value){rows.set(String(key),value)},
    async remove(key){rows.delete(String(key))}
  };
}
function jwt(payload){
  const encode=value=>Buffer.from(JSON.stringify(value)).toString('base64url');
  return encode({alg:'none',typ:'JWT'})+'.'+encode(payload)+'.';
}

test('account session reports explicit unavailable state without a configured OAuth provider',async()=>{
  const session=createAccountSession({env:{},secretStore:memorySecretStore(),openExternal:async()=>{}});
  assert.deepEqual(await session.status(),{kind:'logged-out',available:false,reason:'Account OAuth provider is not configured.'});
  await assert.rejects(()=>session.login(),/not configured/);
});

test('account session completes PKCE loopback login and returns the settled logged-in state',async()=>{
  const changes=[];
  const store=memorySecretStore();
  const env={
    FABUSHI_ACCOUNT_AUTHORIZATION_URL:'https://accounts.example.test/authorize',
    FABUSHI_ACCOUNT_TOKEN_URL:'https://accounts.example.test/token',
    FABUSHI_ACCOUNT_CLIENT_ID:'fabushi-desktop',
    FABUSHI_ACCOUNT_SCOPES:'openid profile email'
  };
  const session=createAccountSession({
    env,secretStore:store,
    onChanged:status=>changes.push(status.kind),
    openExternal:async value=>{
      const authorization=new URL(value);
      assert.equal(authorization.searchParams.get('code_challenge_method'),'S256');
      assert.equal(authorization.searchParams.get('client_id'),'fabushi-desktop');
      const callback=new URL(authorization.searchParams.get('redirect_uri'));
      callback.searchParams.set('code','fixture-code');
      callback.searchParams.set('state',authorization.searchParams.get('state'));
      const response=await fetch(callback);
      assert.equal(response.ok,true);
    },
    fetchImpl:async(url,options)=>{
      assert.equal(String(url),env.FABUSHI_ACCOUNT_TOKEN_URL);
      assert.equal(options.method,'POST');
      const params=new URLSearchParams(String(options.body));
      assert.equal(params.get('code'),'fixture-code');
      assert.ok(params.get('code_verifier'));
      return new Response(JSON.stringify({
        access_token:'fixture-access-token',
        token_type:'Bearer',
        expires_in:3600,
        id_token:jwt({sub:'user-1',email:'person@example.test',name:'Example Person'})
      }),{status:200,headers:{'content-type':'application/json'}});
    }
  });
  assert.deepEqual(await session.status(),{kind:'logged-out',available:true,reason:null});
  const loggedIn=await session.login();
  assert.equal(loggedIn.kind,'logged-in');
  assert.equal(loggedIn.authId,'user-1');
  assert.equal(loggedIn.email,'person@example.test');
  assert.equal(loggedIn.displayName,'Example Person');
  assert.deepEqual(changes,['logging-in','logged-in']);
  const renamed=await session.updateName('Renamed Person');
  assert.equal(renamed.displayName,'Renamed Person');
  const loggedOut=await session.logout();
  assert.equal(loggedOut.kind,'logged-out');
  assert.deepEqual(changes,['logging-in','logged-in','logged-in','logged-out']);
});

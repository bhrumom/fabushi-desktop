'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const http=require('node:http');
const {createMcpOAuthManager}=require('../electron/grok-mcp-oauth.cjs');

function mapStore(){
  const values=new Map();
  return{
    async get(key){return values.get(String(key))||null},
    async set(key,value){values.set(String(key),value)},
    async remove(key){values.delete(String(key))},
    async move(from,to){const value=values.get(String(from));values.delete(String(from));if(value!=null)values.set(String(to),value)}
  };
}

test('HTTP MCP OAuth uses PKCE loopback, stores token outside renderer state, and supplies Authorization',async t=>{
  const seen={register:0,token:0,authorizedMcp:0,codeChallenge:null,resource:null};
  let base='';
  const server=http.createServer((req,res)=>{
    let raw='';req.setEncoding('utf8');req.on('data',chunk=>{raw+=chunk});req.on('end',()=>{
      const url=new URL(req.url||'/',base);
      if(req.method==='POST'&&url.pathname==='/mcp'){
        if(req.headers.authorization!=='Bearer fixture-token'){
          res.statusCode=401;
          res.setHeader('www-authenticate',`Bearer resource_metadata="${base}/resource-meta"`);
          res.end('auth required');return;
        }
        seen.authorizedMcp++;
        const message=JSON.parse(raw||'{}');
        res.setHeader('content-type','application/json');
        if(message.method==='initialize'){
          res.end(JSON.stringify({jsonrpc:'2.0',id:message.id,result:{protocolVersion:'2025-06-18',capabilities:{},serverInfo:{name:'fixture',version:'1'}}}));return;
        }
        res.end(JSON.stringify({jsonrpc:'2.0',id:message.id,result:{}}));return;
      }
      if(req.method==='GET'&&url.pathname==='/resource-meta'){
        res.setHeader('content-type','application/json');
        res.end(JSON.stringify({resource:base+'/mcp',authorization_servers:[base+'/issuer'],scopes_supported:['tools']}));return;
      }
      if(req.method==='GET'&&url.pathname==='/.well-known/oauth-authorization-server/issuer'){
        res.setHeader('content-type','application/json');
        res.end(JSON.stringify({
          issuer:base+'/issuer',
          authorization_endpoint:base+'/authorize',
          token_endpoint:base+'/token',
          registration_endpoint:base+'/register',
          scopes_supported:['tools']
        }));return;
      }
      if(req.method==='POST'&&url.pathname==='/register'){
        seen.register++;
        res.setHeader('content-type','application/json');
        res.end(JSON.stringify({client_id:'fixture-client'}));return;
      }
      if(req.method==='POST'&&url.pathname==='/token'){
        seen.token++;
        const form=new URLSearchParams(raw);
        assert.equal(form.get('grant_type'),'authorization_code');
        assert.equal(form.get('client_id'),'fixture-client');
        assert.equal(form.get('code'),'fixture-code');
        assert.ok(form.get('code_verifier'));
        assert.equal(form.get('resource'),base+'/mcp');
        res.setHeader('content-type','application/json');
        res.end(JSON.stringify({access_token:'fixture-token',token_type:'Bearer',expires_in:3600,scope:'tools'}));return;
      }
      res.statusCode=404;res.end('not found');
    });
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  t.after(()=>new Promise(resolve=>server.close(resolve)));
  const address=server.address();assert.equal(typeof address,'object');
  base='http://127.0.0.1:'+address.port;

  const tokenStore=mapStore();
  const fixtureServer={id:'remote',name:'Remote',transport:'http',url:base+'/mcp',accountKey:'default'};
  const oauth=createMcpOAuthManager({
    getServer:async id=>id==='remote'?fixtureServer:null,
    tokenStore,
    openExternal:async authorizationUrl=>{
      const auth=new URL(authorizationUrl);
      assert.equal(auth.pathname,'/authorize');
      assert.equal(auth.searchParams.get('client_id'),'fixture-client');
      assert.equal(auth.searchParams.get('code_challenge_method'),'S256');
      seen.codeChallenge=auth.searchParams.get('code_challenge');
      seen.resource=auth.searchParams.get('resource');
      const redirect=new URL(auth.searchParams.get('redirect_uri'));
      redirect.searchParams.set('code','fixture-code');
      redirect.searchParams.set('state',auth.searchParams.get('state'));
      const response=await fetch(redirect);
      assert.equal(response.status,200);
    }
  });

  const connected=await oauth.connect('remote','default');
  assert.equal(connected.connected,true);
  assert.equal(connected.accountKey,'default');
  assert.equal(connected.scope,'tools');
  assert.equal(seen.register,1);
  assert.equal(seen.token,1);
  assert.ok(seen.codeChallenge);
  assert.equal(seen.resource,base+'/mcp');
  assert.equal(await oauth.authorizationHeader(fixtureServer),'Bearer fixture-token');

  const response=await fetch(base+'/mcp',{method:'POST',headers:{authorization:await oauth.authorizationHeader(fixtureServer),'content-type':'application/json'},body:JSON.stringify({jsonrpc:'2.0',id:1,method:'initialize',params:{}})});
  assert.equal(response.status,200);
  assert.equal(seen.authorizedMcp,1);

  const disconnected=await oauth.disconnect('remote','default');
  assert.equal(disconnected.connected,false);
  assert.equal(await oauth.authorizationHeader(fixtureServer),null);
});

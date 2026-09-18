'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const http=require('node:http');
const {createPluginMarketplace}=require('../electron/grok-plugin-marketplace.cjs');
const {createCoordinatorRuntime}=require('../electron/grok-agent-coordinator.cjs');

function windows(){
  return class BrowserWindow{
    static getAllWindows(){return[]}
  };
}
async function fixture(){
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-marketplace-'));
  return{
    root,
    app:{getPath(name){if(name!=='userData')throw Error('unexpected path '+name);return root}},
    BrowserWindow:windows(),
    shell:{openExternal:async()=>{}}
  };
}

test('marketplace provider lists metadata and returns executable install payload from a real HTTP endpoint',async t=>{
  let base='';
  const seen=[];
  const server=http.createServer((req,res)=>{
    let raw='';req.setEncoding('utf8');req.on('data',chunk=>{raw+=chunk});req.on('end',()=>{
      const url=new URL(req.url||'/',base);seen.push(req.method+' '+url.pathname);
      res.setHeader('content-type','application/json');
      if(req.method==='GET'&&url.pathname==='/plugins'){
        res.end(JSON.stringify({plugins:[{
          id:'fixture',displayName:'Fixture tools',description:'Real provider fixture',category:'Developer',
          connectors:[{name:'fixture-mcp',description:'Echo connector'}],
          skills:[{name:'Fixture skill',description:'Fixture private skill'}],
          fields:[{key:'workspace',label:'Workspace',isRequired:true,isSecret:false}]
        }]}));return;
      }
      if(req.method==='POST'&&url.pathname==='/plugins/fixture/install'){
        const body=JSON.parse(raw||'{}');assert.equal(body.values.workspace,'demo');
        res.end(JSON.stringify({
          servers:[{name:'Fixture MCP',transport:'stdio',command:'/usr/bin/printf',args:['fixture']}],
          skills:[{name:'Fixture skill',description:'Fixture private skill',body:'# Fixture skill\nUse the fixture connector.'}]
        }));return;
      }
      res.statusCode=404;res.end('{}');
    });
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  t.after(()=>new Promise(resolve=>server.close(resolve)));
  const address=server.address();assert.equal(typeof address,'object');
  base='http://127.0.0.1:'+address.port;
  const provider=createPluginMarketplace({env:{FABUSHI_PLUGIN_MARKETPLACE_URL:base}});
  const listing=await provider.list();
  assert.equal(listing.available,true);
  assert.equal(listing.plugins[0].connectors[0].name,'fixture-mcp');
  const install=await provider.install('fixture',{workspace:'demo'});
  assert.equal(install.servers[0].command,'/usr/bin/printf');
  assert.equal(install.skills[0].body.includes('Fixture skill'),true);
  assert.deepEqual(seen,['GET /plugins','POST /plugins/fixture/install']);
});

test('coordinator marketplace install materializes real MCP server and private SKILL.md then uninstalls both',async t=>{
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const provider={
    providerUrl:'https://marketplace.example/',
    async list(){return{available:true,reason:null,includesPrivateMarketplaces:false,plugins:[{
      id:'fixture',name:'fixture',displayName:'Fixture',description:'Executable fixture',category:'Developer',homepage:null,iconUrl:null,
      connectors:[{name:'fixture-mcp',description:'connector'}],skills:[{name:'Fixture skill',description:'skill'}],fields:[],publisher:null,marketplace:null
    }]}},
    async install(){return{
      servers:[{name:'Fixture MCP',transport:'stdio',command:'/usr/bin/printf',args:['fixture']}],
      skills:[{name:'Fixture skill',description:'Private skill',body:'# Fixture\nUse the connector.',isEnabledForAgent:true,disableModelInvocation:false}]
    }}
  };
  const runtime=createCoordinatorRuntime({...f,pluginMarketplace:provider});
  let catalog=await runtime.installMarketplacePlugin({entryId:'fixture',values:{}});
  assert.equal(catalog.plugins[0].installed,true);
  const installed=await runtime.listPlugins();
  const mcp=installed.find(row=>row.kind==='mcp'&&row.name==='Fixture MCP');
  assert.ok(mcp);
  const skills=await runtime.listWorkflows();
  assert.equal(skills.some(row=>row.name==='Fixture skill'&&row.filePath.endsWith('SKILL.md')),true);
  const state=JSON.parse(await fs.readFile(path.join(f.root,'grok-agent-runtime.json'),'utf8'));
  assert.equal(Array.isArray(state.marketplaceInstalls.fixture.serverIds),true);
  assert.equal(state.marketplaceInstalls.fixture.serverIds.length,1);

  catalog=await runtime.uninstallMarketplacePlugin({entryId:'fixture'});
  assert.equal(catalog.plugins[0].installed,false);
  assert.equal((await runtime.listPlugins()).some(row=>row.kind==='mcp'&&row.name==='Fixture MCP'),false);
  assert.equal((await runtime.listWorkflows()).some(row=>row.name==='Fixture skill'),false);
});

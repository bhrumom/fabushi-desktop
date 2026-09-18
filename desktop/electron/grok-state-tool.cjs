'use strict';

const definition={type:'function',function:{name:'update_state',description:'Change your own durable state: memory, routines, private workflows, profile, or settings. Use this instead of editing Fabushi state files directly.',parameters:{type:'object',properties:{
  target:{type:'string',enum:['memory','routine','workflow','profile','settings']},
  action:{type:'string'},
  fact:{type:'string'},tier:{type:'string',enum:['profile','log','note']},scope:{type:'string',enum:['agent','user','project']},project:{type:'string'},
  id:{type:'string'},name:{type:'string'},prompt:{type:'string'},schedule:{type:'string'},enabled:{type:'boolean'},description:{type:'string'},body:{type:'string'},
  hidden_from_sidebar:{type:'boolean'},notify_on_updates:{type:'boolean'}
},required:['target','action'],additionalProperties:false}}};
async function executeStateTool(args,{updateState}){
  const target=String(args?.target||''),action=String(args?.action||'');
  const routes=new Set(['memory.write','memory.forget','routine.create','routine.update','routine.pause','routine.resume','routine.delete','workflow.write','workflow.delete','profile.set','settings.set']);
  if(!routes.has(target+'.'+action))throw Error('Unsupported state route: '+target+'.'+action);
  const outcome=await updateState(args);
  if(!outcome||outcome.ok===false)return{text:'Not saved — '+String(outcome?.reason||'state update failed')};
  return{text:String(outcome.detail||'Saved.')};
}
module.exports={definition,executeStateTool};

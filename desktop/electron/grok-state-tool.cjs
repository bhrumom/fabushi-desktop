'use strict';

const TARGETS=['memory','routine','workflow','profile','settings','channel','project','avatar'];
const ACTIONS=['write','forget','create','update','pause','resume','delete','set','disconnect','join','leave','clear'];
const definition={type:'function',function:{name:'update_state',description:'Change your own durable state: memory, routines, private workflows, profile, settings, channels, projects, or avatar. Use this instead of editing Fabushi state files directly.',parameters:{type:'object',properties:{
  target:{type:'string',enum:TARGETS},
  action:{type:'string',enum:ACTIONS},
  fact:{type:'string'},tier:{type:'string',enum:['profile','log','note']},scope:{type:'string',enum:['agent','user','project']},project:{type:'string'},
  id:{type:'string'},name:{type:'string'},prompt:{type:'string'},schedule:{type:'string'},
  trigger:{oneOf:[{type:'object',additionalProperties:true},{type:'array',minItems:1,items:{type:'object',additionalProperties:true}}]},
  enabled:{type:'boolean'},description:{type:'string'},body:{type:'string'},
  hidden_from_sidebar:{type:'boolean'},notify_on_updates:{type:'boolean'},platform:{type:'string'},path:{type:'string'}
},required:['target','action'],additionalProperties:false}}};

const routes=new Set([
  'memory.write','memory.forget',
  'routine.create','routine.update','routine.pause','routine.resume','routine.delete',
  'workflow.write','workflow.delete','profile.set','settings.set','channel.disconnect',
  'project.create','project.join','project.leave','avatar.set','avatar.clear'
]);
async function executeStateTool(args,{updateState}){
  const target=String(args?.target||''),action=String(args?.action||'');
  if(!routes.has(target+'.'+action))throw Error('Unsupported state route: '+target+'.'+action);
  if(args?.schedule!=null&&args?.trigger!=null)throw Error("Pass either schedule or trigger for a routine, never both.");
  const outcome=await updateState(args);
  if(!outcome||outcome.ok===false)return{text:'Not saved — '+String(outcome?.reason||'state update failed')};
  return{text:String(outcome.detail||'Saved.')};
}
module.exports={definition,executeStateTool,TARGETS,ACTIONS,routes};

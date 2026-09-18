'use strict';

const EVERY_PATTERN=/^@every\s+(\d+)\s*(s|m|h|d)$/i;
const UNIT_MS={s:1000,m:60000,h:3600000,d:86400000};
const ALIASES={'@hourly':'0 * * * *','@daily':'0 0 * * *','@midnight':'0 0 * * *','@weekly':'0 0 * * 0','@monthly':'0 0 1 * *','@yearly':'0 0 1 1 *','@annually':'0 0 1 1 *'};
const MINUTE_MS=60000,MAX_SEARCH_MINUTES=366*24*60;
const weekday={Sun:0,Mon:1,Tue:2,Wed:3,Thu:4,Fri:5,Sat:6};
const formatters=new Map();

function normalizeSchedule(raw){return String(raw||'').trim().replace(/\s+/g,' ')}
function splitScheduleTimeZone(schedule){
  const normalized=normalizeSchedule(schedule),match=/^(?:CRON_TZ|TZ)=(\S+)\s+/.exec(normalized);
  return match?{schedule:normalized.slice(match[0].length),timeZone:match[1]}:{schedule:normalized};
}
function parseCronField(field,min,max){
  const values=new Set();
  for(const part of field.split(',')){
    const split=part.split('/');if(split.length>2)return null;
    const rangePart=split[0]||'',step=split.length===2?Number(split[1]):1;
    if(!Number.isInteger(step)||step<=0)return null;
    let start,end;
    if(rangePart==='*'||rangePart===''){start=min;end=max}
    else if(rangePart.includes('-')){const pieces=rangePart.split('-');start=Number(pieces[0]);end=Number(pieces[1])}
    else{start=Number(rangePart);end=split.length===2?max:start}
    if(!Number.isInteger(start)||!Number.isInteger(end)||start<min||end>max||start>end)return null;
    for(let value=start;value<=end;value+=step)values.add(value);
  }
  return values.size?values:null;
}
function getZonedFormatter(timeZone){
  if(formatters.has(timeZone))return formatters.get(timeZone);
  let formatter=null;
  try{formatter=new Intl.DateTimeFormat('en-US',{timeZone,hourCycle:'h23',year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',weekday:'short'})}catch{}
  formatters.set(timeZone,formatter);return formatter;
}
function compileCronMatcher(schedule){
  const split=splitScheduleTimeZone(schedule);
  const fields=(ALIASES[split.schedule.toLowerCase()]||split.schedule).split(' ');
  if(fields.length!==5)return null;
  const [mi='',hr='',dom='',mo='',dow='']=fields;
  const minute=parseCronField(mi,0,59),hour=parseCronField(hr,0,23),dayOfMonth=parseCronField(dom,1,31),month=parseCronField(mo,1,12),rawDow=parseCronField(dow,0,7);
  if(!minute||!hour||!dayOfMonth||!month||!rawDow)return null;
  if(split.timeZone&&getZonedFormatter(split.timeZone)==null)return null;
  return{minute,hour,dayOfMonth,month,dayOfWeek:new Set([...rawDow].map(d=>d===7?0:d)),isDayOfMonthRestricted:dom!=='*',isDayOfWeekRestricted:dow!=='*',...(split.timeZone?{timeZone:split.timeZone}:{})};
}
function localWall(date){return{year:date.getFullYear(),minute:date.getMinutes(),hour:date.getHours(),month:date.getMonth()+1,dayOfMonth:date.getDate(),dayOfWeek:date.getDay()}}
function zonedWall(date,formatter){
  const parts={};for(const part of formatter.formatToParts(date))if(part.type!=='literal')parts[part.type]=part.value;
  return{year:Number(parts.year),minute:Number(parts.minute),hour:Number(parts.hour)%24,month:Number(parts.month),dayOfMonth:Number(parts.day),dayOfWeek:weekday[parts.weekday]??0};
}
function cronDayMatches(matcher,wall){
  if(!matcher.month.has(wall.month))return false;
  const dom=matcher.dayOfMonth.has(wall.dayOfMonth),dow=matcher.dayOfWeek.has(wall.dayOfWeek);
  return matcher.isDayOfMonthRestricted&&matcher.isDayOfWeekRestricted?dom||dow:(matcher.isDayOfMonthRestricted?dom:true)&&(matcher.isDayOfWeekRestricted?dow:true);
}
function cronMatchesWallClock(matcher,wall){return matcher.minute.has(wall.minute)&&matcher.hour.has(wall.hour)&&cronDayMatches(matcher,wall)}
function nextCronRun(matcher,afterMs,wallClockOf){
  let cursor=Math.floor(afterMs/MINUTE_MS)*MINUTE_MS+MINUTE_MS;
  const deadline=cursor+MAX_SEARCH_MINUTES*MINUTE_MS;
  while(cursor<deadline){
    const wall=wallClockOf(new Date(cursor));
    if(cronMatchesWallClock(matcher,wall))return cursor;
    if(cronDayMatches(matcher,wall)){cursor+=MINUTE_MS;continue}
    const toMidnight=(23-wall.hour)*60+(60-wall.minute),candidate=cursor+toMidnight*MINUTE_MS,nextWall=wallClockOf(new Date(candidate));
    if(nextWall.year===wall.year&&nextWall.month===wall.month&&nextWall.dayOfMonth===wall.dayOfMonth){cursor=candidate;continue}
    const overshoot=Math.min(nextWall.hour*60+nextWall.minute,toMidnight-1);cursor=candidate-overshoot*MINUTE_MS;
  }
  return null;
}
function parseEveryIntervalMs(schedule){
  const match=EVERY_PATTERN.exec(normalizeSchedule(schedule));if(!match)return null;
  const amount=Number(match[1]),unitMs=UNIT_MS[String(match[2]||'').toLowerCase()];
  return Number.isFinite(amount)&&amount>0&&unitMs?amount*unitMs:null;
}
function isValidSchedule(schedule){
  const normalized=normalizeSchedule(schedule);if(parseEveryIntervalMs(normalized)!=null)return true;
  return compileCronMatcher(normalized)!=null;
}
function computeNextRunAt(schedule,afterMs,timeZone){
  const normalized=normalizeSchedule(schedule),interval=parseEveryIntervalMs(normalized);
  if(interval!=null)return afterMs+interval;
  const matcher=compileCronMatcher(normalized);if(!matcher)return null;
  const zone=matcher.timeZone||timeZone,formatter=zone?getZonedFormatter(zone):null;
  return nextCronRun(matcher,afterMs,formatter?date=>zonedWall(date,formatter):localWall);
}
function describeSchedule(schedule){
  const normalized=normalizeSchedule(schedule),interval=parseEveryIntervalMs(normalized);
  if(interval!=null){
    const match=EVERY_PATTERN.exec(normalized),amount=match?.[1]||'',unit={s:'second',m:'minute',h:'hour',d:'day'}[String(match?.[2]||'').toLowerCase()]||'';
    return amount==='1'?'Every '+unit:'Every '+amount+' '+unit+'s';
  }
  const aliases=Object.entries(ALIASES).find(([,value])=>value===normalized);
  if(aliases)return aliases[0].slice(1).replace(/^./,x=>x.toUpperCase());
  return normalized;
}

module.exports={EVERY_PATTERN,normalizeSchedule,compileCronMatcher,parseEveryIntervalMs,isValidSchedule,computeNextRunAt,describeSchedule};

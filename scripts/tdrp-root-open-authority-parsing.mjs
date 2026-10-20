export function logicalLines(lines){
  const out=[];
  for(let i=0;i<lines.length;i++){
    const start=i+1;
    let text=lines[i],end=start;
    while(/\\\s*$/.test(text)&&i+1<lines.length){text=text.replace(/\\\s*$/,' ')+lines[++i].trim();end=i+1;}
    out.push({start,end,text});
  }
  return out;
}
export function cleanToken(s){return s.replace(/^['"]|['",;)]$/g,'').replace(/\\$/,'');}
function versionsForMajor(source,major){
  return [...new Set([...String(source).matchAll(/os\.environ\[['"]QT['"]\]\s*=\s*['"]([^'"]+)['"]/g)].map(m=>m[1]).filter(v=>v.startsWith(String(major)+'.')))].sort();
}
function oneVersion(source,major,context){
  const versions=versionsForMajor(source,major);
  if(versions.length!==1) throw new Error('unable to resolve '+context+': expected exactly one Qt '+major+' version, found '+versions.join(', '));
  return versions[0];
}
export function resolveQtCloneRef(logicalText,qtVersionSource){
  const line=String(logicalText);
  if(!/\bgit clone\b/.test(line)||! /https:\/\/github\.com\/qt\/qt5\.git(?:\s|$)/.test(line)) return null;
  if(/-b\s+v\$QT-lts-lgpl(?:\s|$)/.test(line)){
    const version=oneVersion(qtVersionSource,5,'legacy Qt clone ref');
    return {ref:'v'+version+'-lts-lgpl',authorityNote:'resolved from Telegram/build/qt_version.py Qt 5 selection for qt<6 legacy branch'};
  }
  if(/-b\s+"""?\s*\+\s*branch\s*\+\s*"""?/.test(line)){
    const version=oneVersion(qtVersionSource,6,'computed Qt 6 clone ref');
    return {ref:'v'+version+(version.startsWith('6.2.')?'-lts-lgpl':''),authorityNote:'resolved from Telegram/build/qt_version.py and branch expression; Qt 6.2 uses the LTS suffix'};
  }
  return null;
}

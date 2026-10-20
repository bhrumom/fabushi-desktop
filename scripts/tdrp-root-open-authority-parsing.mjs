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
  return [...new Set([...String(source).matchAll(/os\\.environ\\[['"]QT['"]\\]\\s*=\\s*['"]([^'"]+)['"]/g)].map(m=>m[1]).filter(v=>v.startsWith(String(major)+'.')))].sort();
}
function oneVersion(source,major,context){
  const versions=versionsForMajor(source,major);
  if(versions.length!==1) throw new Error('unable to resolve '+context+': expected exactly one Qt '+major+' version, found '+versions.join(', '));
  return versions[0];
}
function resolveComputedQt6Branch(prepareSource,version){
  const expressions=[...String(prepareSource).matchAll(/\\bbranch\\s*=\\s*(['"])([^'"]*)\\1\\s*\\+\\s*\\(\\s*(['"])([^'"]*)\\3\\s+if\\s+qt\\.startswith\\(\\s*(['"])([^'"]*)\\5\\s*\\)\\s+else\\s+(['"])([^'"]*)\\7\\s*\\)/g)];
  if(expressions.length!==1) throw new Error('unable to resolve computed Qt 6 clone ref: expected exactly one source branch expression, found '+expressions.length);
  const expression=expressions[0];
  const prefix=expression[2], conditionalSuffix=expression[4], conditionPrefix=expression[6], fallbackSuffix=expression[8];
  if((prefix.match(/\\$QT/g)||[]).length!==1) throw new Error('unsupported Qt 6 branch prefix in Telegram/build/prepare/prepare.py');
  return prefix.replace('$QT',version)+(version.startsWith(conditionPrefix)?conditionalSuffix:fallbackSuffix);
}
export function resolveQtCloneRef(logicalText,qtVersionSource,prepareSource){
  const line=String(logicalText);
  const legacy=line.match(/^\\s*git clone\\s+-b\\s+(v\\$QT[^\\s]*)\\s+https:\\/\\/github\\.com\\/qt\\/qt5\\.git(?:\\s|$)/);
  if(legacy){
    const token=legacy[1];
    if((token.match(/\\$QT/g)||[]).length!==1) throw new Error('unsupported dynamic Qt 5 clone ref in Telegram/build/prepare/prepare.py');
    const version=oneVersion(qtVersionSource,5,'legacy Qt clone ref');
    return {ref:token.replace('$QT',version),authorityNote:'resolved from the actual Qt 5 clone ref expression and Telegram/build/qt_version.py'};
  }
  const computed=/^\\s*git clone\\s+-b\\s+"""?\\s*\\+\\s*branch\\s*\\+\\s*"""?\\s+https:\\/\\/github\\.com\\/qt\\/qt5\\.git(?:\\s|$)/.test(line);
  if(computed){
    const version=oneVersion(qtVersionSource,6,'computed Qt 6 clone ref');
    return {ref:resolveComputedQt6Branch(prepareSource,version),authorityNote:'resolved from the actual computed branch expression in Telegram/build/prepare/prepare.py and Qt 6 selection in Telegram/build/qt_version.py'};
  }
  return null;
}

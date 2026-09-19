'use strict';

const ASCII_MARKERS=Object.freeze([{offset:4,text:'ftyp'},{offset:0,text:'OggS'},{offset:0,text:'FLV'},{offset:8,text:'AVI '}]);
const BYTE_MARKERS=Object.freeze([[26,69,223,163],[48,38,178,117],[0,0,1,186],[0,0,1,179]]);
function hasAsciiAt(bytes,offset,value){if(!(bytes instanceof Uint8Array)||bytes.byteLength<offset+value.length)return false;for(let i=0;i<value.length;i+=1)if(bytes[offset+i]!==value.charCodeAt(i))return false;return true}
function bytesLookLikeVideoContainer(bytes){return bytes instanceof Uint8Array&&(ASCII_MARKERS.some(marker=>hasAsciiAt(bytes,marker.offset,marker.text))||BYTE_MARKERS.some(marker=>bytes.byteLength>=marker.length&&marker.every((byte,index)=>bytes[index]===byte)))}
module.exports={ASCII_MARKERS,BYTE_MARKERS,hasAsciiAt,bytesLookLikeVideoContainer};

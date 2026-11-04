import {test} from 'node:test';
import assert from 'node:assert/strict';
import {validateSourceReviews as validate} from './tdrp-source-review.mjs';
const source = {repository:'owner/repo',commit:'a'.repeat(40),path:'src/message.cpp',object:'b'.repeat(40)};
const review = {...source,category:'product',risk:'high',review_depth:'deep',status:'complete',rationale:'state and recovery reviewed',evidence:['dossier#message'],responsibility_ids:['message']};
const rows = [{responsibility_id:'message'}];
test('product deep review and exact identity coverage',()=>assert.equal(validate([source],[review],rows).reviewed,1));
for (const category of ['generated','dependency','resource','documentation','tooling']) {
 test(category+' accepts evidenced non-deep review',()=>assert.equal(validate([source],[{...review,category,risk:'low',review_depth:['generated','dependency'].includes(category)?'contract':'disposition',responsibility_ids:[]}],rows).reviewed,1));
}
for (const [name,patch] of Object.entries({stale:{object:'c'.repeat(40)},pending:{status:'pending'},evidence:{evidence:[]},productShallow:{review_depth:'contract'},highRiskShallow:{category:'dependency',review_depth:'contract'},unmapped:{responsibility_ids:['absent']},missingProductMapping:{responsibility_ids:[]},unknown:{category:'ignored'},dependencyShallow:{category:'dependency',risk:'low',review_depth:'disposition'}})) {
 test('reject '+name,()=>assert.throws(()=>validate([source],[{...review,...patch}],rows)));
}
test('reject missing and duplicate identities',()=>{assert.throws(()=>validate([source],[],rows));assert.throws(()=>validate([source],[review,review],rows));assert.throws(()=>validate([source],undefined,rows));});

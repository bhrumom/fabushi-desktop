import assert from "node:assert/strict";
import test from "node:test";
import { preferredIndicesWithToggle } from "./network-proxy.js";
test("proxy preferred-index editor preserves source order and uniqueness",()=>{assert.deepEqual(preferredIndicesWithToggle([2,0],1,true),[2,0,1]);assert.deepEqual(preferredIndicesWithToggle([2,0,1],0,false),[2,1]);assert.deepEqual(preferredIndicesWithToggle([2,0],2,true),[0,2]);});

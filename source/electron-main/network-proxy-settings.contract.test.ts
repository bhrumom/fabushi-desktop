import assert from "node:assert/strict";
import test from "node:test";
import { MAIN_METHOD_TABLE } from "../shared/rpc/main.js";
import { COORDINATOR_MAIN_METHOD_TABLE } from "../shared/rpc/coordinator-main.js";
test("network proxy Settings stays source-neutral across renderer/Main/Host allowlists",()=>{assert.deepEqual(MAIN_METHOD_TABLE.getNetworkProxySettings,{args:"none"});assert.deepEqual(MAIN_METHOD_TABLE.mutateNetworkProxySettings,{args:"object"});assert.deepEqual(COORDINATOR_MAIN_METHOD_TABLE["settings.snapshot"],{args:"none"});assert.deepEqual(COORDINATOR_MAIN_METHOD_TABLE["settings.mutateNetworkProxy"],{args:"object"});for(const method of ["getNetworkProxySettings","mutateNetworkProxySettings","settings.snapshot","settings.mutateNetworkProxy"])assert.equal(method.toLowerCase().includes("telegram"),false);});
test("selected proxy carrier is not exposed as a Main RPC mutation",()=>{assert.equal(Object.keys(MAIN_METHOD_TABLE).some((method)=>/connect.*proxy|apply.*proxy|proxy.*carrier/i.test(method)),false);});

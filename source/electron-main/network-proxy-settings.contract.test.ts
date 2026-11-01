import assert from "node:assert/strict";
import test from "node:test";
import { MAIN_METHOD_TABLE } from "../shared/rpc/main.js";
import { COORDINATOR_MAIN_METHOD_TABLE } from "../shared/rpc/coordinator-main.js";
import { parseWebProxyBootstrapUrl } from "./main-edge.js";
test("network proxy Settings stays source-neutral across renderer/Main/Host allowlists",()=>{assert.deepEqual(MAIN_METHOD_TABLE.getNetworkProxySettings,{args:"none"});assert.deepEqual(MAIN_METHOD_TABLE.mutateNetworkProxySettings,{args:"object"});assert.deepEqual(COORDINATOR_MAIN_METHOD_TABLE["settings.snapshot"],{args:"none"});assert.deepEqual(COORDINATOR_MAIN_METHOD_TABLE["settings.mutateNetworkProxy"],{args:"object"});for(const method of ["getNetworkProxySettings","mutateNetworkProxySettings","settings.snapshot","settings.mutateNetworkProxy"])assert.equal(method.toLowerCase().includes("telegram"),false);});
test("selected proxy carrier is not exposed as a Main RPC mutation",()=>{assert.equal(Object.keys(MAIN_METHOD_TABLE).some((method)=>/connect.*proxy|apply.*proxy|proxy.*carrier/i.test(method)),false);});

test("WebProxy bootstrap launcher accepts only one-shot loopback URLs",()=>{assert.equal(parseWebProxyBootstrapUrl("http://127.0.0.1:49152/#opaque-capability"),"http://127.0.0.1:49152/#opaque-capability");for(const value of ["https://127.0.0.1:49152/#x","http://localhost:49152/#x","http://127.0.0.1:49152/path#x","http://127.0.0.1:49152/?q=1#x","http://127.0.0.1:49152/","http://user@127.0.0.1:49152/#x","https://proxy.example/#x"])assert.equal(parseWebProxyBootstrapUrl(value),null);});

"use strict";
const { performance } = require('node:perf_hooks');
const { callChannel, createRendererEdge, defineEdge, serveMainEdge } = require('./edge-ipc.cjs');

const ITERATIONS = Number(process.env.GBF_EDGE_BENCH_ITERATIONS || 20000);
const MAX_AVERAGE_MICROS = Number(process.env.GBF_EDGE_MAX_AVERAGE_MICROS || 500);
const handlers = new Map();
const ipcMain = { handle: (channel, fn) => handlers.set(channel, fn), removeHandler: (channel) => handlers.delete(channel) };
const edge = defineEdge('bench', { ping: { args: 'none' } });
serveMainEdge(ipcMain, edge, { ping: async () => 1 });
const ipcRenderer = { invoke: (channel, args) => handlers.get(channel)({}, args), on() {}, off() {} };
const client = createRendererEdge(ipcRenderer, edge);

(async () => {
  for (let i = 0; i < 100; i += 1) await client.ping();
  const started = performance.now();
  for (let i = 0; i < ITERATIONS; i += 1) await client.ping();
  const elapsedMs = performance.now() - started;
  const averageMicros = elapsedMs * 1000 / ITERATIONS;
  const result = { iterations: ITERATIONS, elapsedMs: Number(elapsedMs.toFixed(3)), averageMicros: Number(averageMicros.toFixed(3)), budgetMicros: MAX_AVERAGE_MICROS };
  console.log(JSON.stringify(result));
  if (averageMicros > MAX_AVERAGE_MICROS) process.exitCode = 1;
})().catch((error) => { console.error(error); process.exitCode = 1; });

'use strict';

/**
 * Compatibility facade kept at the Electron composition boundary.
 * The agent implementation is the coordinator -> host -> local-exec graph,
 * not a CLI process and not a monolithic renderer-owned loop.
 */
const {createCoordinatorRuntime}=require('./grok-agent-coordinator.cjs');

function createRuntime(deps){
  return createCoordinatorRuntime(deps);
}

module.exports={createRuntime};

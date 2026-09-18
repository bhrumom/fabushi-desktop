import GrokApp from './grok-app';
import { acquireProductionRendererRuntime, mountProductionRenderer, requireProductionRendererMount } from './production/bootstrap';
import { RootErrorBoundary } from './production/root-error-boundary';

acquireProductionRendererRuntime(window);
const mount=requireProductionRendererMount(document.querySelector<HTMLDivElement>('#root'));
mountProductionRenderer(mount,<RootErrorBoundary><GrokApp/></RootErrorBoundary>);

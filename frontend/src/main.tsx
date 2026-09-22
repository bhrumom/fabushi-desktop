import { bootstrapProductionRenderer } from './production/bootstrap';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

bootstrapProductionRenderer(root);

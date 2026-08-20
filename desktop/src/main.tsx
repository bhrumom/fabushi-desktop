import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import HostClient from '../../frontend/apps/web/src/app/host/host-client';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

createRoot(root).render(
  <StrictMode>
    <HostClient />
  </StrictMode>,
);

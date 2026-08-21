import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import DesktopShellV2 from './messaging-shell-v2';

const root = document.querySelector<HTMLDivElement>('#root');
if (!root) {
  throw new Error('Fabushi desktop root element is missing');
}

createRoot(root).render(
  <StrictMode>
    <DesktopShellV2 />
  </StrictMode>,
);

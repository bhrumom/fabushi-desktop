import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import GrokApp from './grok-app';
const root=document.querySelector<HTMLDivElement>('#root');
if(!root) throw new Error('Fabushi desktop root element is missing');
createRoot(root).render(<StrictMode><GrokApp/></StrictMode>);
import React from 'react';
import { createRoot } from 'react-dom/client';
import { PluginsBrowser } from '../../../frontend/src/recovered/features/plugins/overlay/browser';
const items = [
  { kind: 'plugin' as const, id: 'github', displayName: 'GitHub', publisher: 'GitHub', description: 'Work with repositories, issues and pull requests.', installed: false },
  { kind: 'plugin' as const, id: 'drive', displayName: 'Google Drive', publisher: 'Google', description: 'Find documents and connect your shared knowledge.', installed: true },
  { kind: 'plugin' as const, id: 'skill', displayName: 'Research assistant', description: 'Organize sources and prepare a research summary.', installed: false },
];
createRoot(document.getElementById('root')!).render(<div className="sand-plugins-dialog"><PluginsBrowser items={items} catalog={items.map(item => ({id:item.id,connectors:item.id==='skill'?[]:[{}],skills:item.id==='skill'?[{}]:[]}))} onInstall={id => { document.body.dataset.installed = id; }} /></div>);

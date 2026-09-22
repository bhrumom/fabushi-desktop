import React from 'react';
import DesktopApp from '../../../desktop/src/app/DesktopApp';
import CredentialVault from '../../../desktop/src/credential-vault';

/**
 * Canonical production renderer boundary for the Grok-shaped desktop tree.
 *
 * Product-specific surfaces are still being migrated out of desktop/src, but
 * this module owns the renderer root so the migration has a single canonical
 * entrypoint under frontend/** rather than a second parallel shell.
 */
export default function ProductionRenderer() {
  return <>
    <DesktopApp />
    <CredentialVault />
  </>;
}

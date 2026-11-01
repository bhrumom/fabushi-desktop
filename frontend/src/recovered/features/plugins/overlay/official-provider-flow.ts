/** Exact Fabushi provider listings whose native owner supports account OAuth. */
export function usesOfficialProviderOAuth(pluginId: string): boolean {
  return /^fabushi-official-(github|google-(gmail|drive|docs|sheets|slides|calendar|chat|people))$/.test(pluginId);
}
export function needsPluginSetupBeforeAdd(pluginId: string, fields: readonly { isRequired: boolean }[], teamConfigured = false): boolean {
  if (teamConfigured) return false;
  return usesOfficialProviderOAuth(pluginId) ? fields.some(field => field.isRequired) : fields.length > 0;
}

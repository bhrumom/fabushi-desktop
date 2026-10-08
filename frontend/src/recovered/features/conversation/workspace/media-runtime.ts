export async function resolveWithSingleRetry<T>(
  resolver: (source: string) => Promise<T | null>,
  source: string,
): Promise<T | null> {
  let lastError: unknown = null;
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const value = await resolver(source);
      if (value != null) return value;
      lastError = null;
    } catch (error) {
      lastError = error;
    }
  }
  if (lastError != null) throw lastError;
  return null;
}

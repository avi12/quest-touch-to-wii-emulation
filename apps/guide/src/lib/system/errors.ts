/** An error's message for display, whatever was thrown. */
export function describeError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function natcasecmp(
  ...values: [string | null | undefined, string | null | undefined][]
): number {
  for (const [a, b] of values) {
    const result = (a ?? "").localeCompare(b ?? "", undefined, {
      numeric: true,
      sensitivity: "case",
    });
    if (result != 0) return result;
  }
  return 0;
}

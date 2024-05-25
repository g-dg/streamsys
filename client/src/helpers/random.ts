const ALPHANUMERIC =
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

export function randomString(length: number, chars?: string): string {
  const useChars = chars ?? ALPHANUMERIC;
  return Array.from({ length }, () =>
    useChars.charAt(Math.floor(Math.random() * useChars.length))
  ).join("");
}

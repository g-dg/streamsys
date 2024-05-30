const ALPHANUMERIC =
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

export function randomString(length: number, chars?: string): string {
  const useChars = chars ?? ALPHANUMERIC;
  return Array.from({ length }, () =>
    useChars.charAt(Math.floor(Math.random() * useChars.length))
  ).join("");
}

export function uuid(): string {
  if (crypto.randomUUID != undefined) {
    return crypto.randomUUID();
  }

  function hex(bytes: number): string {
    let array = new Uint8Array(bytes);
    crypto.getRandomValues(array);
    return Array.from(array)
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
  }

  return hex(4) + "-" + hex(2) + "-" + hex(2) + "-" + hex(2) + "-" + hex(6);
}

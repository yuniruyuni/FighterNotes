import { isIP } from "node:net";

export function requestClientKey(
  headers: Headers,
  trustCloudflareConnectingIp: boolean,
): string {
  if (!trustCloudflareConnectingIp) return "untrusted-proxy";
  const value = headers.get("cf-connecting-ip")?.trim() ?? "";
  const version = isIP(value);
  if (version === 4) return value;
  if (version === 6) {
    const hostname = new URL(`http://[${value}]/`).hostname;
    return hostname.slice(1, -1).toLowerCase();
  }
  return "unknown";
}

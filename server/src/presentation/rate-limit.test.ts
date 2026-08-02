import { describe, expect, test } from "bun:test";
import { requestClientKey } from "./rate-limit";

describe("sharing client identity", () => {
  test("明示されたCloud Run trust境界内だけでCloudflareのIPを使う", () => {
    const headers = new Headers({
      "CF-Connecting-IP": "203.0.113.10",
      "X-Forwarded-For": "198.51.100.2, 198.51.100.3",
    });
    expect(requestClientKey(headers, true)).toBe("203.0.113.10");
    expect(requestClientKey(headers, false)).toBe("untrusted-proxy");
  });

  test("直接spoofや不正値を識別子として使わずIPv6を正規化する", () => {
    expect(
      requestClientKey(new Headers({ "CF-Connecting-IP": "not-an-ip" }), true),
    ).toBe("unknown");
    expect(
      requestClientKey(
        new Headers({ "CF-Connecting-IP": "2001:0DB8:0:0::1" }),
        true,
      ),
    ).toBe("2001:db8::1");
    expect(requestClientKey(new Headers(), true)).toBe("unknown");
  });
});

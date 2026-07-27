interface WindowEntry {
  count: number;
  resetsAt: number;
}

export interface RateLimitDecision {
  allowed: boolean;
  retryAfterSeconds: number;
}

const WINDOW_MILLISECONDS = 60_000;
const MAX_TRACKED_KEYS = 10_000;

export class FixedWindowRateLimiter {
  private readonly entries = new Map<string, WindowEntry>();
  private nextSweepAt = 0;

  constructor(private readonly limit: number) {}

  consume(key: string, now = Date.now()): RateLimitDecision {
    this.removeExpired(now);
    const current = this.entries.get(key);
    if (current && current.resetsAt > now) {
      if (current.count >= this.limit) {
        return denied(current.resetsAt, now);
      }
      current.count += 1;
      return { allowed: true, retryAfterSeconds: 0 };
    }

    if (this.entries.size >= MAX_TRACKED_KEYS) {
      return { allowed: false, retryAfterSeconds: 60 };
    }
    this.entries.set(key, { count: 1, resetsAt: now + WINDOW_MILLISECONDS });
    return { allowed: true, retryAfterSeconds: 0 };
  }

  private removeExpired(now: number): void {
    if (now < this.nextSweepAt && this.entries.size < MAX_TRACKED_KEYS) return;
    for (const [key, entry] of this.entries) {
      if (entry.resetsAt <= now) this.entries.delete(key);
    }
    this.nextSweepAt = now + WINDOW_MILLISECONDS;
  }
}

export function requestClientKey(headers: Headers): string {
  // Cloud Run is internal-only and receives public traffic through the
  // Cloudflare tunnel. Cloudflare overwrites this header; X-Forwarded-For is
  // deliberately ignored because a client can append spoofed values to it.
  const value = headers.get("cf-connecting-ip") ?? "unknown";
  return value.trim().slice(0, 128) || "unknown";
}

function denied(resetsAt: number, now: number): RateLimitDecision {
  return {
    allowed: false,
    retryAfterSeconds: Math.max(1, Math.ceil((resetsAt - now) / 1000)),
  };
}

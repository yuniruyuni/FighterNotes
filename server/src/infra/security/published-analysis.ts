import { randomBytes } from "node:crypto";
import type {
  DeletePassword,
  DeletePasswordHash,
  ShareId,
} from "../../models/published-analysis";
import {
  type PublishedAnalysisSecurity,
  SecurityCapacityError,
} from "../../usecases/services";

const DEFAULT_CONCURRENCY = 2;
const DEFAULT_QUEUE_LIMIT = 8;
const DEFAULT_WAIT_MILLIS = 250;

interface PublishedAnalysisSecurityOptions {
  concurrency?: number;
  queueLimit?: number;
  waitMillis?: number;
  hashPassword?: (password: DeletePassword) => Promise<DeletePasswordHash>;
  verifyPassword?: (
    password: DeletePassword,
    hash: DeletePasswordHash,
  ) => Promise<boolean>;
}

interface Waiter {
  readonly resolve: (release: () => void) => void;
  readonly reject: (error: SecurityCapacityError) => void;
  readonly timer: ReturnType<typeof setTimeout>;
}

class BoundedConcurrency {
  private active = 0;
  private readonly waiters: Waiter[] = [];

  constructor(
    private readonly concurrency: number,
    private readonly queueLimit: number,
    private readonly waitMillis: number,
  ) {}

  async run<T>(operation: () => Promise<T>): Promise<T> {
    const release = await this.acquire();
    try {
      return await operation();
    } finally {
      release();
    }
  }

  private acquire(): Promise<() => void> {
    if (this.active < this.concurrency) {
      this.active += 1;
      return Promise.resolve(this.releaseOnce());
    }
    if (this.waiters.length >= this.queueLimit) {
      return Promise.reject(new SecurityCapacityError());
    }
    return new Promise((resolve, reject) => {
      const waiter: Waiter = {
        resolve,
        reject,
        timer: setTimeout(() => {
          const index = this.waiters.indexOf(waiter);
          if (index >= 0) this.waiters.splice(index, 1);
          reject(new SecurityCapacityError());
        }, this.waitMillis),
      };
      this.waiters.push(waiter);
    });
  }

  private releaseOnce(): () => void {
    let released = false;
    return () => {
      if (released) return;
      released = true;
      const waiter = this.waiters.shift();
      if (waiter) {
        clearTimeout(waiter.timer);
        waiter.resolve(this.releaseOnce());
        return;
      }
      this.active -= 1;
    };
  }
}

export function createPublishedAnalysisSecurity(
  options: PublishedAnalysisSecurityOptions = {},
): PublishedAnalysisSecurity {
  const concurrency = options.concurrency ?? DEFAULT_CONCURRENCY;
  const queueLimit = options.queueLimit ?? DEFAULT_QUEUE_LIMIT;
  const waitMillis = options.waitMillis ?? DEFAULT_WAIT_MILLIS;
  if (!Number.isInteger(concurrency) || concurrency < 1) {
    throw new Error("Argon2 concurrency must be a positive integer");
  }
  if (!Number.isInteger(queueLimit) || queueLimit < 0) {
    throw new Error("Argon2 queue limit must be a non-negative integer");
  }
  if (!Number.isInteger(waitMillis) || waitMillis < 1) {
    throw new Error("Argon2 wait must be a positive integer");
  }
  const capacity = new BoundedConcurrency(concurrency, queueLimit, waitMillis);
  const hashPassword = options.hashPassword ?? defaultHashPassword;
  const verifyPassword = options.verifyPassword ?? defaultVerifyPassword;

  return {
    generateShareId(): ShareId {
      return randomBytes(16).toString("base64url") as ShareId;
    },
    hashDeletePassword(password) {
      return capacity.run(() => hashPassword(password));
    },
    verifyDeletePassword(password, hash) {
      return capacity.run(() => verifyPassword(password, hash));
    },
  };
}

async function defaultHashPassword(
  password: DeletePassword,
): Promise<DeletePasswordHash> {
  return (await Bun.password.hash(password, {
    algorithm: "argon2id",
    memoryCost: 7_168,
    timeCost: 5,
  })) as DeletePasswordHash;
}

async function defaultVerifyPassword(
  password: DeletePassword,
  hash: DeletePasswordHash,
): Promise<boolean> {
  try {
    return await Bun.password.verify(password, hash, "argon2id");
  } catch {
    return false;
  }
}

export const publishedAnalysisSecurity = createPublishedAnalysisSecurity();

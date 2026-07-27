import { randomBytes } from "node:crypto";
import type {
  DeletePassword,
  DeletePasswordHash,
  ShareId,
} from "../../models/published-analysis";
import type { PublishedAnalysisSecurity } from "../../usecases/services";

export const publishedAnalysisSecurity: PublishedAnalysisSecurity = {
  generateShareId(): ShareId {
    return randomBytes(16).toString("base64url") as ShareId;
  },

  async hashDeletePassword(
    password: DeletePassword,
  ): Promise<DeletePasswordHash> {
    return (await Bun.password.hash(password, {
      algorithm: "argon2id",
      memoryCost: 7_168,
      timeCost: 5,
    })) as DeletePasswordHash;
  },

  async verifyDeletePassword(
    password: DeletePassword,
    hash: DeletePasswordHash,
  ): Promise<boolean> {
    try {
      return await Bun.password.verify(password, hash, "argon2id");
    } catch {
      return false;
    }
  },
};

import type {
  DeletePassword,
  DeletePasswordHash,
  ShareId,
} from "../models/published-analysis";

export interface PublishedAnalysisSecurity {
  generateShareId(): ShareId;
  hashDeletePassword(password: DeletePassword): Promise<DeletePasswordHash>;
  verifyDeletePassword(
    password: DeletePassword,
    hash: DeletePasswordHash,
  ): Promise<boolean>;
}

export interface RuntimeServices {
  publishedAnalysisSecurity: PublishedAnalysisSecurity;
}

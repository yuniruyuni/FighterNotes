import type {
  ManagedShare,
  ManagedShareSnapshot,
} from "../domain/managed-share.js";
import type { PublishedAnalysisCandidate } from "../domain/published-analysis.js";
import type { PublishedAnalysisShare } from "../domain/share.js";

export interface NativeShareData {
  title: string;
  text: string;
  url: string;
}

export interface PublishedAnalysisGateway {
  create(
    candidate: PublishedAnalysisCandidate,
    deletePassword: string,
  ): Promise<PublishedAnalysisShare>;
  delete(
    share: Pick<PublishedAnalysisShare, "id">,
    deletePassword: string,
  ): Promise<void>;
  errorMessage(error: unknown): string;
}

export interface ManagedShareRepository {
  save(share: ManagedShare, now: Date): boolean;
  load(now: Date): ManagedShareSnapshot;
  remove(id: string): boolean;
  subscribe(listener: () => void): () => void;
}

export interface ShareCapabilities {
  copyText(value: string): Promise<void>;
  canShare(): boolean;
  share(data: NativeShareData): Promise<void>;
  confirm(message: string): boolean;
  origin(): string;
  isCancelledShare(error: unknown): boolean;
}

export interface SharingServices {
  gateway: PublishedAnalysisGateway;
  managedShares: ManagedShareRepository;
  capabilities: ShareCapabilities;
  generateDeleteCode(): string;
  now(): Date;
}

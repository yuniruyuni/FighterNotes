import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { formatCharacterId } from "~/modules/analysis/contracts.js";
import type { PublicationSource } from "../domain/publication.js";
import {
  PublishedAnalysisCandidate,
  ShareProjectionError,
} from "../domain/published-analysis.js";
import type { PublishedAnalysisShare } from "../domain/share.js";
import type { SharingServices } from "./ports.js";

export interface CreatedPublication {
  published: PublishedAnalysisShare;
  storedLocally: boolean;
}

export function preparePublication(
  report: AdviceReport,
  context: AnalysisContext,
  services: SharingServices,
): PublicationSource {
  return { report, context, deleteCode: services.generateDeleteCode() };
}

export function renewPublication(
  source: PublicationSource,
  services: SharingServices,
): PublicationSource {
  return { ...source, deleteCode: services.generateDeleteCode() };
}

export async function createPublication(
  source: PublicationSource,
  services: SharingServices,
): Promise<CreatedPublication> {
  const candidate = PublishedAnalysisCandidate.from(
    source.context,
    source.report,
  );
  const published = await services.gateway.create(candidate, source.deleteCode);
  const now = services.now();
  const storedLocally = services.managedShares.save(
    {
      id: published.id,
      deleteCode: source.deleteCode,
      createdAt: now.toISOString(),
      expiresAt: published.expiresAt,
      label: publicationLabel(source.context),
    },
    now,
  );
  return { published, storedLocally };
}

export async function discardPublication(
  published: PublishedAnalysisShare,
  source: PublicationSource,
  services: SharingServices,
): Promise<void> {
  await services.gateway.delete(published, source.deleteCode);
}

export async function deletePublication(
  published: PublishedAnalysisShare,
  source: PublicationSource,
  storedLocally: boolean,
  services: SharingServices,
): Promise<boolean> {
  await services.gateway.delete(published, source.deleteCode);
  return !storedLocally || services.managedShares.remove(published.id);
}

export function publicationLabel(context: AnalysisContext): string {
  const own = context.ownSide === "p1" ? context.p1 : context.p2;
  const opponent = context.ownSide === "p1" ? context.p2 : context.p1;
  return `${formatCharacterId(own.character)} vs ${formatCharacterId(opponent.character)}`;
}

export function publicationErrorMessage(
  error: unknown,
  services: SharingServices,
): string {
  if (error instanceof ShareProjectionError) return error.message;
  return services.gateway.errorMessage(error);
}

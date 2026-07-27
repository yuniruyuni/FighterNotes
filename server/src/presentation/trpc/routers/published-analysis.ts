import { TRPCError } from "@trpc/server";
import { z } from "zod";
import {
  MAX_DELETE_PASSWORD_LENGTH,
  MIN_DELETE_PASSWORD_LENGTH,
  publishedAnalysisCandidateSchema,
} from "../../../models/published-analysis";
import {
  createPublishedAnalysisUsecase,
  deletePublishedAnalysisUsecase,
} from "../../../usecases/published-analysis";
import { handleResult } from "../handle-result";
import { publicProcedure, router } from "../init";

const deletePasswordSchema = z
  .string()
  .min(MIN_DELETE_PASSWORD_LENGTH)
  .max(MAX_DELETE_PASSWORD_LENGTH)
  .regex(/\S/u);

const createInputSchema = z.strictObject({
  analysis: publishedAnalysisCandidateSchema,
  deletePassword: deletePasswordSchema,
});

const deleteInputSchema = z.strictObject({
  id: z.string().regex(/^[A-Za-z0-9_-]{22}$/),
  deletePassword: deletePasswordSchema,
});

export const publishedAnalysisRouter = router({
  create: publicProcedure
    .input(createInputSchema)
    .mutation(async ({ ctx, input }) => {
      ensureEnabled(ctx.config.sharing.enabled);
      const result = handleResult(
        await createPublishedAnalysisUsecase(
          input.analysis,
          input.deletePassword,
          ctx.config.sharing.retentionDays,
          ctx.config.sharing.createLimits,
        ).run(ctx),
      );
      return {
        url: new URL(`/s/${result.id}`, ctx.config.publicBaseUrl).toString(),
        expiresAt: result.expiresAt.toISOString(),
      };
    }),
  delete: publicProcedure
    .input(deleteInputSchema)
    .mutation(async ({ ctx, input }) => {
      return handleResult(
        await deletePublishedAnalysisUsecase(
          input.id,
          input.deletePassword,
        ).run(ctx),
      );
    }),
});

function ensureEnabled(enabled: boolean): void {
  if (!enabled) {
    throw new TRPCError({
      code: "NOT_FOUND",
      message: "Published analysis sharing is disabled",
    });
  }
}

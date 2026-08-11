import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useReducer,
  useRef,
} from "react";
import { useLocation } from "wouter";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import {
  createPublication,
  deletePublication,
  discardPublication,
  preparePublication,
  publicationErrorMessage,
  renewPublication,
} from "../application/publication-service.js";
import {
  Publication,
  type PublicationSource,
  type PublicationState,
  type PublicationTone,
} from "../domain/publication.js";
import { shareIdFromPath } from "../domain/share.js";
import { useSharingServices } from "./SharingServicesProvider.js";

interface PublicationValue {
  state: PublicationState;
  publish(report: AdviceReport, context: AnalysisContext): Promise<void>;
  retry(): Promise<void>;
  deleteShare(): Promise<void>;
  setFeedback(message: string, tone?: PublicationTone): void;
  reset(): void;
}

const PublicationContext = createContext<PublicationValue | null>(null);

export interface PublicationRoutes {
  home: string;
  share(id: string): string;
}

export function PublicationProvider({
  children,
  routes,
}: {
  children: ReactNode;
  routes: PublicationRoutes;
}) {
  const services = useSharingServices();
  const [location, navigate] = useLocation();
  const [state, dispatch] = useReducer(
    Publication.reduce,
    undefined,
    Publication.initial,
  );
  const revision = useRef(0);

  const createShare = useCallback(
    async (source: PublicationSource, currentRevision: number) => {
      dispatch({ type: "creating", source });
      try {
        const created = await createPublication(source, services);
        if (currentRevision !== revision.current) {
          await discardPublication(created.published, source, services).catch(
            () => undefined,
          );
          return;
        }
        dispatch({ type: "created", ...created });
        navigate(routes.share(created.published.id), replacingUrlOnly());
      } catch (error) {
        if (currentRevision !== revision.current) return;
        dispatch({
          type: "failed",
          message: publicationErrorMessage(error, services),
        });
      }
    },
    [navigate, routes, services],
  );

  const publish = useCallback(
    async (report: AdviceReport, context: AnalysisContext) => {
      revision.current += 1;
      const source = preparePublication(report, context, services);
      dispatch({ type: "prepare", source });
      await createShare(source, revision.current);
    },
    [createShare, services],
  );

  const retry = useCallback(async () => {
    if (!Publication.canRetry(state)) return;
    revision.current += 1;
    const source =
      state.phase === "deleted"
        ? renewPublication(state.source, services)
        : state.source;
    await createShare(source, revision.current);
  }, [createShare, services, state]);

  const deleteShare = useCallback(async () => {
    if (!Publication.canDelete(state)) return;
    if (
      !services.capabilities.confirm(
        "この共有結果を削除します。共有先からも閲覧できなくなります。続行しますか？",
      )
    ) {
      return;
    }

    const currentRevision = revision.current;
    const deleting = state.published;
    dispatch({ type: "deleting" });
    try {
      const removedLocally = await deletePublication(
        deleting,
        state.source,
        state.storedLocally,
        services,
      );
      if (currentRevision !== revision.current) return;
      dispatch({ type: "deleted", removedLocally });
      if (location === routes.share(deleting.id)) {
        navigate(routes.home, replacingUrlOnly());
      }
    } catch {
      if (currentRevision === revision.current) {
        dispatch({ type: "deleteFailed" });
      }
    }
  }, [location, navigate, routes, services, state]);

  const setFeedback = useCallback(
    (message: string, tone: PublicationTone = "") =>
      dispatch({ type: "feedback", message, tone }),
    [],
  );

  const reset = useCallback(() => {
    revision.current += 1;
    dispatch({ type: "reset" });
    if (shareIdFromPath(location)) {
      navigate(routes.home, replacingUrlOnly());
    }
  }, [location, navigate, routes]);

  const value = useMemo(
    () => ({
      state,
      publish,
      retry,
      deleteShare,
      setFeedback,
      reset,
    }),
    [state, publish, retry, deleteShare, setFeedback, reset],
  );

  return (
    <PublicationContext.Provider value={value}>
      {children}
    </PublicationContext.Provider>
  );
}

/** URL だけを差し替え、画面が積んだ history state はその entry に残す。 */
function replacingUrlOnly() {
  return { replace: true, state: window.history.state };
}

export function usePublication(): PublicationValue {
  const value = useContext(PublicationContext);
  if (!value) {
    throw new Error("usePublication must be used within PublicationProvider");
  }
  return value;
}

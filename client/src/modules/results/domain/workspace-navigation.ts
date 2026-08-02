import type { AdviceCard } from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "./scene-selection.js";

export type WorkspaceView = "summary" | "video" | "debug";

export interface WorkspaceNavigationState {
  view: WorkspaceView;
  selected: string;
  scene: SceneSelection | null;
  nextSceneKey: number;
}

export type WorkspaceNavigationAction =
  | { type: "summary" }
  | { type: "video" }
  | { type: "debug" }
  | { type: "card"; card: AdviceCard; index: number }
  | {
      type: "scene";
      scene: Omit<SceneSelection, "key">;
      selected?: string;
    };

function initialWorkspaceNavigation(): WorkspaceNavigationState {
  return {
    view: "summary",
    selected: "summary",
    scene: null,
    nextSceneKey: 1,
  };
}

function reduceWorkspaceNavigation(
  state: WorkspaceNavigationState,
  action: WorkspaceNavigationAction,
): WorkspaceNavigationState {
  if (action.type === "summary") {
    return { ...state, view: "summary", selected: "summary" };
  }
  if (action.type === "debug") {
    return { ...state, view: "debug", selected: "debug" };
  }
  if (action.type === "video") {
    return { ...state, view: "video", selected: "video" };
  }
  if (action.type === "scene") {
    return openWorkspaceScene(
      action.selected ? { ...state, selected: action.selected } : state,
      action.scene,
    );
  }

  const evidence = action.card.evidence[0];
  const selected = `card-${action.index}`;
  if (!evidence) return { ...state, view: "summary", selected };
  return openWorkspaceScene(
    { ...state, selected },
    {
      frame: evidence.frame,
      card: action.card,
      endFrame: evidence.end_frame,
    },
  );
}

function openWorkspaceScene(
  state: WorkspaceNavigationState,
  scene: Omit<SceneSelection, "key">,
): WorkspaceNavigationState {
  return {
    ...state,
    view: "video",
    scene: { ...scene, key: state.nextSceneKey },
    nextSceneKey: state.nextSceneKey + 1,
  };
}

export const WorkspaceNavigation = {
  initial: initialWorkspaceNavigation,
  reduce: reduceWorkspaceNavigation,
  openScene: openWorkspaceScene,
};

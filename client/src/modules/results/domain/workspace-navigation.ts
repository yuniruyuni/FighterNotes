import type { AdviceCard } from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "./scene-selection.js";

export type WorkspaceView = "summary" | "video" | "debug";

export interface WorkspaceNavigationState {
  view: WorkspaceView;
  selected: string;
  scene: SceneSelection | null;
  nextSceneKey: number;
}

/** browser history へ残せる形にした scene。card は id だけを持つ。 */
export interface WorkspaceSceneLocation {
  frame: number;
  cardId: string | null;
  label?: string;
  endFrame?: number;
}

/** browser history へ残せる形にした workspace の現在位置。 */
export interface WorkspaceLocation {
  view: WorkspaceView;
  selected: string;
  scene: WorkspaceSceneLocation | null;
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

/**
 * history entry へ残す位置。scene は動画を表示している間だけ意味を持つため、
 * 他の view では持ち越さない。
 */
function workspaceLocation(state: WorkspaceNavigationState): WorkspaceLocation {
  const scene = state.view === "video" ? state.scene : null;
  return {
    view: state.view,
    selected: state.selected,
    scene: scene
      ? {
          frame: scene.frame,
          cardId: scene.card?.id ?? null,
          label: scene.label,
          endFrame: scene.endFrame,
        }
      : null,
  };
}

function sameWorkspaceLocation(a: WorkspaceLocation, b: WorkspaceLocation) {
  return (
    a.view === b.view &&
    a.selected === b.selected &&
    sameSceneLocation(a.scene, b.scene)
  );
}

function sameSceneLocation(
  a: WorkspaceSceneLocation | null,
  b: WorkspaceSceneLocation | null,
) {
  if (!a || !b) return a === b;
  return (
    a.frame === b.frame &&
    a.cardId === b.cardId &&
    a.label === b.label &&
    a.endFrame === b.endFrame
  );
}

/**
 * history から受け取った位置へ戻す。位置がない entry は初期位置として扱う。
 * scene は毎回新しい key を振り、戻る/進むでも再生位置を取り直させる。
 */
function restoreWorkspaceLocation(
  state: WorkspaceNavigationState,
  location: WorkspaceLocation | null,
  cards: readonly AdviceCard[],
): WorkspaceNavigationState {
  const target = location ?? workspaceLocation(initialWorkspaceNavigation());
  const scene = target.scene;
  if (!scene) {
    return {
      ...state,
      view: target.view,
      selected: target.selected,
      scene: null,
    };
  }
  return {
    ...state,
    view: target.view,
    selected: target.selected,
    scene: {
      frame: scene.frame,
      card: cards.find((card) => card.id === scene.cardId) ?? null,
      label: scene.label,
      endFrame: scene.endFrame,
      key: state.nextSceneKey,
    },
    nextSceneKey: state.nextSceneKey + 1,
  };
}

export const WorkspaceNavigation = {
  initial: initialWorkspaceNavigation,
  reduce: reduceWorkspaceNavigation,
  openScene: openWorkspaceScene,
  location: workspaceLocation,
  sameLocation: sameWorkspaceLocation,
  restore: restoreWorkspaceLocation,
};

import { useCallback, useReducer, useState } from "react";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import {
  readHistoryStack,
  useHistoryStack,
} from "~/shared/browser/use-history-stack.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import {
  type WorkspaceLocation,
  WorkspaceNavigation,
  type WorkspaceNavigationAction,
  type WorkspaceNavigationState,
} from "../../domain/workspace-navigation.js";

export interface WorkspaceNavigator {
  navigation: WorkspaceNavigationState;
  /** 同じ項目を選び直した場合も含め、表示切り替えごとに増える。 */
  focusRevision: number;
  navigate(action: WorkspaceNavigationAction): void;
  openScene(scene: Omit<SceneSelection, "key">): void;
  /** workspace が積んだ history entry を捨てて解析前の位置へ戻す。 */
  leave(onSettled?: () => void): void;
}

/**
 * workspace の項目移動を browser history と対応付ける。
 *
 * 解析ごとに独立した stack key を使うため、別の解析や reload が残した entry を
 * この解析の位置として復元しない。
 */
export function useWorkspaceNavigation(
  session: string,
  cards: readonly AdviceCard[],
): WorkspaceNavigator {
  const key = `workspace-navigation:${session}`;
  const [navigation, setNavigation] = useState(() =>
    WorkspaceNavigation.restore(
      WorkspaceNavigation.initial(),
      readHistoryStack<WorkspaceLocation>(key),
      cards,
    ),
  );
  const [focusRevision, requestFocus] = useReducer(
    (revision: number) => revision + 1,
    0,
  );

  const restore = useCallback(
    (location: WorkspaceLocation | null) => {
      setNavigation((previous) =>
        WorkspaceNavigation.restore(previous, location, cards),
      );
      requestFocus();
    },
    [cards],
  );
  const history = useHistoryStack<WorkspaceLocation>(key, restore);

  const navigate = (action: WorkspaceNavigationAction) => {
    const next = WorkspaceNavigation.reduce(navigation, action);
    const to = WorkspaceNavigation.location(next);
    const from = WorkspaceNavigation.location(navigation);
    if (!WorkspaceNavigation.sameLocation(from, to)) history.push(to);
    setNavigation(next);
    requestFocus();
  };

  const openScene = (scene: Omit<SceneSelection, "key">) => {
    const cardIndex = scene.card
      ? cards.findIndex((card) => card.id === scene.card?.id)
      : -1;
    navigate({
      type: "scene",
      scene,
      selected: cardIndex >= 0 ? `card-${cardIndex}` : "video",
    });
  };

  return {
    navigation,
    focusRevision,
    navigate,
    openScene,
    leave: history.unwind,
  };
}

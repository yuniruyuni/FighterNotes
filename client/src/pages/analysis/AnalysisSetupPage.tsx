import {
  AnalysisSession,
  AnalysisSettings,
  SetupNotes,
  useAnalysisSession,
  VideoFilePicker,
} from "~/modules/analysis/index.js";

export function AnalysisSetupPage() {
  const session = useAnalysisSession();
  const { state } = session;
  const busy = state.phase === "analyzing" || state.phase === "canceling";
  const canceled = state.phase === "canceled";

  return (
    <div id="screen-setup">
      <div className="hero">
        <h1 className="hero-title" data-echo={"FIGHTER\nNOTES"}>
          Fighter
          <br />
          Notes
        </h1>
        <p className="hero-label">SF6 リプレイ解析</p>
        <p className="tagline">
          Street Fighter 6 の対戦リプレイを録画した動画から、
          被弾場面・弱点・練習メニューを自動で抽出する、カプコン非公式の個人開発ツールです。
        </p>
      </div>
      <div className="setup-columns">
        <div className="setup-main">
          <VideoFilePicker
            file={state.file}
            disabled={busy}
            onChange={session.setFile}
          />
          <AnalysisSettings
            side={state.side}
            ownCharacter={state.ownCharacter}
            opponentCharacter={state.opponentCharacter}
            busy={busy}
            canAnalyze={
              session.runtime.available && AnalysisSession.canStart(state)
            }
            unavailableReason={
              session.runtime.available ? undefined : session.runtime.reason
            }
            onSideChange={session.setSide}
            onOwnCharacterChange={session.setOwnCharacter}
            onOpponentCharacterChange={session.setOpponentCharacter}
            onSubmit={() => void session.analyze()}
          />
          {(busy || canceled || state.error) && (
            <div className="card progress-card">
              <h2>
                {state.error
                  ? "解析エラー"
                  : canceled
                    ? "解析を中止しました"
                    : "解析中…"}
              </h2>
              {busy && (
                <progress max={100} value={state.progress}>
                  {state.progress}%
                </progress>
              )}
              <div className={state.error ? "analysis-error" : "status"}>
                {state.error || state.status}
              </div>
              {busy && (
                <button
                  type="button"
                  className="analysis-cancel-btn"
                  disabled={state.phase === "canceling"}
                  onClick={session.cancel}
                >
                  {state.phase === "canceling"
                    ? "中止しています…"
                    : "解析を中止"}
                </button>
              )}
            </div>
          )}
        </div>
        <aside className="setup-notes">
          <SetupNotes />
        </aside>
      </div>
    </div>
  );
}

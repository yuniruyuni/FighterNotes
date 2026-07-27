export function AppHeader() {
  return (
    <>
      <header className="site-header">
        <div className="brand">
          Fighter Notes
          <span className="brand-tag">
            スト6の対戦リプレイを撮影した動画を解析して改善案を提案
          </span>
        </div>
        <div className="header-note">SF6 Replay Analyzer</div>
      </header>
      <div className="header-strip" aria-hidden="true" />
    </>
  );
}

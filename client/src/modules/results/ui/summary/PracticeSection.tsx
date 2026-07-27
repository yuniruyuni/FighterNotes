export function PracticeSection({ items }: { items: readonly string[] }) {
  return (
    <section className="summary-section" data-wm="Training">
      <h2>練習メニュー</h2>
      {items.length === 0 ? (
        <p className="muted-note">なし</p>
      ) : (
        items.map((item) => (
          <div className="practice-item" key={item}>
            • {item}
          </div>
        ))
      )}
    </section>
  );
}

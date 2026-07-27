export function SetupNotes() {
  return (
    <>
      <div className="bubble">
        <span className="bcat">Source Video</span>
        <h3>どんな動画が必要？</h3>
        SF6 の
        <strong>対戦リプレイを再生した画面を、そのまま録画した動画</strong>
        です。 こちらでテストしたのは{" "}
        <strong>Steam 版（Windows）の画面を OBS で録画したもの</strong>
        だけです。それ以外の環境での録画は正しく解析できない可能性があります。
        <strong>1920×1080 / 60fps（固定）/ 16:9</strong> で録画してください
        （可変フレームレートは避ける）。
        <p className="bubble-sub">
          対戦前のメニュー画面や対戦後の映像が含まれていても問題ありません
          （ラウンド開始を自動検出します）。
        </p>
      </div>
      <div className="bubble">
        <span className="bcat">Game Settings</span>
        <h3>リプレイ再生時の SF6 側の設定</h3>
        解析に必須です。次の表示のもとで録画してください。
        <ul>
          <li>
            <strong>入力履歴を両プレイヤーとも ON</strong>
            （守り・暴れ・投げなどの行動推定に使います）
          </li>
          <li>
            <strong>フレームメーターを ON</strong>
            （被弾原因のフレーム単位の推定に使います）
          </li>
          <li>
            <strong>HUD（体力・ドライブ・SA・タイマー）を隠さない</strong>
          </li>
        </ul>
      </div>
      <div className="bubble">
        <span className="bcat">Warning</span>
        <h3>画面加工をしない</h3>
        クロップ・拡大・字幕・配信オーバーレイ・顔出し・黒帯の追加など、
        <strong>ゲーム画面そのもの以外の要素を加えない</strong>
        でください。画面の位置ずれは HUD の読み取りを狂わせます。
      </div>
    </>
  );
}

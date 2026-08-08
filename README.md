# Fighter Notes

Street Fighter 6 のリプレイ再生画面を録画した動画から、対戦イベント、改善点、
練習項目を抽出する Web アプリケーションです。

動画の demux、decode、画像認識、イベント生成は browser 内で行います。解析 engine は Rust/WASM、
client は TypeScript / React、配信と集計結果の共有は Bun + Hono + PostgreSQL で構成されています。

## 現在できること

- HUD、入力履歴、リプレイ用フレームメーターを全フレームで読み取る
- round、damage、hit / block、jump、throw、DI、Drive Rush、burnout、両者のSA/CA使用などを同じ時間軸へ統合する
- 確反、対空、防御中の行動、resource 管理などを evidence 付きの指摘と戦術統計へまとめる
- 指摘の場面を元動画で見直し、認識結果と meter timeline を debug 表示する
- 対戦集計を browser の IndexedDB に最大 200 件保存する
- 動画を含まない集計結果の公開 URL を作成し、この端末または削除コードから削除する

解析はゲーム内部データではなく録画画面に対する決定的なルールベース推定です。
検出結果は断定ではなく、元動画を見直すための候補として扱います。

## 対応する録画

現在の較正は次の条件を前提にしています。

- Street Fighter 6 のリプレイ再生画面
- 1920x1080、16:9、固定 60fps
- P1/P2 の入力履歴とリプレイ用フレームメーターを表示
- HUD、画面位置、crop を変更せず、字幕や配信 overlay を重ねない

解析前に自分の side と両者の character を選択します。character の OCR 自動判定は行いません。
詳細と精度上の限界は [動画解析パイプライン](docs/analysis.md) を参照してください。

動画解析には Secure Context（HTTPS または `localhost`）と WebCodecs `VideoDecoder` が必要です。
ブラウザは最新版を使用してください。HTTPS ではない LAN 内 IP からの利用や、動画の codec を
ブラウザが WebCodecs で decode できない場合は、decode 開始前に理由を表示します。

## 解析結果の公開

解析は公開共有なしで利用できます。解析完了後に「共有URLを生成」を実行した場合だけ、
集計結果を server へ送信して公開 URL を作成します。既定の公開期間は 30 日です。

server へ送るのは両者の character、round 集計、指摘の種別・件数・強度、戦術統計だけです。
元動画、画像、場面 clip、ファイル名、入力した自由文、詳細 report は送信しません。
削除コードは結果画面に表示し、この browser に保存します。

公開 URL は認証されていない public page です。検索や preview service の cache は、削除または
期限切れ後も残る可能性があります。保存項目と削除仕様は [共有](docs/sharing.md) を参照してください。

## Repository

```text
crates/
  pixel-color/       HUD と入力履歴が共有する色空間変換
  frame-meter/       frame-meter cell の認識
  meter-tracker/     game frame timeline の復元
  hud-vision/        HP・drive・SA gauge と round 開始表示の読み取り
  input-vision/      入力履歴欄の読み取りと系列の補修
  attack-info-vision/ 攻撃情報表示の読み取りと系列化
  temporal-confirm/  読み取り結果の時系列確定
  analysis-context/  対戦の前提と技の frame data
  match-event-layer/ 確定済み観測から試合 event を組み立てる
  advice-report/     event から指摘・優先順位・根拠を組み立てる
  spatial-refine/    位置関係による event の再評価
  video-analyzer/    各層の pipeline 結線と公開 API
  wasm-bridge/       browser Worker から使う WASM API
client/              browser SPA、動画 decode、Worker、表示、local storage
server/              static 配信、共有 API / page、cleanup batch
schema/              PostgreSQL の desired schema
.github/workflows/   CI、schema plan、image build、Cloud Run deploy
```

全体のデータフローは [システムアーキテクチャ](docs/architecture.md) に記載しています。

## ローカル起動

Docker Compose を使うと PostgreSQL、migration、application をまとめて起動できます。

```bash
docker compose up --build
```

起動後に `http://localhost:3000` を開きます。

host 上で build する場合は Rust 1.95、`wasm32-unknown-unknown`、wasm-pack 0.15.0、
Bun 1.3.x が必要です。

```bash
bun install --frozen-lockfile
bun run check
bun scripts/validate-frame-data.ts
bun run build
bun test scripts/
```

`bun run check` は generated WASM binding を先に作り、RustとTypeScriptの検査をまとめて実行します。
共有を含む通常動作を host 上で確認する場合は PostgreSQL と schema migration も必要です。
詳しい起動、watch、integration test、frame data contract の検証は
[開発ガイド](docs/development.md) を参照してください。

## ドキュメント

設計、解析、共有、開発、運用の詳細は次を参照してください。

| 文書 | 内容 |
| --- | --- |
| [architecture.md](docs/architecture.md) | component、runtime flow、保存場所 |
| [analysis.md](docs/analysis.md) | 認識、event、advice、空間解析、限界 |
| [sharing.md](docs/sharing.md) | 公開 payload、保持、削除、quota |
| [development.md](docs/development.md) | build、test、frame data contract |
| [DEPLOY.md](docs/DEPLOY.md) | CI/CD、migration、Cloud Run、rollback |
| [security-operations.md](docs/security-operations.md) | 監視、緊急停止、credential / DB incident |
| [DATA_NOTICE.md](DATA_NOTICE.md) | 正規化済みframe data・認識用統計モデルの取扱い |

## 非公式・非提携表記

Fighter Notesは個人が開発・運営する非公式ツールであり、株式会社カプコンおよび
その関連会社との提携、協賛または承認関係はありません。製品名、会社名、ゲーム名、
キャラクター名、ロゴその他の商標・著作物は、株式会社カプコンまたは各権利者に
帰属します。

## ライセンス

特に記載がない限り、Fighter Notesが独自に作成したsource code、test、script、設定、
CI/CD、SQL schema、CSS、HTMLおよび文書は
[MIT License](LICENSE)で提供します。

次のものはMIT Licenseの対象外です。

- `crates/analysis-context/data/`以下の正規化済みframe dataとmanifest
- 実ゲーム撮影動画から生成した認識用の数値・統計モデル
- `client/src/shared/assets/`以下の画像その他のmedia asset
- `THIRD_PARTY_NOTICES.md`および第三者のlicense・NOTICE本文
- npm / Cargo dependencyと、bundle、WASM、binaryに含まれる第三者component
- 製品名、会社名、ロゴ、商標、ゲーム由来要素その他の第三者の権利

正規化済みデータと認識用統計モデルについては [DATA_NOTICE.md](DATA_NOTICE.md)、
第三者componentについては [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)を
確認してください。生成された成果物のうちFighter Notesが権利を持つ部分には
MIT Licenseが適用されますが、第三者componentにはそれぞれのlicenseが引き続き適用されます。

Contributionの条件は [CONTRIBUTING.md](CONTRIBUTING.md)を確認してください。

# システムアーキテクチャ

最終確認: 2026-07-22

## 目的

Fighter Notes は、Street Fighter 6 のリプレイ再生画面を録画した動画から、
対戦を振り返るためのイベント、指摘、練習項目を生成する Web アプリケーションである。

動画のデコードと解析はブラウザ内で完結する。サーバーは解析バックエンドではなく、
SPA の配信、軽量な共有結果の保存と公開、期限切れデータの cleanup を担当する。

## 全体構成

```text
ローカル動画ファイル
  -> client SPA
     -> MP4Box + WebCodecs / Canvas
     -> analyzer Worker
        -> Rust/WASM
           -> frame-meter
           -> meter-tracker
           -> video-analyzer
     -> 結果画面 / 動画プレイヤー / デバッグビュー
     -> IndexedDB の対戦集計
     -> 共有用の集計値だけを射影
        -> tRPC
           -> Bun + Hono server
              -> PostgreSQL
              -> /s/:id の server-rendered HTML
```

## コンポーネント

| 領域 | 実装 | 責務 |
| --- | --- | --- |
| Client | `client/src` | ファイル選択、動画デコード、Worker 制御、結果表示、ローカル履歴、共有操作 |
| WASM bridge | `crates/wasm-bridge` | JavaScript と Rust のメモリ境界、解析セッション API |
| Frame meter | `crates/frame-meter` | リプレイ用フレームメーターのセル観測と状態分類 |
| Meter tracker | `crates/meter-tracker` | セル観測からゲームフレーム単位の状態タイムラインを復元 |
| Video analyzer | `crates/video-analyzer` | HUD・入力の確定、イベント帰属、空間再評価、助言レポート生成 |
| Server | `server/src` | 静的配信、共有 API、公開ページ、rate limit、cleanup batch |
| Database | `schema` | 共有結果、指摘、戦術統計、作成 quota event、共有rate-limit counterの保存 |
| Delivery | `Dockerfile`、`cloudrun*.yaml`、`.github/workflows` | build、migration、Cloud Run service / Job のリリース |

## ブラウザ側

### 起動と画面

`client/src/entrypoints/main.tsx` は React root に `App` を描画する。`app/routes.tsx` は
Wouter で URL を解決し、
次の画面を構成する。

- `/`: 動画選択、解析、結果サマリー、証拠区間プレイヤー、認識デバッグ
- `/manage`: この端末で作成した共有の一覧と削除
- `/manage/:id`: 共有 ID を入力済みにした削除画面

`AnalysisSessionProvider` は選択ファイル、解析 context、進捗、特徴量、タイムライン、report を、
`PublicationProvider` は共有作成・削除と公開 URL の状態を memory 上で保持する。
`AppProviders` は解析、結果表示、共有のブラウザ実装を各 Provider へ注入する。
解析完了だけでは共有処理を呼ばず、結果画面で利用者が「共有URLを生成」を実行した場合にだけ
`PublicationProvider` が共有を作成する。
ページの reload 後に動画付き解析画面を復元する機能はない。

解析直後の `/s/:id` だけは、memory 上の share ID と一致すれば `AnalysisWorkspace` を維持する。
直接 navigation や reload では server-rendered 公開ページを取得するため、SPA は同じ URL を
network navigation し直す。

### Client の境界

```text
entrypoints -> app / pages -> modules -> shared
                             domain
                               ^
                          application
                           ^         ^
                         ui     infrastructure
```

- `entrypoints`: main thread と analyzer Worker の起動だけを行う。
- `app`: Provider の組み立て、Wouter route、path を所有する。
- `pages`: 複数の module UI を一画面へ合成する。
- `modules/analysis`: 解析 context、report、セッション、decode / Worker 実装を所有する。
- `modules/results`: サマリー、プレイヤー、デバッグ viewer、IndexedDB 履歴を所有する。
- `modules/sharing`: 公開用射影、共有 API、localStorage、共有管理 UI を所有する。
- `shared`: module に依存しない UI、browser hook、style、asset を所有する。

module 間は `index.ts`、`contracts.ts`、`browser.ts` など明示した公開 entrypoint だけを使う。
React UI は concrete adapter を直接 import せず、application port を Provider から受け取る。
各 module の `browser.ts` が browser adapter を application port へ割り当て、
`modules/results/ui/debug` は React の操作、viewer session、Canvas 描画を所有する。
`modules/results/infrastructure/frame-access` は動画フレーム取得、
`modules/results/infrastructure/frame-inspection` は generated WASM inspection、
`modules/results/infrastructure/history-persistence` は IndexedDB 履歴を所有する。
debug viewer のボタンとキーボード入力は UI が domain の navigation action へ変換し、
domain model が action ごとの移動フレーム数とカーソル範囲を決定する。
`bun run check:arch` は循環、layer の逆依存、module 内部への直接参照を検査する。

### 動画処理

通常経路は
`client/src/modules/analysis/infrastructure/pipeline/browser-analysis-engine.ts` から起動する。
`video-decoding` が MP4Box と WebCodecs、`frame-extraction` が Canvas 上の領域抽出、
`spatial-analysis` が候補区間の第二段解析、`worker-bridge` が main thread と Worker の通信、
`wasm-bridge` が生成済み WASM API との境界、`diagnostics` が browser の開発者向け出力を所有する。

`video-decoding/mp4-video-source.ts` が動画トラックを demux して、
WebCodecs の `VideoDecoder` へ encoded sample を供給する。各 decoded frame から
次の領域だけを `OffscreenCanvas` へ切り出す。

| 領域 | 1920x1080 基準 | 用途 |
| --- | --- | --- |
| HUD strip | `y=64..133` | HP と Drive gauge |
| Input strip | `y=232..267` | P1/P2 の入力履歴 row 0 |
| Frame-meter strip | `y=796..873` | P1/P2 のフレームメーター |

2 組の transferable buffer を main thread と
`client/src/entrypoints/analyzer-worker.ts` の間で往復させる。
デコーダの queue と Worker 未処理フレーム数には上限があり、長い動画でも
ImageData が無制限に滞留しないよう backpressure を掛ける。

動画解析は Secure Context と WebCodecs `VideoDecoder` を必須とする。HTTPS ではない
LAN 内 IP、Worker、OffscreenCanvas 2D、`VideoFrame` の bitmap 切り出し、`VideoDecoder` の
いずれかを使えないブラウザからの実行は開始前に拒否する。

ファイル選択時は `video-decoding/mp4-video-preflight.ts` が MP4Box の要求 offset に沿って
metadata 範囲だけを段階的に読み、`mdat` 全体を読み込まず container、track 寸法、track matrix、
sample の presentation timestamp を検査する。1920x1080、59〜61fps CFR、回転・変形なしの
非 fragmented MP4 だけを受け付け、続けて動画固有 codec と実際の `VideoFrame` bitmap 切り出しを
probe する。選び直しは前の検証を abort し、遅れて完了した結果を現在の選択へ適用しない。
検証済み metadata は exact `File` identity と一緒に pipeline へ渡し、通常 demux で再利用する。
検証に通るまでは解析 Worker を作成せず、したがって WASM も初期化しない。

### Rust/WASM 境界

`Analyzer` は HUD、入力、フレームメーター用の固定バッファを WASM linear memory に持つ。
Worker は JavaScript 側の buffer をその領域へコピーし、フレームごとに次を呼ぶ。

1. `analyze_meter_inplace`
2. `push_hud_features_inplace`
3. `analyze_input_inplace`

第一段の完了後に `finish` で確定処理とイベント生成を行う。イベントから選ばれた
短い候補区間だけを 480x270 で再デコードし、`SpatialWindowAnalyzer` へ渡す。
最後に `refine_with_spatial` が距離、前進、空中状態などの証拠をイベントへ追加する。

詳細は [analysis.md](./analysis.md) を参照する。

## サーバー側

### 起動モード

`server/src/index.ts` は起動時に遅延接続の PostgreSQL pool と application context を作る。

- 引数なし: Hono application を `Bun.serve` で起動
- `--batch=cleanup`: 期限切れ共有、古いrate-limit counter、古いquota eventを削除して終了

実際の DB connection は最初の query で確立するため、静的配信と `/health` だけなら DB なしでも応答する。
共有 read / create / delete と cleanup batch には、schema 適用済みの DB が必要である。
`SHARE_RESULTS_ENABLED=false` は create と公開 read を止めるが、削除 API は継続する。

### レイヤー

| レイヤー | ディレクトリ | 内容 |
| --- | --- | --- |
| Presentation | `server/src/presentation` | Hono route、tRPC router、HTML、cache、rate limit |
| Use case | `server/src/usecases` | create / get / delete / cleanup の処理列と transaction 境界 |
| Model | `server/src/models` | closed schema、ID、削除 credential、quota、lifecycle |
| Repository | `server/src/repositories` | specification を PostgreSQL query へ変換する adapter |
| Infrastructure | `server/src/infra` | DB pool、parameterized SQL、logger |

Repository は read / write capability と transaction context を明示的に受け取る。
共有作成では Argon2id の前に日次件数・active row・logical payload bytesを事前確認し、hash 後の
transaction で advisory lock を取得して quota を再確認する。quota event と結果本体の insert は
同じ transaction で行う。共有rate limitはclient keyのdigestとbucketをPostgreSQLの原子的
upsertで更新し、instanceやcold startをまたいで共有する。

期限切れcleanupは期限用`(expires_at, created_at, id)`とretention用
`(created_at, expires_at, id)`を別々に走査し、各bounded CTE内で`FOR UPDATE SKIP LOCKED`と
DELETEを完結させる。storage quotaはparentに記録したlogical sizeの合計で、
物理relation file lengthではないためDELETEした容量を直後から再利用できる。

### HTTP surface

| Method / path | 用途 |
| --- | --- |
| `GET /health` | DB非依存、`no-store`のprocess liveness |
| `GET /ready` | runtime app role・DB tunnel・catalog contract・grantの`no-store` readiness |
| `POST /api/trpc/publishedAnalysis.create` | 軽量な共有結果を作成 |
| `POST /api/trpc/publishedAnalysis.delete` | 削除コードで共有を削除 |
| `GET /s/:id` | PostgreSQL の共有結果から HTML を生成 |
| `GET /manage`、`GET /manage/:id` | SPA fallback で管理画面を配信 |
| その他の静的 path | `client/static` を配信し、未知 path は SPA へ fallback |

`/s/:id` を通常 navigation した場合は server-rendered の公開ページになる。
解析直後の同一 tab では History API で同じ path に変更するだけなので、動画付きの
ローカル結果画面を維持する。このローカル画面と公開 HTML は別の表現である。

## データの所在

| データ | 保存場所 | 寿命 |
| --- | --- | --- |
| 元動画 | 利用者の File と browser memory | tab を閉じるまで |
| decoded frame / RGBA | main thread、Worker、WASM memory | 解析中 |
| 詳細レポート、特徴量、タイムライン | browser memory | tab を閉じるまで |
| 対戦集計履歴 | IndexedDB `fighter-notes/analysis-history` | 最大 200 件、または利用者が個別・全件削除するまで |
| 対戦履歴の保存設定 | localStorage | 利用者が変更するか site data を削除するまで |
| 共有 ID と削除コード | localStorage | 共有期限まで、または削除まで |
| 公開用集計 | PostgreSQL | 既定 30 日、または手動削除まで |

動画、場面画像、証拠フレーム、ファイル名、詳細レポートは PostgreSQL へ保存しない。
共有境界の詳細は [sharing.md](./sharing.md) を参照する。

解析履歴の IndexedDB と、共有管理情報の localStorage は独立した lifecycle を持つ。
結果画面から解析履歴を削除しても、共有 URL と削除コードは保持する。

## Build と配信

production image は multi-stage build で作る。

1. Rust 1.95 と wasm-pack 0.15.0 で `wasm-bridge` を build
2. Bun で SPA、Worker、WASM binary、HTML、分割 CSS、image を `client/static` へ出力
3. Bun で server を単一 executable に compile
4. distroless nonroot image へ executable と static files だけを配置

Cloud Run では application service、schema migration Job、cleanup Job を分ける。
各 workload の PostgreSQL 接続は Cloudflare Access TCP sidecar を経由する。
詳細は [DEPLOY.md](./DEPLOY.md) を参照する。

## 維持する境界

- 動画解析を server API へ移さない。
- UI は Rust の event / report JSON を表示し、判定ロジックを重複実装しない。
- viewer 表示と event layer は、時間方向に確定済みの同じ特徴量を使う。
- 空間解析は第一段の証拠を置き換えず、候補の確認または棄却に使う。
- 公開モデルへ自由文、動画依存値、削除コードを混ぜない。
- live infrastructure の状態を repository manifest だけから断定しない。

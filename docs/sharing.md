# 分析結果の共有と保存

最終確認: 2026-08-03

## 基本契約

動画解析そのものはブラウザ内で行い、解析しただけでは結果を server へ送信しない。
解析完了後に利用者が結果画面の「共有URLを生成」を明示的に実行した場合だけ、
軽量な共有結果を作成する。

共有は次の性質を持つ。

- 共有URLを生成しない限り、解析結果は公開されない。
- URL を知る人は認証なしで閲覧できる。
- 内容は作成後に変更できない snapshot である。
- 既定の公開期限は作成から30日。
- 利用者は自動発行された削除コードで期限前に削除できる。
- 動画や詳細な場面データは共有 payload に含めない。
- 共有作成に失敗しても、同じ tab のローカル解析結果は表示できる。

この契約は client の共有URL生成前 disclosure とその test、client の射影、server schema、
公開ページで一致させる。

## 共有作成の流れ

1. 利用者が side と両キャラクターを指定して解析を開始する。
2. client が解析結果をローカル画面へ表示する。この時点では共有 API を呼ばない。
3. 利用者が結果画面の「共有URLを生成」を実行する。
4. client が解析結果と `AnalysisContext` から公開可能な集計だけを射影する。
5. client が暗号学的乱数から12文字の削除コードを生成する。
6. 集計と削除コードを same-origin tRPC mutation へ送る。
7. server が strict schema を再検証し、削除コードを Argon2id で hash 化する。
8. server が 128-bit random ID の共有を PostgreSQL に作る。
9. client は共有 ID、削除コード、期限、対戦 label を localStorage に保存する。
10. 現在の tab は History API で `/s/:id` に変更するが、動画付きローカル結果画面を維持する。

別の tab や端末から `/s/:id` を開くと、server-rendered の公開ページを表示する。

## 公開するデータ

`client/src/modules/sharing/domain/published-analysis.ts` が `AdviceReport` を次の closed model へ変換する。

| 項目 | 内容 |
| --- | --- |
| `rulesetVersion` | 解析ルール世代 |
| `ownCharacter` | 自キャラクターの closed ID |
| `opponentCharacter` | 相手キャラクターの closed ID |
| `rounds` | 検出数、勝ち、負け、未確定 |
| `findings` | 種別、assessment、件数、basis point 化した severity |
| `tactics` | 対空、DI、raw Drive Rush、dash throw、throw whiff、minus 後の回答、burnout 集計 |
| `superArts` | ruleset v9以降の両者のSA1/2/3/CA使用数と結果、自分側だけの利用文脈。各側にcomplete / partial / unavailableを持つ |

character ID、finding ID、assessment は client、server、database の全境界で allowlist にする。
未知 ID、重複 finding、非整数、round 数の不整合、上限超過、未知 field は拒否する。
SA/CAの件数は、全検出ラウンドの各フレームで試合画面かつその側のゲージを信頼して
観測できた割合が70%以上あり、消費確定に必要な90フレーム内12信頼サンプル、
最大欠測区間、ラウンド開始・終了フレームの観測条件をすべて満たす側だけ`complete`にする。
欠落フレーム、非試合画面、不確実な読みはすべて欠測として扱う。単発の信頼フレームや検出済みSAイベントだけでは、
ほかの使用を見逃していないと保証できないため`complete`にしない。ただし、時間方向に確定したSA/CAを
1件以上検出できた側は`partial`とし、level、結果、自分側文脈のcountを「確認できた下限」として公開する。
完全性を満たさず検出も0件の側は`unavailable`だけを公開し、件数0として扱わない。
`complete`と`partial`は全countを必須にし、`unavailable`へのcount混入を拒否する。

## 公開しないデータ

次は server へ保存しない。

- 元動画、音声、decoded frame、clip、screenshot
- 動画ファイル名、size、lastModified
- frame number、timestamp、evidence range
- HP / Drive の frame-by-frame 系列
- 入力履歴、meter timeline、spatial observation
- SA/CAの表示damage sample・合計、low-scaling判定用の値、最終gauge量
- 相手側のSA/CA利用文脈
- `AdviceReport` の summary、description、practice などの自由文
- browser の詳細な対戦履歴
- 削除コードの平文

削除コードは create / delete request では server へ送信されるが、hash 化後に破棄する。
create response、公開 URL、公開 HTML、share text、application log へ含めない。

公開ページの日本語は保存 payload ではなく、
`server/src/presentation/published-analysis-catalog.ts` の固定文言から生成する。

## Browser storage

### IndexedDB

database `fighter-notes` の `analysis-history` store に最大200件の対戦集計を保存する。

- record ID: size、lastModified、side、両キャラクター、ruleset から作る SHA-256 digest
- value: 作成時刻、ruleset、両キャラクター、round 数、戦術統計

動画ファイル名、動画本体、詳細レポートは保存しない。旧形式の record ID に残っている
動画ファイル名は、database version 2 への更新時に不透明な ID へ置き換える。
結果画面では今後の自動保存を停止・再開でき、旧 ruleset を含む保存件数の確認、個別削除、
全件削除ができる。解析履歴の削除は次項の共有管理情報を削除しない。

### localStorage

key prefix `fighter-notes:managed-share:v1:` に次だけを保存する。

- 共有 ID
- 削除コード
- 作成時刻と期限
- `自キャラ vs 相手キャラ` の label

期限切れ、不正形式、破損 record は一覧読込時に削除する。共有を削除した場合も該当 record を消す。

対戦履歴の保存 ON/OFF は独立した key
`fighter-notes:analysis-history:saving-enabled:v1` に保存する。初期値は ON とし、storage を
利用できない場合または値が壊れている場合は OFF として扱う。共有管理情報と保存設定の key を
分けることで、解析履歴だけの削除が共有 URL・削除コードへ波及しないようにする。

## Server storage

PostgreSQL は次の table を持つ。

| table | 内容 |
| --- | --- |
| `published_analyses` | ID、version、character、round、削除 hash、logical size、作成・期限 |
| `published_analysis_findings` | finding の順序、種別、assessment、件数、severity |
| `published_analysis_tactics` | 戦術統計 |
| `published_analysis_super_arts` | ruleset v9以降のSA/CA公開契約が存在することを示すmarker |
| `published_analysis_own_super_arts` | complete / partialな自分側だけが持つ、完全性flagと全列必須のSA/CA集計 |
| `published_analysis_opponent_super_arts` | complete / partialな相手側だけが持つ、完全性flagと全列必須のSA/CA集計 |
| `published_analysis_create_events` | UTC 日次 create quota 用の成功 event |
| `published_analysis_rate_limits` | bucket別の共有固定窓counter（client keyはdigestのみ） |

finding、tactics、super arts markerとside集計はparent削除時にcascade deleteする。create eventは結果本体と独立させ、
同じ日に共有を削除しても日次 create 件数が減らないようにする。

schema version は現在1、presentation revision は1。server はruleset 3〜9を受理し、
新規解析はruleset 9を生成する。旧rulesetのrowはmarkerを持たず、従来どおりの公開ページを表示する。
v9以降はmarkerを必須とし、side集計行が無ければ`unavailable`、行の`complete`がfalseなら`partial`、
trueなら`complete`として復元する。

## 削除と期限

新規共有の削除 credential は Argon2id PHC string として保存する。
不正 ID、存在しない ID、誤った削除コード、削除済み ID は外部から区別できない応答にする。

正常な削除後、origin の新しい read は `404` になる。正常な公開 HTML は browser と edge で
最大15秒 cache できるため、削除直前の response がその範囲だけ見える可能性がある。
期限切れ、`404`、`429`、`503` は `no-store` とし、期限後の stale response は許可しない。

cleanup batch は次のどちらかを満たす parent row を削除する。

- `expires_at` を過ぎた
- `created_at + SHARE_RETENTION_DAYS` を過ぎた

1 batch の既定値は500件、最大1000 batch で停止する。期限判定は
`(expires_at, created_at, id)`、retention判定は`(created_at, expires_at, id)`の専用indexと
別々のbounded CTEを使う。各CTEは`FOR UPDATE SKIP LOCKED`とDELETEを1 statementで完結させ、
並行Jobが同じrowを処理しない。row lockに必要なUPDATE権限は、値がCHECKで固定された
`schema_version`列だけに限定する。全 parent cleanup が完了した場合だけ、終了後2分を過ぎた
rate-limit counterと2 UTC 日より古いcreate quota eventを削除する。repository はcleanup Jobを定義するが、
定期実行の Scheduler は外部管理である。頻度と live 状態は deployment 時に別途確認する。

## HTTP と cache

| Surface | 制約 |
| --- | --- |
| tRPC POST body | 12 KiB 以下 |
| Content-Type | `application/json` のみ |
| Origin | request origin または `PUBLIC_BASE_URL` と一致 |
| mutation batch | create / delete を含む tRPC batch は拒否 |
| share create / delete | client key ごとの固定窓 rate limit |
| public GET | client key ごとの固定窓 rate limit |
| public HTML `200` | 有効期限を超えない最大15秒 cache |
| error response | `no-store` |
| `/health`、`/ready` | 成否とも`no-store` |

application rate limiter は PostgreSQL の原子的 upsert を使う固定窓で、Cloud Run instance と
cold start をまたいで共有する。create、delete、public read は別 bucket とし、client key は
SHA-256 digest だけを保存する。counter store が利用できない場合は共有 request だけを `503` で
fail closed にし、静的配信と `/health` は継続する。

create は hard quota の事前確認後に Argon2id を実行し、書込み transaction の advisory lock 内で
quota を再確認する。hash / verify は process ごとに合計同時実行数と待機 queue を制限する。

| Hard quota | 既定値 |
| --- | ---: |
| UTC 日次作成成功数 | 1,000 |
| active parent row | 50,000 |
| 保存payloadのlogical使用量 | 1 GiB |

logical使用量は新規rowではclosed payloadのserialized byte数、移行前rowでは安全側の8 KiBとして
parentに保存する。物理relation file sizeとは分離しているため、通常のDELETE直後に減少し、
`VACUUM FULL`なしでcreateを再開できる。quota到達時はfail closedとし、cleanup lagや濫用の原因を
確認せず上限だけを緩めない。物理DB容量は別signalとして監視する。

## 設定

| 環境変数 | 既定値 | 用途 |
| --- | --- | --- |
| `SHARE_RESULTS_ENABLED` | `true` | `false` で create と公開 read を停止 |
| `PUBLIC_BASE_URL` | `https://fighter.yuniruyuni.net` | 共有 URL と許可 origin |
| `SHARE_RETENTION_DAYS` | `30` | 公開期限と cleanup cutoff |
| `SHARE_CREATE_RATE_LIMIT_PER_MINUTE` | `10` | create の共有DB固定窓 limit |
| `SHARE_DELETE_RATE_LIMIT_PER_MINUTE` | `10` | delete の共有DB固定窓 limit |
| `SHARE_GET_RATE_LIMIT_PER_MINUTE` | `120` | `/s/:id` の共有DB固定窓 limit |
| `TRUST_CLOUDFLARE_CONNECTING_IP` | `false` | internal Cloud Run ingress + HTTPS originでだけCloudflare client IPを信頼 |
| `SHARE_ARGON2_CONCURRENCY` | `2` | hash / verify 合計同時実行数 |
| `SHARE_ARGON2_QUEUE_LIMIT` | `8` | Argon2待機 request 上限 |
| `SHARE_ARGON2_WAIT_MS` | `250` | Argon2待機時間上限 |
| `SHARE_DAILY_CREATE_LIMIT` | `1000` | DB hard quota |
| `SHARE_ACTIVE_LIMIT` | `50000` | DB hard quota |
| `SHARE_STORAGE_LIMIT_BYTES` | `1073741824` | 保存payloadのlogical hard quota |
| `CLEANUP_BATCH_SIZE` | `500` | cleanup 1 transaction の最大件数 |
| `CLEANUP_MAX_BATCHES` | `1000` | 1 Job の安全上限 |

## 運用上の注意

- 公開 URL は secret ではない。認証や user account は存在しない。
- 外部 service が作った preview、cache、screenshot は削除や期限後も残る可能性がある。
- `SHARE_RESULTS_ENABLED=false` は既存共有も `404` にし、ローカル動画解析は継続させる。
- character / finding catalog を増やす場合は client、server、schema、公開文言、整合テストを同時に変更する。
- 保持期間や公開項目を変える場合は、共有URL生成前 disclosure も同じ release で更新する。

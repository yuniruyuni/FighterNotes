# 分析結果の共有と保存

最終確認: 2026-07-24

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

character ID、finding ID、assessment は client、server、database の全境界で allowlist にする。
未知 ID、重複 finding、非整数、round 数の不整合、上限超過、未知 field は拒否する。

## 公開しないデータ

次は server へ保存しない。

- 元動画、音声、decoded frame、clip、screenshot
- 動画ファイル名、size、lastModified
- frame number、timestamp、evidence range
- HP / Drive の frame-by-frame 系列
- 入力履歴、meter timeline、spatial observation
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

### localStorage

key prefix `fighter-notes:managed-share:v1:` に次だけを保存する。

- 共有 ID
- 削除コード
- 作成時刻と期限
- `自キャラ vs 相手キャラ` の label

期限切れ、不正形式、破損 record は一覧読込時に削除する。共有を削除した場合も該当 record を消す。

## Server storage

PostgreSQL は次の table を持つ。

| table | 内容 |
| --- | --- |
| `published_analyses` | ID、version、character、round、削除 hash、作成・期限 |
| `published_analysis_findings` | finding の順序、種別、assessment、件数、severity |
| `published_analysis_tactics` | 戦術統計 |
| `published_analysis_create_events` | UTC 日次 create quota 用の成功 event |

finding と tactics は parent 削除時に cascade delete する。create event は結果本体と独立させ、
同じ日に共有を削除しても日次 create 件数が減らないようにする。

schema version は現在1、presentation revision は1。server は既存データ表示のため
ruleset 3、4、5、6 を受理し、新規解析は ruleset 6 を生成する。

## 削除と期限

新規共有の削除 credential は Argon2id PHC string として保存する。
不正 ID、存在しない ID、誤った削除コード、削除済み ID は外部から区別できない応答にする。

正常な削除後、origin の新しい read は `404` になる。正常な公開 HTML は browser と edge で
最大15秒 cache できるため、削除直前の response がその範囲だけ見える可能性がある。
期限切れ、`404`、`429`、`503` は `no-store` とし、期限後の stale response は許可しない。

cleanup batch は次のどちらかを満たす parent row を削除する。

- `expires_at` を過ぎた
- `created_at + SHARE_RETENTION_DAYS` を過ぎた

1 batch の既定値は500件、最大1000 batch で停止する。全 parent cleanup が完了した場合だけ、
2 UTC 日より古い create quota event を削除する。repository は cleanup Job を定義するが、
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

application rate limiter は process 内の固定窓で、instance 間では共有しない。
これとは別に create transaction は PostgreSQL advisory lock の内側で hard quota を確認する。

| Hard quota | 既定値 |
| --- | ---: |
| UTC 日次作成成功数 | 1,000 |
| active parent row | 50,000 |
| 関係 table の使用量 | 1 GiB |

quota 到達時は fail closed とし、cleanup lag や濫用の原因を確認せず上限だけを緩めない。

## 設定

| 環境変数 | 既定値 | 用途 |
| --- | --- | --- |
| `SHARE_RESULTS_ENABLED` | `true` | `false` で create と公開 read を停止 |
| `PUBLIC_BASE_URL` | `https://fighter.yuniruyuni.net` | 共有 URL と許可 origin |
| `SHARE_RETENTION_DAYS` | `30` | 公開期限と cleanup cutoff |
| `SHARE_CREATE_RATE_LIMIT_PER_MINUTE` | `10` | create / delete の process-local limit |
| `SHARE_GET_RATE_LIMIT_PER_MINUTE` | `120` | `/s/:id` の process-local limit |
| `SHARE_DAILY_CREATE_LIMIT` | `1000` | DB hard quota |
| `SHARE_ACTIVE_LIMIT` | `50000` | DB hard quota |
| `SHARE_STORAGE_LIMIT_BYTES` | `1073741824` | DB hard quota |
| `CLEANUP_BATCH_SIZE` | `500` | cleanup 1 transaction の最大件数 |
| `CLEANUP_MAX_BATCHES` | `1000` | 1 Job の安全上限 |

## 運用上の注意

- 公開 URL は secret ではない。認証や user account は存在しない。
- 外部 service が作った preview、cache、screenshot は削除や期限後も残る可能性がある。
- `SHARE_RESULTS_ENABLED=false` は既存共有も `404` にし、ローカル動画解析は継続させる。
- character / finding catalog を増やす場合は client、server、schema、公開文言、整合テストを同時に変更する。
- 保持期間や公開項目を変える場合は、共有URL生成前 disclosure も同じ release で更新する。

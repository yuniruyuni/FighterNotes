# セキュリティ運用

最終確認: 2026-08-03

## 適用範囲

この文書は、共有結果を扱う server、PostgreSQL、Cloud Run service / Job、Cloudflare Access、
GitHub Actions の運用判断を扱う。browser 内の動画解析そのものは server へ動画を送らない。

次の3種類を混同しない。

- repository manifest: code review できる目標状態
- external infrastructure code: IAM、Cloudflare、DB login、Scheduler の宣言
- live state: 現在有効な policy、secret version、revision、Job 実行結果、alert

インシデント判断では3つを照合する。manifest に名前があるだけで、live 環境の deny、rotation、
監視設定が成立しているとはみなさない。

## データ分類

| データ | 扱い |
| --- | --- |
| 元動画、decoded frame、証拠画像 | browser 内限定。server request、DB、log へ送らない |
| 詳細な解析 report、timeline、入力履歴 | browser 内限定。公開 payload へ追加しない |
| 公開用集計 | public URL で誰でも閲覧可能。個人情報や自由文を入れない |
| 共有 ID | 公開 URL の識別子。認証 credential とみなさない |
| 削除コード | secret。結果画面、localStorage、create / delete request だけで扱い、DB には Argon2id hash だけを置く |
| DB / Cloudflare / GitHub credential | secret manager または GitHub secret だけで管理する |

共有の closed schema と保持期間は [sharing.md](./sharing.md) を参照する。

## Trust boundary

| Identity | 必要な責務 | 持たせない権限 |
| --- | --- | --- |
| GitHub builder | source checkout、Artifact Registry / GHCR への image push | Cloud Run 更新、production secret access、DB access |
| GitHub deployer | Cloud Run service / Job の置換、migration Job 実行 | image build 用の常設 credential、DB password の直接読取 |
| `fighter-runtime` | runtime 用 DB password と専用 Cloudflare Access token の参照 | DB owner、DDL、migration / cleanup token |
| `fighter-migration` | owner password と専用 DB Access token、schema apply | runtime / cleanup token、application traffic |
| `fighter-cleanup` | app role password と専用 DB Access token、Job 実行 | DB owner、DDL、runtime / migration token |
| Scheduler identity | `fighter-cleanup` の実行 | service 更新、secret 読取、他 Job 実行 |

application role `fighter_app` の権限は `schema/` の明示的な `GRANT` に限定する。
role の login policy と password は外部 DB infrastructure が所有する。

GitHubのbuilder secretはrepository / organization scope、deployer secretは`production` environment scopeに
分離する。reusable build workflowへ全secretを継承せず、builder identityではCloud Run service / Jobの
参照・更新を拒否し、deployer identityではArtifact Registry / GHCRへのpushを拒否する。

third-party Actionとcontainerはreview済みcommit / manifest digestへ固定する。更新時はversion commentと
digestを同じPRで変更し、Renovateの差分をupstream releaseと照合する。緊急時も`latest`やmajor tagへ
一時的に戻さず、対象digestを明示する。

## Log 方針

server log に次を追加しない。

- 削除コード、hash、DB password、Cloudflare token
- 共有 payload、共有 ID、公開 URL
- raw IP address、forwarded header 全体、request body
- 元ファイル名、動画 metadata、frame、詳細 report

現在の application log は create / delete の成否、quota 理由、rate limit、cleanup 件数、
分類済み error を中心にする。調査用の一時 log でも secret や payload を出さず、必要なら件数、
固定 enum、処理時間など低 cardinality の値を使う。

## 日常確認

この repository は alert policy を作成しない。運用環境では少なくとも次を監視し、
live alert の有無と通知先を定期的に棚卸しする。

| Signal | 確認内容 |
| --- | --- |
| Service availability | `/health` の失敗、5xx、revision 起動失敗、latency |
| DB path | `/ready`、共有 read / create / delete、connection / statement / lock timeout |
| Abuse control | bucket別429/503、Argon2 capacity、daily / active / storage quota 到達理由 |
| Cleanup | Scheduler 最終成功時刻、Job exit、削除件数、batch 安全上限、backlog |
| Capacity | logical bytes、active row数、物理relation size、DB connection、Cloud Run instance数 |
| Identity | IAM policy 変更、Secret Manager access、Workload Identity の失敗 |
| DB tunnel | Cloudflare Access deny、token 認証失敗、sidecar startup probe 失敗 |
| Delivery | 対象CI run / SHA、production environment実行者、image digest、migration / cleanup / deployの結果 |

`/health` は DB を query しない。`/ready`はruntime app roleのread-only queryで利用列、default、
constraint、PK/FK、cleanup index、必要grantのcatalog contractを確認する。どちらも`no-store`で、失敗時も
DB host、credential、schema差分をresponseへ出さない。`/ready`成功後もclosed schemaのtest dataで
read / create / deleteを確認し、実利用者の共有IDや削除コードをprobeに使わない。

## 共有の緊急停止

濫用、意図しない公開項目、共有 read の脆弱性が疑われる場合は、Web service の
`SHARE_RESULTS_ENABLED` を `false` にする。

```bash
gcloud run services update fighter \
  --region us-west1 \
  --project "$GCP_PROJECT_ID" \
  --container app \
  --update-env-vars SHARE_RESULTS_ENABLED=false
```

この状態では新しい共有作成と `/s/:id` の取得が無効になり、既存 URL は `404` になる。
削除 API は継続する。browser 内の動画解析とローカル履歴も継続する。

停止後は次を行う。

1. 新 revision と環境変数、公開 URL の `404` を確認する。
2. cache purge が必要か Cloudflare の live 設定を確認する。
3. 原因、影響した ruleset / schema / revision、公開期間を特定する。
4. 修正と再開条件を review し、`cloudrun.yaml` も同じ状態へ同期する。
5. 再開時に create、GET、delete、期限、cache header を一式確認する。

公開先や crawler が保持した preview、screenshot、cache は、DB 削除や緊急停止だけでは回収できない。

## Credential 漏えい

1. 漏えい候補の credential と consumer を1つに絞る。
2. audit log で利用時刻、source、対象 resource を確認する。
3. 新 credential / secret version を作り、その consumer だけを更新する。
4. 新 revision または Job を起動し、正常系と deny 系を確認する。
5. 旧 credential を disable する。
6. rollback window 後に旧 version を破棄し、原因を修正する。

`secretKeyRef.key: latest` を使う workload は、新 version 作成だけでは既存 instance が切り替わらない。
service は新 revision、Job は定義更新または再起動で新値を読むことを確認する。

DB app password、DB owner password、runtime / migration / cleanup の Cloudflare token を一度に
rotation しない。owner credential を application の復旧用に配布せず、default service account や
共有 token へ一時的に権限を広げない。

## DB incident

不正 query、権限逸脱、data corruption が疑われる場合は次の順で扱う。

1. application revision、Job execution、DB session、Cloudflare Access event の時刻を揃える。
2. 影響する workload の共有停止、Job 停止、token disable の最小 containment を行う。
3. `fighter_app` の grant と実際の role membership を比較する。
4. table row、relation size、create event、schema version を read-only に確認する。
5. backup / point-in-time recovery の復元先を別 DB に作り、整合性を検証する。
6. 復旧後に app role の create / read / delete、owner role の migration、deny test を分けて行う。

production DB 上で原因調査と同時に手動 DDL や大量削除を行わない。rollback が schema を伴う場合は
[DEPLOY.md](./DEPLOY.md) の互換性条件を先に確認する。

## Cleanup failure

最初に Cloud Scheduler の対象、identity、最終実行、Cloud Run Job の revision と image を確認する。
次に sidecar、DB timeout、batch 安全上限、期限切れ row の backlog を確認する。

```bash
gcloud run jobs execute fighter-cleanup \
  --region us-west1 \
  --project "$GCP_PROJECT_ID" \
  --wait
```

手動実行を繰り返す前に、1回の Job が一部削除後に失敗したのか、接続前に失敗したのかを log で分ける。
cleanup は短い batch と `ON DELETE CASCADE` を使うため再実行可能だが、安全上限の引き上げは DB load と
quota event prune の順序を確認してから行う。

### Quotaの予兆と回復

外部監視では`SHARE_ACTIVE_LIMIT`と`SHARE_STORAGE_LIMIT_BYTES`に対し80%でwarning、90%でcritical、
100%でcreate停止として扱う。少なくとも次のread-only query相当を収集し、logical quotaと物理容量を
別signalにする。

```sql
SELECT
  count(*) FILTER (WHERE expires_at > clock_timestamp()) AS active_rows,
  coalesce(sum(logical_size_bytes), 0) AS logical_bytes,
  count(*) FILTER (WHERE expires_at <= clock_timestamp()) AS expired_backlog
FROM published_analyses;
```

閾値到達時はSchedulerの最終成功とJob logを確認し、cleanupを1回実行する。`logical_bytes`はDELETEの
commit直後に減るため、その値と新規createを再確認する。物理relation fileが縮まらなくてもquota回復に
`VACUUM FULL`は不要である。DB diskの警告が別途残る場合はautovacuum状況を確認し、必要ならonlineの
`VACUUM (ANALYZE)`で再利用と統計更新を促す。blocking maintenanceはDB運用手順として別に計画する。

## 定期棚卸し

少なくとも四半期ごと、または IAM / schema / network 変更後に次を確認する。

- builder が deploy できず、deployer が image push や secret 読取をできないこと
- builder jobからproduction environmentのdeployer secretを参照できないこと
- 対象SHAのCI失敗時にreleaseが起動せず、開始済みmigrationが後続pushでcancelされないこと
- runtime / cleanup が DDL できず、migration だけが schema apply できること
- 各 Cloudflare token が別 workload から利用できないこと
- Scheduler identity が cleanup Job 以外を起動できないこと
- sharing disable、期限切れ、誤った削除コード、quota 超過が fail closed になること
- DB backup の復元 test と、旧 image / 新 schema の rollback 互換性
- secret version、不要な role binding、古い image、失敗した Job execution の棚卸し
- RenovateがActions、pgschema、PostgreSQL service、Cloud Run sidecarの更新PRを作成できること

supply chain と CI の未実装項目は [DEPLOY.md](./DEPLOY.md) の残余リスクを参照する。

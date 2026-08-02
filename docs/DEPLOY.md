# デプロイ

最終確認: 2026-07-18

## 対象と正本

この文書は、この repository にある GitHub Actions と Cloud Run manifest の release 手順を扱う。

- application / migration image: `Dockerfile`、`Dockerfile.migration`
- CI/CD: `.github/workflows/`
- Cloud Run service / Job: `cloudrun.yaml`、`cloudrun-job.yaml`、`cloudrun-cleanup-job.yaml`
- schema: `schema/`、`.pgschemaignore`、`bin/migrate.sh`

Cloudflare、PostgreSQL login、Secret Manager、IAM binding、cleanup の Scheduler は
`yuniruyuni/yuniruyuni.net` 側の Terraform / NixOS と live inventory が正本である。
この repository の service account 名や secret 参照は接続契約であり、実在や権限付与の証明ではない。

## Release 構成

| Workload | Manifest | Image | DB role | Service account |
| --- | --- | --- | --- | --- |
| Web service `fighter` | `cloudrun.yaml` | `fighter` | `fighter_app` | `fighter-runtime` |
| Migration Job `fighter-migration` | `cloudrun-job.yaml` | `fighter-migration` | DB owner `fighter` | `fighter-migration` |
| Cleanup Job `fighter-cleanup` | `cloudrun-cleanup-job.yaml` | `fighter` | `fighter_app` | `fighter-cleanup` |

各 workload は `cloudflared access tcp` sidecar を通して `db.yuniruyuni.net` へ接続する。
runtime と cleanup は DML 用 password、migration だけが DDL 用 owner password を使う。

Web service は internal ingress、最大 2 instance、container concurrency 80、timeout 60 秒である。
外部公開経路は Cloud Run manifest の外側にある。

## GitHub 設定

repository または production environment に次の secret が必要である。値は文書や log に残さない。

- `GCP_PROJECT_ID`
- `GCP_BUILDER_WORKLOAD_IDENTITY_PROVIDER`
- `GCP_BUILDER_SERVICE_ACCOUNT`
- `GCP_DEPLOYER_WORKLOAD_IDENTITY_PROVIDER`
- `GCP_DEPLOYER_SERVICE_ACCOUNT`

builder と deployer は別の Workload Identity / service account を使う。builder は image push、
deployer は Cloud Run service / Job の置換と Job 実行に必要な最小権限だけを持たせる。
実際の IAM binding は外部 infrastructure repository と GCP live policy で確認する。

## CI gate

`ci.yml` は `main` / `develop` への push と pull request で次を実行する。

1. PostgreSQL 18 を起動し、Rust workspace test を実行する。
2. WASM と client / server を build する。
3. TypeScript、Biome、client / server test を実行する。
4. 別 Job で `bun audit` を実行する。
5. application と migration image を build し、application の `/health` を smoke test する。

`/health` は process の HTTP 応答だけを確認し、DB query は行わない。DB を含む read / create / delete は
PostgreSQL integration test と release 後の共有経路で別に確認する。

## Schema 変更

schema 関連 path の pull request では `schema-plan.yml` が次を行う。

1. PostgreSQL 18 に `fighter_app` role を用意する。
2. `origin/main` の schema を baseline として適用する。
3. pull request の `schema/main.sql` に対する `pgschema plan` を生成する。
4. plan を pull request comment へ反映する。

`DROP`、`GRANT`、`REVOKE`、privilege 変更を含む plan は特に確認する。`pgschema plan` の comment は
成功・失敗のどちらでも更新するが、plan command が失敗した check は comment 投稿後に必ず失敗する。
branch protection では `Schema Plan / plan` を必須 check にする。comment は review 補助であり、
データ移行、lock 時間、rollback 可能性の判断を代替しない。

migration は application より先に production DB へ適用される。したがって schema 変更は、少なくとも
直前の application image と新しい image の両方から利用できる後方互換な段階に分ける。
列削除や制約強化は、利用コードの release と rollback window が終わった後の別 release にする。

## 自動デプロイ

`main` への push で `deploy.yml` が動く。image tag は
`<git-sha>-<run-id>-<run-attempt>` で、再実行を含め一意になる。

1. application image と migration image を build する。
2. 両 image を Artifact Registry と archive 用 GHCR へ push する。
3. migration Job manifest を対象 tag で置換し、Job を同期実行する。
4. cleanup Job manifest を同じ application tag で置換する。
5. Web service manifest を同じ application tag で置換する。
6. `https://fighter.yuniruyuni.net/health` を smoke test する。

Job と service の置換は GitHub `production` environment の deployer identity で行う。
release 中は migration の完了前に Web service を更新しない。

## Release 後確認

最低限、次を browser または HTTP client で確認する。

- `/` が静的 asset と WASM を読み込む。
- `/health` が `200` と `{ "status": "ok" }` を返す。
- 実動画の解析が完了し、結果画面を表示できる。
- 新規共有を作成し、発行された `/s/:id` を別 session で取得できる。
- `/manage` と `/manage/:id` が表示でき、削除コードで共有を削除できる。
- 削除後と期限切れの `/s/:id` が `404` になり、cache されない。

共有 payload に動画、画像、ファイル名、詳細レポートが含まれないことも Network panel で確認する。

## Cleanup

deploy workflow は `fighter-cleanup` Job を install / update するが、定期実行 Scheduler は作らない。
production では外部 infrastructure repository と Cloud Scheduler live inventory で、対象 Job、region、
実行 identity、頻度、最終成功時刻を確認する。

手動実行は次のとおり。

```bash
gcloud run jobs execute fighter-cleanup \
  --region us-west1 \
  --project "$GCP_PROJECT_ID" \
  --wait
```

成功 log は `expired`、`quota_events`、`batches` を出力する。batch 安全上限に達した場合は失敗終了し、
quota event の prune へ進まない。原因と backlog を確認してから設定または実装を変更する。

## Rollback

1. 直前に正常だった application image の一意 tag または digest を特定する。
2. その image を指定して `fighter` service を更新する。
3. `/health` だけでなく共有 read / create / delete と browser 解析を確認する。
4. cleanup Job も不具合のある application image を参照している場合は、正常 image へ戻す。

```bash
gcloud run services update fighter \
  --region us-west1 \
  --project "$GCP_PROJECT_ID" \
  --container app \
  --image "$PREVIOUS_IMAGE"
```

schema は application より先に更新済みである。旧 image が新 schema と互換でない場合は単純 rollback せず、
forward fix または検証済み backup restore を選ぶ。破壊的 DDL をその場の手動 SQL で戻さない。

## Repository から確認できる残余リスク

- GitHub Action は commit SHA ではなく major tag を参照している。
- `pgschema` は version tag、`cloudflared` sidecar は `latest` を参照している。
- deploy manifest は image digest ではなく一意 tag を使用する。
- CI の dependency 検査は `bun audit` が中心で、Rust audit、secret scan、container scan はない。
- SBOM、provenance、署名、attestation の生成・検証はない。
- browser E2E と visual regression は release gate にない。
- edge rate limit、audit log、IAM、Scheduler、DB backup の live 状態はこの repository では証明できない。

これらを変更した場合は [security-operations.md](./security-operations.md) と外部 infrastructure の
運用手順も同時に更新する。

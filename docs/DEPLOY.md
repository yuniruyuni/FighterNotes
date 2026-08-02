# デプロイ

最終確認: 2026-08-03

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
外部公開経路は Cloud Run manifest の外側にある。`CF-Connecting-IP` をrate-limit keyとして
信頼するのは、internal ingressを維持し、Cloudflare経路だけがoriginへ到達できる構成に限る。
`TRUST_CLOUDFLARE_CONNECTING_IP=true` のreleaseではlive ingressとCloudflare tunnel / routeを
監査し、Cloud Runへの別経路が無いことを確認する。

## GitHub 設定

値は文書や log に残さず、consumer ごとに次の scope へ分ける。

| Secret | Scope |
| --- | --- |
| `GCP_PROJECT_ID` | repository または organization |
| `GCP_BUILDER_WORKLOAD_IDENTITY_PROVIDER` | repository または organization |
| `GCP_BUILDER_SERVICE_ACCOUNT` | repository または organization |
| `GCP_DEPLOYER_WORKLOAD_IDENTITY_PROVIDER` | `production` environment のみ |
| `GCP_DEPLOYER_SERVICE_ACCOUNT` | `production` environment のみ |

reusable build workflow は `workflow_call.secrets` で builder 用3項目だけを受け取り、caller も個別に渡す。
`secrets: inherit` は使わない。deployer secret は `production` environment の直列 release job だけから
参照する。builder は image push、deployer は Cloud Run service / Job の置換と Job 実行に必要な最小権限
だけを持たせる。実際の IAM binding は外部 infrastructure repository と GCP live policy で確認する。

## CI gate

`ci.yml` は `main` / `develop` への push と pull request で次を実行する。

1. PostgreSQL 18 を起動し、Rust workspace test を実行する。
2. WASM と client / server を build する。
3. TypeScript、Biome、client / server test を実行する。
4. 別 Job で `bun audit` を実行する。
5. application と migration image を build し、application の `/health` を smoke test する。

`/health` は process の HTTP 応答だけを確認し、DB query は行わない。DB を含む read / create / delete は
PostgreSQL integration testで確認する。production deployはさらに`/ready`でruntime app roleからの
read-only query、利用列の型/nullability、critical constraint/default、PK/FK/indexと最小grantを
短いtimeout内に確認する。smoke testのcurl自体にも接続・全体timeoutと有限retryを設定する。

`deploy.yml` は `main` pushを直接契機にせず、同じSHAに対する `CI` workflowの `push` runが成功した
`workflow_run` だけを受け付ける。branch protectionでは `CI / test`、`CI / security`、`CI / docker` と、
全pull requestで `Schema Plan / plan` を必須にする。GitHubのlive branch ruleはrepository外の状態なので、
設定変更後と四半期棚卸し時に実在を確認する。

`scripts/release-workflows.test.ts` は全workflowのthird-party Actionが40桁commit SHAとversion commentを
持つこと、service / release containerがdigest固定されていること、schema planとreleaseの安全条件を
検査する。`CI / test` がこのcontract testを直接実行する。

## Schema 変更

`schema-plan.yml` は全pull requestで同じ `Schema Plan / plan` jobを作成し、PRのbase SHAとhead SHAの
差分から `schema/**`、`.pgschemaignore`、`Dockerfile.migration`、`bin/migrate.sh` の変更有無を判定する。
対象pathがなければ成功no-opとし、PostgreSQLやmigration toolchainは起動しない。対象pathがある場合だけ
次を行う。

1. PostgreSQL 18 に `fighter_app` role を用意する。
2. pull requestのbase SHAにあるschemaをbaselineとして適用する。
3. pull request の `schema/main.sql` に対する `pgschema plan` を生成する。
4. planと注意表示をjob summaryへ残し、可能ならpull request commentも更新する。
5. 保存したplan commandの終了コードを検査し、非0、出力未生成、終了コード未記録を失敗にする。

`DROP`、`GRANT`、`REVOKE`、privilege 変更を含む plan は特に確認する。public forkなどで
`GITHUB_TOKEN`がread-onlyの場合、comment投稿失敗はwarningとして扱い、planはjob summaryで確認する。
commentはbest-effortのreview補助であり、その成否でplan/enforce結果を上書きしない。plan commandや
enforceが失敗したcheckは、commentの成否にかかわらず必ず失敗する。branch protectionでは全PRに対して
`Schema Plan / plan`を必須checkにする。これはデータ移行、lock時間、rollback可能性の判断を代替しない。

migration は application より先に production DB へ適用される。したがって schema 変更は、少なくとも
直前の application image と新しい image の両方から利用できる後方互換な段階に分ける。
列削除や制約強化は、利用コードの release と rollback window が終わった後の別 release にする。

## 自動デプロイ

`main` pushの `CI` workflowが成功すると `deploy.yml` が動く。release対象はCI runの40桁SHAへ固定し、
image tagは `<git-sha>-<deploy-run-id>-<run-attempt>` で再実行を含め一意にする。配置時にはpush応答から
得たArtifact Registry digestへ置き換え、tagの再利用には依存しない。

1. application image と migration image を build する。
2. 両 image を Artifact Registry と archive 用 GHCR へ push する。
3. production release lockを取得し、対象SHAが現在の`main`であることを再確認する。
4. migration Job manifest をmigration image digestで置換し、Jobを同期実行する。
5. cleanup Job manifest をapplication image digestで置換する。
6. Web service manifest を同じapplication image digestで置換する。
7. `https://fighter.yuniruyuni.net/health`と`/ready`をsmoke testする。

build jobは並行できるが、migration、cleanup、service更新、smoke testは1つの
`fighter-production-release` concurrency groupで直列実行し、後続pushから開始済みreleaseをcancelしない。
lock待機中に古くなったSHAはmigration前に見送り、現在の`main`を後から古いrevisionで上書きしない。
Jobとserviceの置換はGitHub `production` environmentのdeployer identityで行う。job summaryには対象SHA、
両image digest、migration / cleanup / service / smokeの各結果を残す。

## Immutable dependency の更新

GitHub Actionsは40桁commit SHAで固定し、review時に追跡できるrelease versionを同じ行のcommentへ残す。
GitHub ActionsのPostgreSQL service、`Dockerfile.migration`の`pgschema`、Cloud Runの`cloudflared`は
`version@sha256:digest`で固定する。application / migration imageもbuild後のArtifact Registry digestで
配置するため、同じrepository commitの再実行でmutable tagをproductionへ持ち込まない。

`renovate.json`はActions、Dockerfile、workflow service imageに加え、custom managerで3つのCloud Run
manifestの`cloudflared`を更新対象にする。Renovate GitHub Appまたは同等runnerがrepositoryで有効であることは
live設定で確認する。更新PRではversionとdigestの両方、upstream release note、schema plan、CI、Cloud Run
sidecar startup、cleanup smokeを確認する。緊急security updateでもtagだけへ戻さず、検証したdigestを直接更新する。

rollbackはrelease summaryとCloud Run revisionに記録されたapplication / sidecar digestを使う。
古いmutable tagからdigestを再解決してはならない。

## Release 後確認

最低限、次を browser または HTTP client で確認する。

- `/` が静的 asset と WASM を読み込む。
- `/health` が `200` と `{ "status": "ok" }` を返す。
- `/ready` が `200` と `{ "status": "ready" }` を返す。
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

成功 log は `expired`、`rate_limits`、`quota_events`、`batches` を出力する。batch安全上限に達した場合は失敗終了し、
quota event の prune へ進まない。原因と backlog を確認してから設定または実装を変更する。

10,000件backlogのintegration testは期限用indexと2 workerの全batch処理を検証する。さらに、
active 100,000件よりexpires_at順で後方にあるretention対象をcreated_at専用indexから取得する病的分布も
30秒未満に制限する。production Jobのtimeoutは600秒だが、release後は実データのrow幅、cascade対象、
DB負荷を含む実行時間と`EXPLAIN (ANALYZE, BUFFERS)`を別途確認する。

## Rollback

1. 直前に正常だった application image の digest をrelease summaryまたはCloud Run revisionから特定する。
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

- CI の dependency 検査は `bun audit` が中心で、Rust audit、secret scan、container scan はない。
- SBOM、provenance、署名、attestation の生成・検証はない。
- browser E2E と visual regression は release gate にない。
- edge rate limit、audit log、IAM、Scheduler、DB backup の live 状態はこの repository では証明できない。

これらを変更した場合は [security-operations.md](./security-operations.md) と外部 infrastructure の
運用手順も同時に更新する。

# デプロイ

最終確認: 2026-08-22

## 対象と正本

この文書は、この repository にある GitHub Actions と yunirun 宣言による release 手順を扱う。

- application / migration image: `Dockerfile`、`Dockerfile.migration`
- CI/CD: `.github/workflows/`
- デプロイ宣言: `yunirun.jsonc`
- VPS の既知ホスト鍵: `ssh/known_hosts`
- schema: `schema/`、`.pgschemaignore`、`bin/migrate.sh`

このアプリは Cloud Run から、VPS 上のデプロイシステム yunirun へ移した。Cloud Run、
Cloud Scheduler、Artifact Registry、Workload Identity、Secret Manager は使わない。

アプリの取り込み宣言、uid と port の割り当て、PostgreSQL の login、HAProxy、Cloudflare の
tunnel と route、agenix の秘密は `yuniruyuni/yuniruyuni.net` 側の NixOS 設定と live inventory が
正本である。この repository の宣言は接続契約であり、実在や権限付与の証明ではない。

## Release 構成

| Workload | 起動 | Image | DB role |
| --- | --- | --- | --- |
| Web app `fighter` | blue/green の2コンテナを片方ずつ入れ替える | `ghcr.io/yuniruyuni/fighter` | `fighter_app` |
| migration | 入れ替えの前に1回だけ実行する | `ghcr.io/yuniruyuni/fighter-migration` | owner `fighter` |
| cleanup | systemd timer (`yunirun.jsonc` の `"schedule": "02:23"`) | `ghcr.io/yuniruyuni/fighter` を `--batch=cleanup` で起動 | `fighter_app` |

DB 名とロールは `fighter`、owner の `fighter`、app の `fighter_app` である。各 workload は
同じ VPS 上の PostgreSQL へ Unix socket で直結する。Cloud Run 時代の `cloudflared access tcp`
sidecar と DB 用の Cloudflare Access tunnel は不要になったので使わない。

DB password は yunirun がホスト鍵と管理者鍵で暗号化して保持する。runtime と cleanup へは
app role の password だけを渡し、owner password は root だけが読める env ファイルに置く。
migration は root 側の unit として実行するので、アプリのユーザから owner の資格情報へは届かない。

公開経路は Cloudflare tunnel から VPS の HAProxy (frontend `127.0.0.1:8260`) を通り、
blue/green のコンテナへ入る。コンテナは loopback にだけ publish するので、この経路以外から
origin へ到達できない。`CF-Connecting-IP` を rate-limit key として信頼できるのはこの構成に限る。
`TRUST_CLOUDFLARE_CONNECTING_IP=true` のまま公開経路を変更する場合は、HAProxy の bind、
tunnel の route、コンテナの publish 先を監査し、別経路が無いことを確認する。

## GitHub 設定

deploy 用の長期資格情報をこの repository に置かない。

| 用途 | 使うもの |
| --- | --- |
| GHCR への image push | `GITHUB_TOKEN` (`packages: write`) |
| VPS への SSH | OIDC token (`id-token: write`) から opkssh が発行する短命の SSH 証明書 |
| VPS からの GHCR pull | deploy job の `GITHUB_TOKEN` を stdin で渡す。job 終了とともに失効する |

長期の SSH 秘密鍵も、GCP の Workload Identity provider / service account も使わない。誰が
deploy できるかは VPS 側の取り込み宣言と opkssh の認可先が決めるので、repository secret を
足しても deploy 権限は増えない。

deploy job に `environment:` を付けない。付けると OIDC の `sub` が `...:environment:<name>` へ
変わり、VPS 側の認可と一致しなくなって ssh に入れない。GHCR token は argv ではなく stdin で
渡す。argv に載せると `ps` から見える。どちらも `scripts/release-workflows.test.ts` で固定してある。

opkssh の version は VPS 側と揃える。証明書の principals の扱いが version で変わり、
食い違うと sshd 側で拒否される。

## CI gate

`ci.yml` は `main` / `develop` への push と pull request で次を実行する。

1. PostgreSQL 18 を起動し、Rust workspace test を実行する。
2. WASM と client / server を build する。
3. TypeScript、Biome、client / server test を実行する。
4. 別 Job で `bun audit` を実行する。
5. 別 Job で application と migration image を build し、application の `/health` を smoke test する。

image build は test の完了を待たない。同じ commit を別々に検証するだけで順序に意味はなく、
待たせると build 時間がそのまま CI 全体に積み上がる。test が落ちた場合は build 1 回分を無駄に払う。

`cargo install` する CLI、Rust の build 成果物、image の layer は cache から復元する。cache は
`main` の push でだけ保存する。pull request で作った cache はその pull request からしか見えず、
他へ再利用されないまま容量を占める。cache は成果物の再現手段であり、release の入力ではない。
release が配置するのは、その commit SHA を tag として GHCR へ push した image である。

`/health` は process の HTTP 応答だけを確認し、DB query は行わない。DB を含む read / create / delete は
PostgreSQL integration testで確認する。`/ready`はruntime app roleからのread-only query、利用列の
型/nullability、critical constraint/default、PK/FK/indexと最小grantを短いtimeout内に確認する。

`deploy-yunirun.yml` は `main` への push で走り、同じ push の `CI` workflow とは並行する。CI の成功は
デプロイの前提になっていない。壊れた commit を `main` へ入れないことは pull request 側の必須 check で
担保する。branch protection では `CI / test`、`CI / security`、`CI / docker` と、全 pull request で
`Schema Plan / plan` を必須にする。GitHub の live branch rule は repository 外の状態なので、
設定変更後と四半期棚卸し時に実在を確認する。

`scripts/release-workflows.test.ts` は全 workflow の third-party Action が40桁 commit SHA と version
comment を持つこと、workflow の service / container image が digest 固定であること、schema plan の
安全条件、そしてデプロイ側の契約 (中断しない、`/health` を確認する、`environment:` を付けない、
token を stdin で渡す、`yunirun.jsonc` に cleanup の schedule と Cloudflare client IP の前提が
揃っている) を検査する。`CI / test` がこの contract test を直接実行する。

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

migration は blue/green の入れ替えより先に実行されるので、schema は必ず新しい application より先に
production DB へ入る。したがって schema 変更は、少なくとも直前の application image と新しい image の
両方から利用できる後方互換な段階に分ける。列削除や制約強化は、利用コードの release と rollback window が
終わった後の別 release にする。

## 自動デプロイ

`main` への push、または `main` に対する `workflow_dispatch` で `deploy-yunirun.yml` が走る。
VPS 側の認可は `main` の ref に限ってあるので、別 branch から起動しても ssh に入れない。

1. `build-ghcr.yml` を2回呼び、`Dockerfile` と `Dockerfile.migration` から image を build して
   `ghcr.io/yuniruyuni/fighter` と `ghcr.io/yuniruyuni/fighter-migration` へ push する。tag は
   その commit の SHA そのものである。
2. deploy job が cloudflared と opkssh を入れ、`ssh/known_hosts` を配置し、`opkssh login github` で
   OIDC token から短命の SSH 証明書を受け取る。
3. `yunirun-fighter@ssh.yuniruyuni.net` へ ssh し、`yunirun deploy <sha>` を実行する。GHCR token と
   `yunirun.jsonc` の内容は stdin の JSON で渡す。VPS が GitHub を取りに行かないので、manifest 取得用の
   資格情報を VPS 側へ置かずに済む。
4. `https://fighter.yuniruyuni.net/health` を smoke test する。

VPS 側の `yunirun deploy` は次の順で進む。

1. 受け取った `yunirun.jsonc` を保存し、宣言を反映する (unit と HAProxy 設定の書き直し)。
2. GHCR へ login し、application と migration の image を pull する。
3. migration を root 側の unit で1回実行する。
4. blue、green の順に片方ずつ再起動し、それぞれが healthy になるのを待つ。

順序には理由がある。宣言の反映が後だと古い unit のまま起動する。migration が入れ替えより後だと
新旧が食い違う。片側が healthy になる前にもう片側を落とすと無停止でなくなる。migration が失敗した
時点で停止し、稼働中のコンテナには触れない。

concurrency group は `deploy-yunirun-<ref>` で、`cancel-in-progress: false` にする。途中で中断すると
blue/green の入れ替えが片側だけ終わった状態で止まりうる。

## 依存の固定

GitHub Actions は40桁 commit SHA で固定し、review 時に追跡できる release version を同じ行の comment へ
残す。GitHub Actions の PostgreSQL service image と `Dockerfile` / `Dockerfile.migration` の `FROM` は
`version@sha256:digest` で固定する。opkssh の binary は version と SHA-256 を workflow 内で照合する。

application と migration の image は commit SHA を tag にして GHCR へ push し、同じ SHA を指して deploy する。
digest 指定ではないので、同じ SHA で workflow を再実行すると build し直した image が同じ tag へ入る。

`renovate.json` は Actions、Dockerfile、workflow の service image を更新対象にする。Renovate GitHub App
または同等 runner が repository で有効であることは live 設定で確認する。更新PRでは version と digest の
両方、upstream release note、schema plan、CI を確認する。緊急security updateでも tag だけへ戻さず、
検証した digest を直接更新する。

## Release 後確認

最低限、次を browser または HTTP client で確認する。

- `/` が静的 asset と WASM を読み込む。
- `/health` が `200` と `{ "status": "ok" }` を返す。
- `/ready` が `200` と `{ "status": "ready" }` を返す。
- 実動画の解析が完了し、結果画面を表示できる。
- 新規共有を作成し、発行された `/s/:id` を別 session で取得できる。
- `/manage` と `/manage/:id` が表示でき、削除コードで共有を削除できる。
- 削除後と期限切れの `/s/:id` が `404` になり、cache されない。

共有 payload に動画、画像、ファイル名、詳細レポート、frame/input、SA/CAの正確なdamage値と
最終gauge量が含まれず、ruleset v9以降ではavailability付き集計だけが含まれることもNetwork panelで確認する。

VPS 上での確認は、アプリのユーザ (`yunirun-fighter`) の systemd user unit を見る。

```bash
systemctl --user status fighter-blue.service fighter-green.service
journalctl --user -u fighter-blue.service -n 100
```

blue/green の片方だけが動いている状態は、入れ替えの途中か、片側が healthy にならずに止まったことを
意味する。HAProxy はヘルスチェックで振り分けを追従するので、片系でも公開は続く。

## Cleanup

cleanup は `yunirun.jsonc` の `"schedule": "02:23"` から systemd timer として作られる。旧 Cloud Scheduler の
`23 2 * * *` (JST) を移したものである。timer は停止中に予定時刻を過ぎていれば起動後に一度実行し、
複数アプリが同時刻に集中しないよう最大15分の遅延を入れる。実行時刻はその分ずれる。

```bash
systemctl --user list-timers fighter-cleanup.timer
systemctl --user start fighter-cleanup.service
journalctl --user -u fighter-cleanup.service -n 200
```

成功 log は `expired`、`rate_limits`、`quota_events`、`batches` を出力する。batch安全上限に達した場合は失敗終了し、
quota event の prune へ進まない。原因と backlog を確認してから設定または実装を変更する。

cleanup は application 本体と同じ image を `--batch=cleanup` で起動し、`yunirun.jsonc` で `PGPOOL_MAX=1`、
`PG_STATEMENT_TIMEOUT_MS=30000` を上書きする。1本の長い処理なので接続は1つで足り、文が長いぶん
statement の上限を伸ばす。

10,000件backlogのintegration testは期限用indexと2 workerの全batch処理を検証する。さらに、
active 100,000件よりexpires_at順で後方にあるretention対象をcreated_at専用indexから取得する病的分布も
30秒未満に制限する。release後は実データのrow幅、cascade対象、DB負荷を含む実行時間と
`EXPLAIN (ANALYZE, BUFFERS)`を別途確認する。

## Rollback

1. 直前に正常だった commit を特定する。
2. その状態へ戻す revert commit を `main` へ入れる。deploy は `main` の push で走り、VPS 側の認可も
   `main` の ref に限られているので、過去の commit や別 branch を選んで deploy する経路は無い。
3. `/health` だけでなく共有 read / create / delete と browser 解析を確認する。

cleanup はアプリ本体と同じ image を使うので、application を戻せば cleanup も同じ image に戻る。

rollback でも migration は旧 commit の schema で走り直す。後方互換でない schema 変更を含む release を
戻すと、DDL の巻き戻しが production DB に対して実行される。旧 image が新 schema と互換でない場合や、
戻す側の plan が破壊的な場合は単純 rollback せず、forward fix または検証済み backup restore を選ぶ。
破壊的 DDL をその場の手動 SQL で戻さない。

## Repository から確認できる残余リスク

- deploy は同じ push の CI 成功を待たない。壊れた commit を `main` へ入れないことは pull request 側の
  必須 check に依存する。
- image は commit SHA tag で指しており、digest 固定ではない。
- CI の dependency 検査は `bun audit` が中心で、Rust audit、secret scan、container scan はない。
- SBOM、provenance、署名、attestation の生成・検証はない。
- browser E2E と visual regression は release gate にない。
- edge rate limit、VPS 上の unit / timer / 秘密 / DB backup の live 状態は、この repository では証明できない。

これらを変更した場合は [security-operations.md](./security-operations.md) と外部 infrastructure の
運用手順も同時に更新する。

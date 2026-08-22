# セキュリティ運用

最終確認: 2026-08-22

## 適用範囲

この文書は、共有結果を扱う server、PostgreSQL、VPS 上の yunirun workload、Cloudflare、
GitHub Actions の運用判断を扱う。browser 内の動画解析そのものは server へ動画を送らない。

次の3種類を混同しない。

- repository manifest: code review できる目標状態 (`yunirun.jsonc`、`schema/`、workflow)
- external infrastructure code: アプリの取り込み、Cloudflare、DB login、agenix の宣言
- live state: 現在動いている unit と timer、コンテナの image、秘密の実体、実行結果、alert

インシデント判断では3つを照合する。manifest に名前があるだけで、live 環境の deny、rotation、
監視設定が成立しているとはみなさない。

## データ分類

| データ | 扱い |
| --- | --- |
| 元動画、decoded frame、証拠画像 | browser 内限定。server request、DB、log へ送らない |
| 詳細な解析 report、timeline、入力履歴 | browser 内限定。公開 payload へ追加しない |
| 公開用集計 | public URL で誰でも閲覧可能。SA/CAはlevel・結果・自分側文脈のcountだけとし、不完全な場合は下限と明示する。個人情報、自由文、正確なdamage、最終gaugeを入れない |
| 共有 ID | 公開 URL の識別子。認証 credential とみなさない |
| 削除コード | secret。結果画面、localStorage、create / delete request だけで扱い、DB には Argon2id hash だけを置く |
| DB / GitHub credential | DB password は yunirun がホスト上で暗号化して保持し、その他の秘密は agenix と GitHub secret で管理する |

共有の closed schema と保持期間は [sharing.md](./sharing.md) を参照する。

## Trust boundary

| Identity | 必要な責務 | 持たせない権限 |
| --- | --- | --- |
| GitHub build job | source checkout、GHCR への image push | VPS への SSH、DB access |
| GitHub deploy job | OIDC からの短命 SSH 証明書、`yunirun deploy` の実行 | 長期の SSH 鍵、owner password の読取、他アプリへの deploy |
| `yunirun-fighter` (アプリのユーザ) | runtime env ファイル (app role password) の読取、blue/green と cleanup の起動 | DB owner、DDL、owner password の読取 |
| migration (root 側の unit) | owner password の読取、schema apply | application traffic、常時稼働 |
| cleanup (timer) | app role での DML、期限切れ row の削除 | DB owner、DDL、公開 traffic の受付 |

per-workload の権限分離の考え方は Cloud Run のときと同じである。runtime と cleanup は DML だけ、
migration だけが DDL できる。実現手段が service account と Secret Manager から、yunirun の
ユーザ分離と env ファイルの権限へ変わった。owner password の env ファイルは root 所有、runtime の
env ファイルはアプリのユーザ所有で、どちらも mode `0400` で tmpfs 上に置く。migration の unit を
アプリのユーザ側に作らないので、deploy 経路から owner の値を読み出す手段が無い。

DB へは同じホストの Unix socket で直結する。workload ごとに分けていた Cloudflare Access の
service token 3本と、その sidecar は使わなくなったので削除した。

application role `fighter_app` の権限は `schema/` の明示的な `GRANT` に限定する。
role の login policy は外部 DB infrastructure が所有し、password は yunirun がホスト鍵と
管理者鍵で暗号化して保持する。

deploy に GCP の資格情報を使わないので、builder / deployer secret の分離は無くなった。代わりに、
deploy job へ `environment:` を付けないこと (付けると OIDC の `sub` が変わり VPS 側の認可と
一致しなくなる) と、GHCR token を argv ではなく stdin で渡すことを contract test で固定する。
誰が deploy できるかは VPS 側の取り込み宣言と opkssh の認可先が決める。

third-party Actionとcontainerはreview済みcommit / manifest digestへ固定する。更新時はversion commentと
digestを同じPRで変更し、Renovateの差分をupstream releaseと照合する。緊急時も`latest`やmajor tagへ
一時的に戻さず、対象digestを明示する。

## Log 方針

server log に次を追加しない。

- 削除コード、hash、DB password、GHCR token
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
| Service availability | `/health` の失敗、5xx、blue/green の起動失敗、latency |
| DB path | `/ready`、共有 read / create / delete、connection / statement / lock timeout |
| Abuse control | bucket別429/503、Argon2 capacity、daily / active / storage quota 到達理由 |
| Cleanup | timer の最終実行、service の exit、削除件数、batch 安全上限、backlog |
| Capacity | logical bytes、active row数、物理relation size、DB connection、VPS の memory / disk |
| Identity | 取り込み宣言と opkssh 認可先の変更、SSH 証明書発行の失敗、deploy の失敗 |
| 公開経路 | cloudflared tunnel の切断、HAProxy backend の down、片系だけの稼働 |
| Delivery | deploy した SHA、GHCR への push、migration / blue-green 入替 / smoke の結果 |

`/health` は DB を query しない。`/ready`はruntime app roleのread-only queryで利用列、default、
constraint、PK/FK、cleanup index、必要grantのcatalog contractを確認する。どちらも`no-store`で、失敗時も
DB host、credential、schema差分をresponseへ出さない。`/ready`成功後もclosed schemaのtest dataで
read / create / deleteを確認し、実利用者の共有IDや削除コードをprobeに使わない。

## 共有の緊急停止

濫用、意図しない公開項目、共有 read の脆弱性が疑われる場合は、`yunirun.jsonc` の
`SHARE_RESULTS_ENABLED` を `false` にして `main` へ push する。環境変数は宣言から渡るので、
反映にはデプロイ1回分の時間がかかる。

それより早く止める必要がある場合は、VPS 上でアプリのユーザとしてコンテナを停止する。
この場合は共有だけでなく静的配信と `/health` も止まる。

```bash
systemctl --user stop fighter-blue.service fighter-green.service
```

この状態では新しい共有作成と `/s/:id` の取得が無効になり、既存 URL は `404` になる。
削除 API は継続する。browser 内の動画解析とローカル履歴も継続する。

停止後は次を行う。

1. 新しいコンテナの環境変数と、公開 URL の `404` を確認する。
2. cache purge が必要か Cloudflare の live 設定を確認する。
3. 原因、影響した ruleset / schema / image、公開期間を特定する。
4. 修正と再開条件を review する。`yunirun.jsonc` が正本なので、そこを戻すまで再開しない。
   コンテナ停止で止めた場合は次の deploy で起動し直ることに注意する。
5. 再開時に create、GET、delete、期限、cache header を一式確認する。

公開先や crawler が保持した preview、screenshot、cache は、DB 削除や緊急停止だけでは回収できない。

## Credential 漏えい

1. 漏えい候補の credential と consumer を1つに絞る。
2. audit log で利用時刻、source、対象 resource を確認する。
3. 新 credential / secret version を作り、その consumer だけを更新する。
4. コンテナと workload を起動し直し、正常系と deny 系を確認する。
5. 旧 credential を disable する。
6. rollback window 後に旧 version を破棄し、原因を修正する。

env ファイルは unit の起動時に読まれるので、値を書き換えただけでは動作中のコンテナは切り替わらない。
blue/green の両方を再起動し、新しい値で healthy になることを確認する。

DB app password と DB owner password を一度に rotation しない。password は yunirun がホスト上で
保持するので、rotation は VPS 側の操作になる。owner credential を application の復旧用に配布せず、
アプリのユーザから読める場所へ一時的に置かない。

## DB incident

不正 query、権限逸脱、data corruption が疑われる場合は次の順で扱う。

1. コンテナの起動、migration / cleanup の実行、DB session、HAProxy と cloudflared の log の時刻を揃える。
2. 影響する workload の共有停止、timer 停止、コンテナ停止の最小 containment を行う。
3. `fighter_app` の grant と実際の role membership を比較する。
4. table row、relation size、create event、schema version を read-only に確認する。
5. backup / point-in-time recovery の復元先を別 DB に作り、整合性を検証する。
6. 復旧後に app role の create / read / delete、owner role の migration、deny test を分けて行う。

production DB 上で原因調査と同時に手動 DDL や大量削除を行わない。rollback が schema を伴う場合は
[DEPLOY.md](./DEPLOY.md) の互換性条件を先に確認する。

## Cleanup failure

最初に timer の最終実行、service の exit、動いている image を確認する。次に DB timeout、
batch 安全上限、期限切れ row の backlog を確認する。

```bash
systemctl --user list-timers fighter-cleanup.timer
systemctl --user start fighter-cleanup.service
journalctl --user -u fighter-cleanup.service -n 200
```

手動実行を繰り返す前に、1回の実行が一部削除後に失敗したのか、接続前に失敗したのかを log で分ける。
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

閾値到達時はtimerの最終実行とlogを確認し、cleanupを1回実行する。`logical_bytes`はDELETEの
commit直後に減るため、その値と新規createを再確認する。物理relation fileが縮まらなくてもquota回復に
`VACUUM FULL`は不要である。DB diskの警告が別途残る場合はautovacuum状況を確認し、必要ならonlineの
`VACUUM (ANALYZE)`で再利用と統計更新を促す。blocking maintenanceはDB運用手順として別に計画する。

## 定期棚卸し

少なくとも四半期ごと、または IAM / schema / network 変更後に次を確認する。

- build job が VPS へ入れず、deploy 経路から owner password を読めないこと
- deploy job に `environment:` が付いておらず、opkssh の認可先が deploy 用ユーザに限られていること
- 開始済みのデプロイが後続pushでcancelされないこと
- runtime / cleanup が DDL できず、migration だけが schema apply できること
- runtime と migration の env ファイルの所有者と mode が分かれていること
- timer が cleanup 以外を起動しないこと
- sharing disable、期限切れ、誤った削除コード、quota 超過が fail closed になること
- DB backup の復元 test と、旧 image / 新 schema の rollback 互換性
- secret version、不要な role binding、古い image、失敗した Job execution の棚卸し
- RenovateがActions、pgschema、PostgreSQL service imageの更新PRを作成できること

supply chain と CI の未実装項目は [DEPLOY.md](./DEPLOY.md) の残余リスクを参照する。

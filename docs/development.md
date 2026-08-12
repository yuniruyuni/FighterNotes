# 開発ガイド

最終確認: 2026-08-03

## 前提

CI と production image は次の toolchain を使う。

| Tool | 基準 version | 用途 |
| --- | --- | --- |
| Rust | 1.95.0 | native test、WASM build |
| `wasm32-unknown-unknown` | Rust target | `wasm-bridge` |
| wasm-pack | 0.15.0 | JavaScript binding と WASM の生成 |
| Bun | 1.3.x | workspace、bundle、server、TypeScript test |
| PostgreSQL | 18 | 共有 API と integration test |

`crates/wasm-bridge/pkg`、`client/static`、`target`、`node_modules` は生成物で、
Git には含めない。checkout 直後は WASM binding がないため、client の型検査より先に
`bun run build` または `bun run check` を実行する。

## 最短の起動方法

Docker Compose は PostgreSQL、schema migration、application をまとめて起動する。

```bash
docker compose up --build
```

起動後は `http://localhost:3000` を開く。DB data は `pgdata` volume に残る。
schema を変更した場合も migration service が application より先に完了する。

## host 上で起動する

### 1. DB と schema

```bash
docker compose up -d postgres
docker compose run --rm migration
```

### 2. dependency と build

```bash
bun install --frozen-lockfile
bun run build
```

root の build は Rust/WASM、client、server executable を workspace script の定義どおりに
生成する。個別手順を複製せず、通常はこの command を使う。

### 3. server

```bash
DB_APP_NAME=template \
DB_PASSWORD=template \
PGUSER=fighter_app \
PGHOST=localhost \
STATIC_DIR=./client/static \
PUBLIC_BASE_URL=http://localhost:3000 \
bun run server/src/index.ts
```

server は起動時に遅延接続の DB pool を作る。静的配信と `/health` だけなら DB なしでも起動できるが、
共有 read / create / delete、`/ready`とcleanup の確認には schema 適用済み DB が必要である。

## watch mode

最初に一度 `bun run build` と migration を完了させ、上記と同じ DB 環境変数を設定して
次を実行する。

```bash
bun run watch:run
```

client の watch task は HTML、CSS、image、JavaScript、Worker を更新するが、WASM は
再生成しない。Rust を変更した後は client directory で次を実行し、必要なら browser を
reload する。

```bash
bun run build:wasm
bun run build:wasmcopy
```

## 検査

release 前の基本順序は次のとおり。

```bash
bun run check
bun scripts/validate-frame-data.ts
bun run build
bun audit
```

`bun run check` はWASM bindingを生成した後、Rustのformat、全警告をエラー扱いするclippy、
全crate test、WASM公開API契約、clientとserverのtype check、Biome、workspaceとroot `scripts/` の
unit testを実行する。`check:scripts`は`*.test.ts`だけを選ぶため、実動画を必要とするrunner本体は起動しない。
frame data validator はnetworkへ接続せず、公開dataのschema、意味制約、件数、checksumを検証する。

### 使用コンポーネントのライセンスinventory

`scripts/generate-third-party-notices.ts`は`bun.lock`のclient/server production dependency
closureを列挙し、各npm packageのlicense metadata・本文を
`license-checker-rseidelsohn`で取得する。Bunのisolated installで依存を取りこぼさないよう、
closureの解決にはlockfileと実際のpackage配置を併用する。Rust側は`cargo-about`が
`Cargo.lock`から`wasm-bridge`の`wasm32-unknown-unknown`向けproduction crateを列挙し、
適用するlicense全文を取得する。取得結果はcopyright表示、SPDX expression、source URL、
利用対象とともに次へ決定的に出力する。

- `THIRD_PARTY_NOTICES.md`: 完全なlicense・NOTICE本文を含む配布用通知
- `client/src/generated/third-party-licenses.ts`: `/licenses`表示用の一覧、license全文とlockfile hash

`cargo-about`はCIと同じversionを使用する。

```bash
cargo install cargo-about --version 0.9.1 --locked --features cli
```

dependency、lockfile、配布画像、icon、font方針または生成物の構成を変更した場合は、依存を
installした状態で次を実行し、2ファイルを同じcommitへ含める。

```bash
bun run generate:licenses
bun run check:licenses
```

許可するSPDX identifierは`scripts/license-policy.json`で管理し、Cargo向けの同じ一覧を
`about.toml`へ設定する。testは両者のずれを拒否する。`check:licenses`は標準SPDX parser、
`license-checker-rseidelsohn`と`cargo-about`を使い、未知・禁止license、license本文欠落、
manual override、asset inventory、lockfile hashおよび生成差分を検査する。許可リスト外の
dependencyを追加するとCIは失敗し、方針のreviewと明示的な更新が必要になる。

packageに専用のlicense・NOTICE文が欠ける場合だけ`scripts/license-overrides/`へversion固定の
review済み本文を追加する。`license-checker-rseidelsohn`がREADMEをlicense候補として返した
場合、README全体をlicense本文とはみなさず、このoverrideを使用する。開発・test専用packageと
compilerはapplication package inventoryの対象外とする。compiled serverへ埋め込まれる
Bun runtimeとbase containerのOS packageもlockfile inventoryの対象外だが、generatorが
Dockerfileのversion・image digestとupstream noticeへの導線を内部inventoryへ記録する。

Rust の変更範囲を絞って確認する場合は crate 単位でも実行できる。

```bash
cargo test -p frame-meter
cargo test -p meter-tracker
cargo test -p video-analyzer
cargo test -p wasm-bridge
```

### Mutation testing

通常 test が実装の変更を実際に検出できることは、StrykerJS 9.6.1 と
mutest-rs で確認する。全 crate を毎回まとめて回す。

Rust 側は mutest-rs を最初に導入する。crates.io に無いので git から入れる。
コンパイラ内部 API を使うため nightly の版が固定されており、`rust-toolchain.toml`
の指定に従って rustup が自動で用意する。追跡する版は
`.github/workflows/mutation.yml` の `MUTEST_REF` が正本。

```bash
git clone https://github.com/zalanlevai/mutest-rs
cd mutest-rs
cargo build --release -p mutest-runtime
cargo install --force --path mutest-driver
cargo install --force --path cargo-mutest
```

```bash
bun run mutation:ts:client
bun run mutation:ts:server-models
bun run mutation:rust:mutest
```

PostgreSQL repository は使い捨て DB を指定して別に実行する。

```bash
TEST_DATABASE_URL=postgres://fighter_test:fighter_test@localhost:5432/fighter_test \
  bun run mutation:ts:server-repositories
```

Stryker の `Survived`、`NoCoverage`、`Timeout`、runtime error は失敗とする。
生成不能な `CompileError` は結果に残すが失敗にはせず、`Ignored` は source 内に理由が
ある場合だけ許可する。`reports/mutation` は生成物であり Git へ含めない。

Stryker は command の終了コードだけで生死を決めるため、test は `--bail` で最初の失敗で
止める。repository の検査はさらに、DB を使わない test を先に走らせ、生き残った変異だけを
integration test へ回す。生き残る変異が走らせる test の集合は変わらないので、判定は同じに
なる。全 suite を毎回通していた頃は CI で 12 分かかっていた。

#### 全域を毎回回す理由

以前は cargo-mutants を使い、PR では変更行だけ、全域は週次にしていた。cargo-mutants は
変異ごとに増分ビルドするため費用の 95% がビルドで、workspace 全体は 13 並列で 13 分
かかっていた。mutest-rs は実行時に切り替わる形へ一度だけ変換するので、同じ 9,820 変異が
逐次 3 分で終わる。

標本抽出や種類の間引きは採らない。走るたびに対象が変わると結果が揺れ、この検査を
門番として信用できなくなる。全域を常に全部回すことがその条件になる。

#### mutest-rs を使う上での約束事

`scripts/run-mutest.ts` が全 crate を回し、結果を判定する。判定には三つの理由がある。

- **変異が 0 個なら失敗**。mutest はテストからの呼び出しグラフを起点に変異を作るので、
  テストが無い crate では変異が生成されず「成功」に見える。守られていない状態が緑に
  なるのは門番として最悪なので、ここで弾く。
- `--lib` を渡す。統合テストは本体を link する別バイナリなので、mutest が本体側の
  変異ハーネスを見つけられない。
- `--isolate=all` を渡す。変異によっては巻き戻せないパニックで abort し、隔離しないと
  走査全体が止まる。
- `--no-emit-metadata` を渡す。マクロ展開由来で実ファイルに対応しない span があると、
  メタデータ書き出しで mutest-driver 自身が落ちる。この出力は判定に使わない。

また、型推論と一時値の寿命に頼った書き方は mutest の変換の下で解けなくなる。
`let x = None;` は型を明示し、式の途中で借りる一時値には名前を付ける。どちらも
元の意図が読み取りやすくなるので、道具の都合を抜きにしても良い形になる。

対象 crate は `package.json` の `mutation:rust:mutest` が正本で、
`bun run check:mutation-coverage` が漏れと誤りを検査する。

`server/src/repositories/published-analysis.integration.test.ts` は `TEST_DATABASE_URL` がない場合に
skip される。この test は共有 table を truncate するため、production や共有開発 DB ではなく、
schema を適用した使い捨て DB を指定する。CI は専用 PostgreSQL service で実行している。

## Test data

raw video、screenshot、解析途中の全frame dumpはrepositoryへ保存しない。画像認識は合成pixel、
event / adviceの結合は`crates/video-analyzer/tests/pipeline_contract.rs`の合成HP・入力・meter timelineで
検査する。回帰条件は再現に必要な最小入力としてtest code内に追加する。

### 結果画面のkeyboard・screen reader確認

結果画面のnavigationやscene遷移を変更した場合は、解析済みの結果を表示して次をbrowser上で確認する。

1. `Tab`で「解析結果」navigation内のサマリー、各指摘、動画、認識デバッグへ順に移動できる。
2. 各項目を`Enter`または`Space`で開くと、現在項目が通知され、サマリー見出し、動画の再生位置、
   または認識デバッグの先頭操作へfocusが移る。
3. サマリー内の証拠場面とラウンド開始buttonを`Enter`と`Space`で開ける。特に`Space`でpageが
   scrollせず、遷移後は動画の再生位置へfocusが移る。
4. 非表示のサマリー、動画、認識デバッグの操作要素が`Tab`順とbrowserのaccessibility treeに
   残らない。pointerによるsidebar、証拠場面、ラウンド行の操作も引き続き動作する。

screen readerでは、Chrome + NVDAまたはSafari + VoiceOverのいずれかで、「解析結果」という
navigation名、現在項目、表示中の「解析結果サマリー」「動画」「認識デバッグ」というregion名を
確認する。証拠場面を開いた直後に非表示のbuttonではなく動画sliderが読み上げられ、非表示regionへ
virtual cursorで移動できないことも確認する。

### ローカル動画によるE2E回帰確認

実動画でしか再現できない問題の原因調査には、Chrome DevTools Protocol経由のローカルE2E runnerを
使用できる。動画、manifest、解析artifactはいずれもGitの無視対象である`video/`と`output/`に置き、
commitしない。問題を切り分けた後は、再現に必要な最小条件を合成pixelまたは合成timelineのtestへ移す。

最初にexampleをコピーして、手元の絶対パス、サイド、キャラクター、期待値を設定する。
Chrome/ChromiumはPATHまたはPlaywrightのローカルcacheから自動検出される。

```bash
cp scripts/local-video-e2e.example.json video/local-video-e2e.json
bun run build
bun run local:e2e
```

`expect.semanticEvents`では、次の検出結果をstableなannotation IDと許容frame範囲で照合できる。

- `fight`: 巨大なFIGHT表示のpeak frame
- `round`: ラウンド開始・終了・勝者
- `damage`: 被弾側、round、HP減少、中央表示damage・始動属性
- `super`: 使用側、SA level、CA、round、damage
- `attackInfo`: 中央表示から直接復元した攻撃列（未帰属も含む）
- `attackInfoAttribution`: 中央表示をHP被弾へ帰属できた攻撃列
- `adviceEvidence`: card IDと利用者へ提示するevidence frame

`detectorGates`でdetectorごとのfalse positive / false negative、precision / recall、平均frame誤差を
制限する。一部のeventだけをannotationする場合は、未annotationの実検出数を考慮して
`maxFalsePositives`を設定する。全件annotationしたfixtureでは0にする。実動画から原因を切り出して
合成testへ移植したannotationには`syntheticTest`へtest pathまたはtest名を記録する。未設定のIDは
summaryの`syntheticCoverage.pendingIds`へ残る。

各caseのreport、timeline、HP、入力、semantic event、空間解析結果、detector metrics、所要時間は
`output/local-video-e2e/current/`へ出力される。期待値違反があればcommandは失敗する。
runner はファイル選択後に画面の `data-video-preflight-status` が `valid` になるまで待ち、
`invalid` になった場合は表示された理由を含めて解析開始前に失敗する。これにより実 browser の
MP4 metadata、codec、`VideoFrame` bitmap probe を通ったこともローカルE2Eで回帰確認できる。
空間解析については処理frame数、decoder queue / decoded frame / Worker pendingのwatermarkとpeakも
case artifactへ保存する。runnerはpeakがhigh watermarkを超えたartifactを拒否する。
runner version 5は、全frame timestamp、frame-to-sample index、sample offset・size・timestampの
canonical SHA-256と件数も保存する。baseline比較ではこのmapping identityを完全一致させ、streaming化で
decode対象が変わっていないことを確認する。さらにmetadata/mediaの読込量、MP4BoxとBlob cacheのpeak、
batch・encoded queueのpeakと固定上限をartifactへ保存し、固定上限超過または設定値の改変を即時に拒否する。
通常の精度確認は1回で実行できる。`performance.measuredRuns`と`warmupRuns`、または
`--runs`と`--warmup-runs`で統計計測回数を指定できる。
出力は同じ親directoryの一時directoryで全caseとsummaryを完成させてから置換するため、解析中断や
artifact生成失敗で既存のcurrent directoryが部分的なrunに置き換わることはない。

実行環境を固定した速度・精度比較では、変更前の出力directoryを残して`--baseline`で指定する。
baseline比較はwarm-up後に最低3回を測り、総時間の中央値/p90とfirst pass、spatial pass、
frame切り出し、Worker copy、meter WASM、HUD WASMの中央値を閾値判定する。閾値超過は非0終了になる。
startupは解析buttonをclickした直前から、browserで最初の`EncodedVideoChunk`が構築されるまでを
外側から計測し、中央値とp90を別々の閾値で判定する。demux内部の開始から最初のsample batchまでの
時間も別の診断値として保存するが、変更前後で開始点が異なる実装の公平なstartup比較には使わない。
同一fixtureの空間処理frame数も一致を要求し、Worker peak pendingは比較ログへ出力する。
baseline側も1回以上のwarm-upと3回以上の計測で生成されていなければ比較を拒否する。
動画内容のSHA-256、side・character設定、annotation・期待値の正規化hash、runner version、
計測回数もbaseline contractへ保存する。動画・設定・期待値を削除または変更した比較は失敗する。
manifestとbaselineのcase集合も完全一致を要求し、caseの追加・削除・重複を比較前に拒否する。
summaryのcapture hash・semantic hashはcase artifactから再計算し、古いrunとの混在や差し替えを拒否する。
baseline比較を行うcaseには、1件以上の`semanticEvents`と`detectorGates`が必要になる。
`--baseline`と`--output`はreal path上でも別かつ包含関係にないdirectoryでなければならず、symlinkを
介して同じdirectoryを指定する比較も解析開始前に拒否する。

Linuxでrunner自身がChromeを起動した場合は、target作成前から解析完了とdecode mapping公開まで、
Chromeのprocess tree RSSを100ms間隔で測る。mappingのJSON化とCDP転送は測定外にする。この値は
Chrome本体や共有processを含み、複数processのVmRSSに現れる共有memory pageを重複計上し得て、
100ms未満の一時peakも捉えないため、同じhost・browser build・実行順でのA/B用proxyとして扱う。
`--cdp`利用時とLinux以外ではRSSは`null`になる。変更前後がともに`null`なら比較を明示的にskipし、
片側だけ`null`なら測定条件不一致として失敗する。

full-buffer版とstreaming版を直接比較するときは、変更前revisionの一時worktreeへ、変更後と同じ
click-to-first-`EncodedVideoChunk` probe、process-tree RSS sampler、decode mapping identity captureだけを
適用する。解析実装と既存のartifact契約は変えず、同じ実動画・manifest・browserでcontrolを先、変更版を
後に連続実行する。runner version 4のartifactはsemantic出力と総時間の比較には利用できるが、
version 5で追加したdecode mapping、streaming、startup、RSS比較は明示的にskipされる。

```bash
# 変更前の一時worktree（外部probeだけを適用済み）
bun run build
bun run local:e2e -- --output output/local-video-e2e/control --runs 3 --warmup-runs 1

# streaming変更後のworktree
bun run build
bun run local:e2e -- --output output/local-video-e2e/streaming --runs 3 --warmup-runs 1
```

controlの`<case>.full-buffer-probe.json`と変更後の`<case>.json`で、
`firstSampleRunsMs`、`firstSampleMedianMs`、`firstSampleP90Ms`、`processTreePeakRssBytes`の4fieldと
`decodeMapping.sha256`を照合する。変更後では4fieldは`streamingPerformance`内にある。mapping hashと
件数が一致し、startup中央値/p90とRSSが同じ測定契約で改善した場合だけ直接比較の根拠にする。

```bash
mv output/local-video-e2e/current output/local-video-e2e/baseline
bun run local:e2e -- --baseline output/local-video-e2e/baseline
```

baseline指定時はreport、timeline、HP、入力、FIGHT、中央攻撃情報、semantic event、空間解析結果も
構造比較する。差分はJSON path単位で表示され、速度が改善していても解析結果が変われば失敗する。
意図した精度変更はdiffと実動画を確認した後にだけ、current directoryを新しいbaselineとして昇格する。
これによりhash変更だけを黙って受け入れない。

中央攻撃情報の結果UIを確認するときは、case artifactの
`report.damage_breakdown.events[].attack_evidence`と同じ被弾場面を結果画面で開く。累積damage、hit数、
最終補正、始動・最終属性、標準HP 10,000換算との差が一致すること、`consistent` / `mismatch` /
`unverified` / 不完全表示が別の状態として読めることを確認する。`mismatch`があるfixtureでは警告内の
場面buttonから該当する動画区間へ移動できることも確認する。動画やartifactは引き続きcommitしない。

自動起動できないbrowserを使う場合は、専用profileとremote debugging portで起動し
`--cdp http://127.0.0.1:9222`を渡す。browserが別OSから動画を読む場合だけ、manifestの
`browserVideoPath`へそのOSから見えるpathを追加する。通常の`videoPath`はrunner自身が
fixtureの存在確認に使う。runner側で`browserVideoPath`の内容をSHA-256へ結び付けられないため、
この指定は単発の調査だけに使用でき、baseline比較では安全のため拒否される。

## 公式 frame data

`scripts/gen-frame-data.ts` は Street Fighter 6 公式 frame data pageを通常のHTTP
GETで取得し、WASM解析器が実行時に使う正規化済みcatalogを再生成する。取得・変換処理と
parserの合成入力testは公開repositoryで管理し、取得したHTMLやJavaScriptそのものは保存しない。
リモートJavaScriptは実行せず、埋め込まれたJSON文字列だけをparseする。

generatorが指定するUser-Agentは、通常のブラウザから閲覧した場合と同じ要求を再現し、
ブラウザ向けの正しいレスポンスを受け取るためのものである。認証やアクセス制御などの
技術的保護を回避する処理は行わない。

次の3ファイルを一つの変更として管理する。

- `frame_data.json`: 確反候補に必要なcommand、発生、damage、category
- `attack_data.json`: 入力照合に必要な発生、打撃属性、Classic / Modern入力pattern
- `manifest.json`: schema version、data version、各data fileのSHA-256と件数

JSONはcharacter keyとfield順を固定した1-space indent、LF終端のcanonical形式とする。
更新ごとに`YYYY-MM-DD.N`形式のdata versionを指定し、dataとchecksumを同じcommitで一致させる。

```bash
bun test scripts/
bun run scripts/gen-frame-data.ts 2026-08-03.1
bun scripts/validate-frame-data.ts
cargo test -p video-analyzer
```

generatorは各characterの取得間隔を1秒空け、HTTP error、技数の異常、ページ構造の変化で
処理を停止する。validatorはunknown field、未対応enum、空入力pattern、character欠落、
canonical形式の崩れ、manifestの件数・checksum不一致を失敗にする。更新PRではcharacter、
move、数値の差分も確認し、parserの変換不良を単なるdata更新としてmergeしない。

## 変更時の確認範囲

| 変更 | 最低限確認する対象 |
| --- | --- |
| HUD / input 認識 | `video-analyzer` の合成 frame / input test |
| meter 認識 | `frame-meter` と `meter-tracker` の合成 test |
| event / advice | `video-analyzer` unit test、`pipeline_contract`、[analysis.md](./analysis.md) |
| WASM API | `wasm-bridge` JSON契約test、`check:wasm-api`、client build、client test |
| 公開 payload / catalog | client、server、schema、[sharing.md](./sharing.md) |
| DB query / transaction | server unit test、PostgreSQL integration test、schema plan |
| workflow / manifest | CI、[DEPLOY.md](./DEPLOY.md)、[security-operations.md](./security-operations.md) |

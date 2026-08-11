# 動画解析パイプライン

最終確認: 2026-08-07

## 解析の位置づけ

解析器はゲーム内部ログや replay binary を読まず、録画映像に表示された情報だけを使う。
目的は試合を完全再現することではなく、複数の映像証拠が一致した場面を抽出し、
利用者が動画を見直す順序を作ることである。

判定は決定的なルールベース処理で、現在の `RULESET_VERSION` は 15。
機械学習モデルや外部推論 API は使わない。

## 入力条件

現在の較正と実動画検証は次の条件を前提にする。

- Street Fighter 6 のリプレイ再生画面
- 1920x1080、16:9、固定 60fps
- P1/P2 両方の入力履歴を表示
- リプレイ用フレームメーターを表示
- HP、Drive、SA、timer を含む HUD を表示
- crop、拡大、黒帯、字幕、配信 overlay などを加えない
- 主な検証環境は Steam 版 Windows を OBS で録画した動画

ファイル選択時に MP4 metadata を段階的に読み、表示・coded 寸法がともに 1920x1080、
59〜61fps、固定 frame rate、回転なしであることを解析開始前に検証する。固定 frame rate は
presentation timestamp を表示順へ並べ、60/1 または 60000/1001fps の一貫した整数量子化に
edit範囲内の全sampleが一致する場合だけ固定frame rateと判定する。B-frame復号用でedit終端外の
sampleは判定対象に含めず、提示範囲内の単発frame欠落はVFRとして拒否する。
fragmented MP4 は `moov` だけで全区間の固定 frame rate を証明できないため、現在は受け付けない。
`ftyp` は先頭のmajor brandを明示したMP4 allowlistと照合し、HEIC、AVIF、3GP、QuickTimeを
compatible brandだけでMP4と誤認しないようにする。
track matrix は標準的な 0/90/180/270 度だけを解釈し、回転、反転、scale、skew がある入力は拒否する。

metadata 条件の通過後に、動画固有 codec の `VideoDecoder.isConfigSupported` と、実際の
`VideoFrame` から `createImageBitmap` できることを確認する。Worker、OffscreenCanvas 2D、
WebCodecs などの実行環境条件も揃うまで解析 Worker と WASM は起動しない。失敗時は container、
寸法、frame rate、VFR、回転、codec、browser 機能を区別して録画または実行環境の直し方を表示する。

自分の side、自キャラクター、相手キャラクターは解析前に利用者が指定する。
現在の client は HUD OCR によるキャラクター自動判定を行わない。

## 解析コンテキスト

`AnalysisContext` は次を P1/P2 に正規化して Rust へ渡す。

- `ownSide`: `p1` または `p2`
- `p1.character` / `p2.character`
- 将来互換用の `controlType`
- 将来互換用の `battleVersion`

キャラクター情報は、確定反撃候補や一部のキャラクター固有行動を保守的に判定するために使う。
入力されていない metadata に依存する検出は有効化しない。

## 第一段: 全フレーム解析

### Demux と decode

事前検証は MP4Box の progressive parser が要求する file offset だけを最大 1MiB ずつ読み、
大きな `mdat` を保持せず `moov` と sample table を確認する。metadata の累積読込量は 32MiB を
上限とする。検証済み track metadata と `File` identity は解析開始時へ引き渡し、同じファイルで
あることを再確認してから再利用する。

`client/src/modules/analysis/infrastructure/video-decoding/mp4-video-source.ts` は元の `File` / `Blob` を
全量の `ArrayBuffer` にせず、MP4Box が要求する metadata 範囲と、次の抽出batchに必要なvideo sample
範囲だけを読む。末尾 `moov` の解析前に破棄された先頭側 `mdat` も、sample table確定後に必要範囲だけ
読み直す。圧縮sampleは1個16MiB、抽出batchは8sampleかつ16MiB、JavaScript側の未投入sample queueは
16sampleかつ32MiBを固定上限とし、queueが8sampleかつ16MiBまで減ってから供給を再開する。
`VideoDecoder`投入後はWorker完了前を12frameまでに制限する。1sample上限からの理論上は192MiBで、
decoder内部copy、codec、GPU memoryはJavaScriptのbuffer統計に含まれない。metadataまたは
圧縮frameが上限を超える入力は、通常のMP4への再多重化または低bitrateでの再エンコードを案内して
解析を停止する。

解析全体の調停は
`client/src/modules/analysis/infrastructure/pipeline/webcodecs-analysis-pipeline.ts` が担当する。
WebCodecs の `VideoDecoder` は encoded sample を順に decode し、各 `VideoFrame` から HUD、入力、
フレームメーターの strip を切り出す。

main thread は描画と buffer 転送を担当し、`client/src/entrypoints/analyzer-worker.ts` から
起動した Worker runtime が WASM を実行する。
2 slot の ping-pong buffer、decode queue 上限、Worker 未完了数の上限で memory 使用量を抑える。
native側を含む実使用量は、ローカルE2EのChrome process-tree RSSを補助指標として比較する。

### 知覚層

各 crate の責務は次のとおり。

| crate / module | 入力 | 出力 |
| --- | --- | --- |
| `frame-meter` | frame-meter strip | 左右 80 cell の色、明度、縞、数字相関、fresh edge |
| `meter-tracker` | フレームごとの cell 観測 | video frame と game frame を対応させた状態 timeline |
| `frame_features` | HUD strip | HP、Drive、SA/CA、burnout、試合画面判定、読取品質 |
| `attack_info` | 画面中央の攻撃情報 | P1/P2別の単発・累積ダメージ、補正率、上中下段・投げ属性 |
| `input_history` | input strip | 方向、button badge、AUTO、投げなどの row 0 観測 |
| `input_tracker` | row 0 の時系列 | 欠測と孤立誤読を補修した `TrackedInput` |

入力履歴とframe-meterの数字認識には、実ゲームを撮影した動画サンプルから生成した
認識用テンプレートを使用する。repositoryと配布物に元動画、frame、screenshot、cropは
含めず、サンプルを整列・集計または二値化した画素平均、標準偏差、正規化値、bit mask
だけを保持する。これらは画面表示用assetとしては使用しない。

フレームメーターは主に次の状態を区別する。

- startup / movement に対応する `counter`
- `punish_counter`
- `motion_recovery`
- `active` / `projectile_active`
- `parry`
- `stun`
- 全身、打撃、飛び道具の無敵縞

`meter-tracker` は fresh edge の進行と停止から game frame を復元する。
cursor が止まる hitstop や演出中は video frame が進んでも game frame を進めない。
リセット前後の状態を誤って結ばないよう、timeline segment には epoch 境界を持たせる。

### 確定層

`video_analyzer::pipeline::finalize_features` が、知覚層の値を時間方向に確定する。

1. HP の uncertain 区間を前後へ拡張し、信頼できる値で補間する。
2. 一瞬だけ急落して戻る HP 誤読を捨てる。
3. 両者 full HP の持続を round reset とし、round 内で HP を単調非増加にする。
4. Drive の短い孤立 segment や遮蔽由来の偽値を uncertain にし、直前の信頼値で埋める。
5. SA の整数ラベルと部分バーを統合し、短い偽ストック低下と同一ストック内の逆行を棄却する。

viewer と event layer は、この確定済み系列を共通の入力にする。画面表示だけ別補正する経路は持たない。

## イベント層

`build_match_events_with_context` は、確定特徴量、入力、meter timeline を同じ frame 座標へ揃え、
次の順で意味イベントを構築する。

1. 両者 full HP の持続から round 候補を作る。
2. SA 暗転、投げ、KO など、両者の game frame が止まる freeze span を確定する。
3. round 内 HP を単調化し、近接する HP 減少を damage sequence へまとめる。
4. damage のない誤 round を捨て、round number を振り直す。
5. meter の停止と HP 変化から hit / block contact を作る。
6. HP 減少の遅延を contact frame へ寄せ、freeze 前の clip anchor も保持する。
7. 中央攻撃情報をP1/P2別のコンボ列にし、攻撃側・round・frame・HP量が整合する被弾へ帰属する。
8. 中央表示のコンボリセットと実HP下降がともに確認できた場合だけ、結合されたdamageを分割する。
9. repaired input を方向・button の segment へまとめる。
10. meter、input、contact、damage を同じ行動へ帰属する。

`MatchEvents` が持つ主なイベントは次のとおり。

| 分類 | 内容 |
| --- | --- |
| Round / damage | round 境界、勝敗、HP 減少 sequence、ゲーム内damage・補正率・攻撃属性、freeze 前 anchor |
| Contact | hit / block、projectile contact、attacker / victim |
| Input action | jump、throw、Drive Impact、raw Drive Rush |
| Whiff | 接触しなかった攻撃判定と、その硬直を狩られたか |
| Knockdown | ダウンと起き上がり、起き上がりへの持続当て |
| Resource | burnout 期間、SA1/2/3/CA 使用、ゲージ前後、使用文脈と結果 |
| Frame interaction | punish chance、reversal、guard break、minus 後の最速打撃・投げ、plus 後の攻め継続 |
| Threat | 残存 projectile、teleport、複合 threat |

主な帰属上の不変条件は次のとおり。

- input が見えただけでは throw whiff や jump 成立と断定しない。
- DI は専用イベントへ帰属し、armor 表示を reversal と二重計上しない。
- meter epoch をまたいで contact、recovery、punish を結ばない。
- freeze をまたぐ combo damage は game time の近さでまとめる。
- 中央表示は単独でHPを上書きせず、被弾とのframe・攻撃側・round整合を確認して補助証拠にする。
- 中央表示とHPのdamageが一致しない場合は不一致として記録し、正確なdamage値として断定しない。
- SA は両者のゲージ低下を主証拠とし、meter の発生・暗転、contact、damage を使用時点と結果の確認に使う。
- SA/CA使用数のavailabilityは全検出ラウンドのゲージ観測被覆から判定する。単発イベントはその1回の証拠にはなるが、ほかの使用が無かった証拠にはしない。
- 接触しない SA2 は `NoImmediateContact` とし、技ごとの役割が不明なまま空振り失敗とは呼ばない。
- projectile の block や弾撃ち合いを、近距離の mashing として扱わない。
- 空振りは投げ・DI・無敵技・弾を除き、専用イベントを持つ行動と二重計上しない。
- ダウンは stun の長さだけで決めず、攻撃側が自由に動けるのに相手が stun の
  ままである空白を必須にする。連続ガードや連続ヒットと区別するため。
- Drive ゲージの消費量はSF6の本数を仮定せず、行動前後の実測差だけを積む。
  読めない区間と、1行動では説明できない大きな減少は消費へ帰属しない。
- 原因別カードへ帰属した大被弾を汎用 `big_hits` へ重複掲載しない。

meter や入力が読めない場合、一部のイベントは HP ベースへ fallback するか、未確認のまま出力しない。
欠測を成功として数えることはしない。

## 第二段: 候補区間の空間解析

第一段だけでは、本体間の距離、jump の実成立、Drive Rush の前進、長い通常技の届く範囲を
十分に区別できない。このため、次の候補だけを短区間で再 decode する。

- teleport と projectile が関係する区間
- reachability が不明な punish missed / whiff 候補
- hit / landed hit 候補を持つ jump
- 高信頼度の throw action
- Drive Rush 候補

各候補は round 境界内で merge し、直前 keyframe から連続 decode する。
480x270 の frame から actor anchor、bounding box、相対距離、左右順序、水平移動、
小型移動体の軌道を抽出する。

再 decode でも元の `File` / `Blob` からsample範囲だけを読み、1個のbounded cacheを再利用する。
cache miss中も旧cacheと新しい読込の合計を固定上限内に保つ。encoded queue と未処理 decoded frameを
high/low watermark で制限する。
RGBA buffer の Worker 転送も予約時点で pending 上限を確保し、ack が low watermark まで
戻ったときだけ再開する。このため長い候補窓でも VideoFrame と transferable buffer の滞留量は
窓長に比例しない。中断、decoder error、Worker error では待機中の admission を棄却し、受領済みの
VideoFrame を閉じてから解析を終了する。

空間観測は第一段の input / meter / contact 証拠を置き換えない。候補区間が実際に sampling され、
必要数の観測と confidence を満たした場合だけ、jump、punish、dash throw、Drive Rush、
teleport defense などを確認または棄却する。

## Advice report

`advice::build_report` は `MatchEvents` を自分 side へ写像し、次を生成する。

- round summary と推定勝敗
- damage taken の互換一覧
- 入力習慣統計
- 戦術統計
- 指摘カード
- 練習項目
- 解析 coverage と warning

coverage は「試合画面を確定ラウンドへ割り当てられた割合」と、HP、Drive、SA、入力履歴、
フレームメーター、中央攻撃表示、候補区間の空間観測を分けて報告する。主要な直接観測が
検出器ごとの閾値（原則60%、時系列で消費を確定するSAとイベント近傍の安定距離で確定する
空間人物追跡は20%）未満なら、対応する統計を確認不能とし、依存するカードを抑制する。
中央攻撃表示はHP被弾列への帰属率も検証する。SAゲージを十分に読めない場合、検出イベントが
無いことを「使用0回」とは扱わない。

現在のカード ID は次の23種である。

| ID | 対象 |
| --- | --- |
| `layered_defense` | projectile と teleport などの複合攻撃への防御 |
| `teleport_defense` | 裸 teleport への迎撃 |
| `anti_air` | 相手 jump-in への対空 |
| `own_jumps` | 自分の jump が落とされた場面 |
| `burnout` | burnout 時間、damage 収支、突入原因 |
| `committed_button_vs_di` | 通常技実行中に受けたDrive Impact |
| `mashing` | 守勢の button 押下と被弾の帰属 |
| `press_while_minus` | 不利 frame 後の最速打撃 |
| `throw_while_minus` | 不利 frame 後の最速投げ |
| `advantage_abandoned` | ガードさせて有利を取った後に攻めを継続せず渡したターン |
| `guard_break` | guard 方向を外した直後の被弾 |
| `reversal_punished` | 無敵技を防がれた後の反撃 |
| `low_scaling_super` | 低い補正率でSA/CAを組み込み、KOしなかった場面 |
| `punish_fail` | 時間は間に合ったが届かなかった反撃 |
| `punish_missed` | 到達可能な確定反撃機会の見逃し |
| `low_conversion` | 確定反撃の低い return |
| `throw_interrupted_by_invincible` | 投げ実行直後に相手の無敵技で被弾した場面 |
| `throw_whiff_punished` | 投げ空振り後に反撃を受けた場面 |
| `whiff_punished` | 接触しなかった技の硬直を狩られた場面 |
| `throw_loop` | 短時間の連続 throw 成立 |
| `early_hits` | round 開始直後の被弾 |
| `lead_loss` | 大きな HP lead を失った round |
| `big_hits` | 他の原因へ帰属しない大被弾 |

### 確度と表現

意味イベントは `Low`、`Medium`、`High` の confidence を持つ。
利用者向けの件数とカードは原則として、高信頼度まで確認できたイベントを使う。

各カードは、その指摘が挙げた場面で実際に失った HP を `hp_lost` として持つ。
被ダメージが指摘の直接の結果である場合だけ設定し、確反の取りこぼしのように
損失が機会費用であるものは未設定にする。0 と書くと「損害なし」に読めるため、
未設定と区別する。表示順は従来どおり分類・確度・severity で、`hp_lost` は
利用者が優先順位を判断するための材料として提示する。

カードは次の3分類を持つ。

- `Diagnosis`: 反復や明確な因果を確認し、改善対象として提示できる
- `Observation`: 事実は確認できるが、単発の読み負けを癖とは断定しない
- `Statistic`: 評価を加えず集計として提示する

読み合いを含む minus 後の回答偏重は、入力付きの機会4回以上、同じ回答3回以上、
選択率70%以上、その回答で2敗以上を `Diagnosis` の基準にする。
不利フレーム後・有利フレーム後・起き攻めは、状況・選んだ回答・結末という同じ形へ
`advice::decisions` で射影し、偏りの判定条件を1か所で持つ。この層は既存イベントの
読み替えであって再導出ではない。各状況の検出はイベント層が担当し、判定の根拠だけを
集約することで、検出結果を動かさずに状況を追加できる。

全機会を正確に数えられないカテゴリも、単発は `Observation` とし、同種の負け方が
最低2回ある場合に反復として扱う。確定反撃見逃しのような非読み合いの失敗は、
必要な証拠が揃えば1回から `Diagnosis` にできる。

## ローカル履歴

解析完了時に、動画の size / lastModified、side、キャラクター、ruleset から
SHA-256 の不透明な ID を作り、round 数と `tactic_stats` を IndexedDB に保存する。
動画ファイル名は ID の生成にも永続化にも使用しない。同じ ID の再解析は上書きし、最大200件を保持する。

結果画面の「今後の解析履歴を保存する」は初期状態で ON とする。利用者はこの設定を
OFF にして以後の自動保存を止め、再び ON にできる。storage が利用不能または設定値が
壊れている場合は privacy 側へ fail closed し、新しい履歴を保存しない。

結果画面は同じ ruleset の記録だけを自キャラ・相手キャラの組み合わせ別に集計する。
ruleset が異なる判定結果を同じ率へ混ぜない。管理欄の保存件数には旧 ruleset も含め、
各 record または解析履歴全件を確認付きで削除できる。解析履歴の削除は別領域の共有 ID・
削除コードを変更しない。分母が0の項目は成功率を表示しない。

## 現在の限界

- 画像上の色、位置、表示 timing に基づく heuristic であり、ゲーム内部の正解ではない。
- character motion から一般的な技名を確定する処理はない。frame data は確反候補の提示に使う。
- 画面上の相対距離は camera movement、effect、遮蔽の影響を受ける。
- 未知の録画環境、解像度、codec、色変換では既存 threshold が合わない可能性がある。
- 入力履歴は画面に表示された row 0 を時系列補修した値で、内部 input log ではない。
- 0件は「失敗しなかった」ではなく、「確認できる機会がなかった」場合を含む。
- SA/CA は使用、hit / block、即時接触なし、反撃、KO、使用文脈を観測事実として集計する。
  ruleset v9以降では両者別に`complete`・`partial`・`unavailable`を出力する。`complete`だけが0回を確定でき、
  `partial`は検出済み件数を下限として扱い、`unavailable`は件数を持たない。
  未使用ゲージだけから「使うべきだった」、残りHPだけから「倒し切れた」とは判定しない。
- 攻撃面は punish、low conversion、与 damage、burnout 収支など一部だけで、neutral の技選択、
  combo 完走率、起き攻めの質を網羅的には評価していない。

精度を変更するときは、局所的な合成テストに加え、`crates/video-analyzer/tests/pipeline_contract.rs` の
複数ラウンドと character 固有シナリオを通して event / advice の結合を確認する。

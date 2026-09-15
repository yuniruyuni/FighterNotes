import type {
  MatchupGuideCatalog,
  MatchupGuideItem,
} from "~/modules/results/domain/matchup-guide.js";

/**
 * キャラ対の手引き(curated)。解析結果ではなく人手で書いた知識なので、
 * 断定はキャラ間で答えが変わらない性質(溜め・投げ不可ガード・体力等)に
 * 限り、フレーム数値の断定は避ける。技は frame_data と同じ記譜で書く。
 *
 * 執筆・確認時点: 2026-09。パッチで答えが変わりうる項目は、次の確認まで
 * 表現を保守的に保つ。
 */
export const MATCHUP_GUIDE_VERSION = "2026-09";

const dpPunish = (
  id: string,
  notations: readonly string[],
): MatchupGuideItem => ({
  id,
  triggers: [{ kind: "move", notations }],
  text: "昇竜系の無敵技をガードしたら、大きな確定反撃の機会です。最大リターンのコンボを 1 つ決めておき、ガード後に迷わず出せるようトレモで反復しましょう。",
});

export const MATCHUP_GUIDES: MatchupGuideCatalog = {
  KEN: [
    {
      id: "ken-jinrai",
      triggers: [
        { kind: "move", notations: ["236LK", "236MK", "236HK", "236KK"] },
      ],
      text: "236K(迅雷脚)は派生を含む多段の連係です。途中で手を出すと派生に負けやすいので、まず最終段までガードし、強度ごとのガード後の状況をトレモで一度確認しておきましょう。",
    },
    dpPunish("ken-dp", ["623LP", "623MP", "623HP", "623PP"]),
    {
      id: "ken-rush",
      triggers: [{ kind: "stat", stat: "raw_drive_rushes_faced" }],
      text: "ケンはドライブラッシュからの中下段と投げの圧が強い相手です。ラッシュの出がかりを止める牽制ボタンを 1 つ決め、止められない距離では後ろ歩きで間合いごと外しましょう。",
    },
  ],
  ZANGIEF: [
    {
      id: "zangief-spd",
      triggers: [
        { kind: "move", notations: ["360LP", "360MP", "360HP", "360PP"] },
        { kind: "stat", stat: "throws_taken" },
      ],
      text: "360P(スクリューパイルドライバー)は通常投げより間合いが広く、ガードでは防げません。密着でガードを固め続けるのが最も危険で、垂直ジャンプ・後ろ歩き・暴れを混ぜて投げ間合いから外れましょう。",
    },
    {
      id: "zangief-sa-grab",
      triggers: [{ kind: "move", notations: ["720P"] }],
      text: "720P(SA の投げ)は間合いがさらに広く発生も速いです。相手に SA ゲージがある間は、密着での様子見と歩き接近を減らし、投げ間合いに入る時間そのものを短くしましょう。",
    },
    {
      id: "zangief-jump-risk",
      triggers: [{ kind: "card", id: "own_jumps" }],
      text: "ザンギエフへの飛び込みは PP(ラリアット)や対空 SA で大きく咎められます。接近はジャンプではなく、弾や牽制で歩かせてからの差し返しを軸にする方が安全です。",
    },
  ],
  CHUN_LI: [
    {
      id: "chunli-kikoken",
      triggers: [
        { kind: "move", notations: ["[4]6LP", "[4]6MP", "[4]6HP", "[4]6PP"] },
      ],
      text: "[4]6P(気功拳)は溜めが必要な弾です。弾を撃った直後は次の弾まで間が空くので、ガードやパリィで受けたらその瞬間に前へ歩き、間合いを詰める時間に変えましょう。",
    },
    {
      id: "chunli-legs",
      triggers: [
        { kind: "move", notations: ["236LK", "236MK", "236HK", "236KK"] },
      ],
      text: "236K(百裂脚)は多段技で、強度により段数とガード後の距離が変わります。途中で手を出さず最後までガードし、ガード後の距離を見てから反撃を判断しましょう。",
    },
    {
      id: "chunli-tensho",
      triggers: [
        { kind: "move", notations: ["[2]8LK", "[2]8MK", "[2]8HK", "[2]8KK"] },
      ],
      text: "[2]8K(天昇脚)は強力な対空です。この技を見せられた後の飛び込みは読まれています。接近は地上主体へ切り替え、飛ぶなら対空の溜めが作れない状況(相手の前歩き中など)に限定しましょう。",
    },
  ],
  AKUMA: [
    {
      id: "akuma-fireball",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "アクマは弾と対空が強い代わりに体力が低い相手です。弾のすべてに飛びで回答しようとすると読み負けるので、歩きガードとパリィで着実に近づき、触ってからのリターンで取り返しましょう。",
    },
    dpPunish("akuma-dp", ["623LP", "623MP", "623HP", "623PP"]),
    {
      id: "akuma-tatsu",
      triggers: [
        { kind: "move", notations: ["214LK", "214MK", "214HK", "214KK"] },
      ],
      text: "214K(竜巻)はガード後の状況が強度で大きく変わります。ガードできたら反撃が確定するかを強度別にトレモで確認しておくと、確反の取り逃しが減ります。",
    },
  ],
  JAMIE: [
    {
      id: "jamie-drink",
      triggers: [{ kind: "move", notations: ["63214K", "63214KK"] }],
      text: "63214K(点辰)を通されると相手の酒が進み、以降の火力と技が強化されます。密着の読み合いで通さないことが第一で、通された直後は強化された圧を無理に受けず、一度距離を取り直しましょう。",
    },
    {
      id: "jamie-freeflow",
      triggers: [
        {
          kind: "move",
          notations: ["236LP", "236MP", "236HP", "236PP"],
        },
      ],
      text: "236P からの派生(~6P / ~6K)は多段の連係で、途中の隙は強度と派生で変わります。まず最後までガードし、派生が途切れた位置で反撃できるかをトレモで確認しましょう。",
    },
    dpPunish("jamie-dp", ["623LK", "623MK", "623HK", "623KK"]),
  ],
  RYU: [
    {
      id: "ryu-fireball",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "リュウは弾と差し返しの地上戦が本体です。弾のすべてに飛びで回答しようとすると読み負けるので、ガードとパリィで受けながら少しずつ前に出て、間合いを詰めた時間で勝負しましょう。",
    },
    dpPunish("ryu-dp", ["623LP", "623MP", "623HP", "623PP"]),
    {
      id: "ryu-tatsu",
      triggers: [
        { kind: "move", notations: ["214LK", "214MK", "214HK", "214KK"] },
      ],
      text: "214K(竜巻)はガード後の状況が強度で変わります。ガードできたら反撃が確定するかを強度別にトレモで確認しておくと、確反の取り逃しが減ります。",
    },
  ],
  LUKE: [
    dpPunish("luke-dp", ["623LP", "623MP", "623HP", "623PP"]),
    {
      id: "luke-knuckle",
      triggers: [
        {
          kind: "move",
          notations: [
            "214LP",
            "214MP",
            "214HP",
            "214PP",
            "214[H]LP",
            "214[H]MP",
            "214[H]HP",
          ],
        },
      ],
      text: "214P(ナックル)には溜め(ホールド)版があり、ガード後の状況が通常版と別物です。ガードしたら反射で手を出さず、溜めの有無を見てから反撃を判断しましょう。",
    },
    {
      id: "luke-fireball",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "236P(サンドブラスト)を軸にした中距離の圧が強い相手です。弾の間合いで待ち合いに付き合わず、パリィと歩きで一段近い間合いを維持しましょう。",
    },
  ],
  GUILE: [
    {
      id: "guile-boom",
      triggers: [
        { kind: "move", notations: ["[4]6LP", "[4]6MP", "[4]6HP", "[4]6PP"] },
      ],
      text: "[4]6P(ソニックブーム)は溜めが必要な弾で、本体がブームの後ろに付いてくる形が本命です。弾そのものより、付いてくる本体への回答(パリィで受けて近づく・置き技)を用意しましょう。",
    },
    {
      id: "guile-flashkick",
      triggers: [
        { kind: "move", notations: ["[2]8LK", "[2]8MK", "[2]8HK", "[2]8KK"] },
      ],
      text: "[2]8K(サマーソルト)は対空と切り返しの要ですが、ガードすれば大きな確定反撃です。撃たせてガード、まで見えたら最大コンボを決めましょう。",
    },
    {
      id: "guile-charge-gap",
      triggers: [{ kind: "stat", stat: "jump_ins_allowed" }],
      text: "ソニックもサマーも溜めが必要なので、ガイルが前に歩いている瞬間はどちらも出せません。前歩きを見た瞬間が差し込みと接近のチャンスです。",
    },
  ],
  JP: [
    {
      id: "jp-zoning",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "JP は設置と弾で「触らせない」ことが本体です。遠距離で付き合う時間が長いほど不利になるので、ラインを上げ続けることを最優先にしましょう。",
    },
    {
      id: "jp-amnesia",
      triggers: [{ kind: "stat", stat: "throws_taken" }],
      text: "22K(アムネジア)は打撃を取る当身です。密着で打撃を連打すると吸われるので、投げと様子見を混ぜて崩しましょう。当身を見せられた後こそ投げの通りが良くなります。",
    },
    {
      id: "jp-no-dp",
      triggers: [{ kind: "card", id: "advantage_abandoned" }],
      text: "JP は無敵の昇竜系を持たないため、一度触れば攻めの継続が通りやすい相手です。有利を取ったら手を止めず、攻め切りましょう。",
    },
  ],
  CAMMY: [
    {
      id: "cammy-arrow",
      triggers: [
        { kind: "move", notations: ["236LK", "236MK", "236HK", "236KK"] },
      ],
      text: "236K(スパイラルアロー)は先端ガードと めり込みでガード後の状況が変わります。めり込みは確定反撃なので、ガードしたら距離を見て反撃を判断しましょう。",
    },
    dpPunish("cammy-spike", ["623LK", "623MK", "623HK", "623KK"]),
    {
      id: "cammy-ground-speed",
      triggers: [{ kind: "stat", stat: "throws_taken" }],
      text: "キャミィは歩きとラッシュの速さで投げ間合いへ入るのが得意です。密着される前の間合い管理(後ろ歩き・置き技)で、投げの間合いに入る回数そのものを減らしましょう。",
    },
  ],
  JURI: [
    {
      id: "juri-fuha",
      triggers: [
        { kind: "move", notations: ["236LK", "236MK", "236HK", "236LKMK"] },
      ],
      text: "ジュリは 236K で風破を溜め、ストックの有無で圧が別物になります。溜める動作そのものは隙なので、見えたら前進と差し込みのチャンスです。",
    },
    dpPunish("juri-dp", ["623LP", "623MP", "623HP", "623PP"]),
  ],
  DEE_JAY: [
    {
      id: "deejay-charge",
      triggers: [
        {
          kind: "move",
          notations: [
            "[4]6MP",
            "[4]6HP",
            "[4]6PP",
            "[2]8MK",
            "[2]8HK",
            "[2]8KK",
          ],
        },
      ],
      text: "エアスラッシャーもジャックナイフも溜めが必要です。ディージェイが前に歩いている瞬間はどちらも出せないので、前歩きを見たら差し込みと接近のチャンスです。",
    },
    {
      id: "deejay-sobat",
      triggers: [{ kind: "move", notations: ["236MK", "236HK", "236KK"] }],
      text: "236K(ソバット)は先端ガードだと反撃が届きにくく、めり込みなら確反です。ガードしたら距離を見て反撃を判断しましょう。フェイントで揺さぶられても、ガード方向を崩さないことが第一です。",
    },
  ],
  E_HONDA: [
    {
      id: "honda-headbutt",
      triggers: [
        {
          kind: "move",
          notations: [
            "[4]6LP",
            "[4]6MP",
            "[4]6HP",
            "[4]6PP",
            "[2]8LK",
            "[2]8MK",
            "[2]8HK",
            "[2]8KK",
          ],
        },
      ],
      text: "[4]6P(頭突き)と [2]8K(百貫)はガード後の反撃可否が強度と当たり方で変わる代表格です。トレモで一度「どれをガードしたら何が確定するか」を確認しておくと、本田戦の景色が変わります。",
    },
    {
      id: "honda-oicho",
      triggers: [
        {
          kind: "move",
          notations: ["63214LK", "63214MK", "63214HK", "63214KK"],
        },
        { kind: "stat", stat: "throws_taken" },
      ],
      text: "63214K(大銀杏投げ)はコマンド投げでガードできません。密着でガードを固め続けず、垂直ジャンプや暴れを混ぜて投げ間合いから外れましょう。",
    },
  ],
  BLANKA: [
    {
      id: "blanka-rolling",
      triggers: [
        { kind: "move", notations: ["[4]6LP", "[4]6MP", "[4]6HP", "[4]6PP"] },
      ],
      text: "[4]6P(ローリング)は当たり方でガード後の状況が変わる代表格です。ガードしたら位置を確認してから反撃、を習慣にしましょう。読みでパリィが取れれば大きなリターンです。",
    },
    {
      id: "blanka-wildhunt",
      triggers: [
        {
          kind: "move",
          notations: ["63214LK", "63214MK", "63214HK", "63214KK"],
        },
      ],
      text: "63214K(ワイルドハント)は跳びかかる掴みです。予備動作が見えたら、ガードではなく打撃かジャンプで拒否できます。",
    },
    {
      id: "blanka-electric",
      triggers: [
        {
          kind: "move",
          notations: ["214P", "214PP"],
        },
      ],
      text: "密着の電撃(214P)に付き合って暴れると連続で被弾します。一度離れて、電撃が空振る距離から差し返しましょう。",
    },
  ],
  DHALSIM: [
    {
      id: "dhalsim-teleport",
      triggers: [{ kind: "card", id: "teleport_defense" }],
      text: "テレポートの着地には必ず隙があります。見てから最速の対空・投げで咎める、をトレモで形にしておくと、裏回りの恐怖が読み合いに変わります。",
    },
    {
      id: "dhalsim-zoning",
      triggers: [
        {
          kind: "move",
          notations: [
            "236LP",
            "236MP",
            "236HP",
            "236LPMP",
            "236LPHP",
            "236MPHP",
          ],
        },
      ],
      text: "ダルシムは手足と弾で遠距離を制圧しますが、体力と切り返しは並以下です。時間をかけても着実に近づき、一度触ったら離さないことが基本方針です。",
    },
  ],
  MARISA: [
    {
      id: "marisa-charge-unblockable",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "236P は溜めるとガードできない版に化けます。マリーザが技を溜め始めたら、ガードで待つのではなく、発生前に潰すか間合いから外れましょう。",
    },
    {
      id: "marisa-armor",
      triggers: [{ kind: "stat", stat: "di_faced" }],
      text: "マリーザはアーマーで突っ込んでくる技を持ちます。単発の置き技で止まらない場面があったら、投げ・下段・多段など、アーマーで受けられない性質の回答へ切り替えましょう。",
    },
  ],
  MANON: [
    {
      id: "manon-grab",
      triggers: [
        {
          kind: "move",
          notations: ["63214LP", "63214MP", "63214HP", "63214PP"],
        },
        { kind: "stat", stat: "throws_taken" },
      ],
      text: "63214P(コマンド投げ)はガードできず、通されるたびにメダルが育って後半の投げがどんどん痛くなります。密着で固まらず、前ジャンプ・バクステ・暴れを配分して序盤から通さないことが大切です。",
    },
    {
      id: "manon-weak-reversal",
      triggers: [{ kind: "card", id: "advantage_abandoned" }],
      text: "マノンは無敵の切り返しが乏しく、触られる展開が苦手です。有利を取ったら手を止めず、攻めを継続しましょう。",
    },
  ],
  LILY: [
    {
      id: "lily-grab",
      triggers: [
        { kind: "move", notations: ["360LP", "360MP", "360HP", "360PP"] },
        { kind: "stat", stat: "throws_taken" },
      ],
      text: "360P(コマンド投げ)持ちです。密着でガードを固め続けず、垂直ジャンプや暴れを混ぜて投げ間合いから外れましょう。",
    },
    {
      id: "lily-wind",
      triggers: [
        {
          kind: "move",
          notations: ["236LK", "236MK", "236HK", "236LKMK/LKHK", "236MKHK"],
        },
      ],
      text: "リリーは風のストックで突進技が強化されます。風を溜める動作そのものは隙なので、見えたら前進と差し込みのチャンスです。",
    },
    dpPunish("lily-dp", ["623LP", "623MP", "623HP", "623PP"]),
  ],
  KIMBERLY: [
    {
      id: "kimberly-run",
      triggers: [
        { kind: "move", notations: ["214LK", "214MK", "214HK", "214KK"] },
      ],
      text: "214K(疾駆け)からの派生は全部を見分けようとせず、まずガードで受け切って観察しましょう。同じ派生に偏っていると分かってから、その択だけ拒否するのが安定します。",
    },
    {
      id: "kimberly-spray",
      triggers: [{ kind: "stat", stat: "raw_drive_rushes_faced" }],
      text: "スプレー缶の設置(22P)は大きな隙です。離れた位置で設置を見たら、前進や弾で咎めてラインを上げましょう。",
    },
    {
      id: "kimberly-no-dp",
      triggers: [{ kind: "card", id: "advantage_abandoned" }],
      text: "キンバリーは無敵の昇竜系を持ちません。一度触れば攻めの継続が通りやすいので、有利を取ったら手を止めないことが大切です。",
    },
  ],
  RASHID: [
    {
      id: "rashid-whirlwind",
      triggers: [
        {
          kind: "move",
          notations: [
            "236LK",
            "236MK",
            "236HK",
            "236KK",
            "236[H]LK",
            "236[H]MK",
            "236[H]HK",
            "236[H]KK",
          ],
        },
      ],
      text: "旋風(竜巻の設置)を盾にした接近が本体です。竜巻は時間で消えるので、無理に突っ込まず、消えるまで下がって待つ位置取りも立派な回答です。",
    },
    {
      id: "rashid-mixer",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "236P(ミキサー)をガードしたら反撃の機会です。空中の機動(壁・ラン跳び)で裏表を作られる場面では、着地点に対空を置く意識を持ちましょう。",
    },
  ],
  M_BISON: [
    {
      id: "bison-scissors",
      triggers: [
        { kind: "move", notations: ["236LK", "236MK", "236HK", "236KK"] },
      ],
      text: "236K(シザースキック)は先端ガードだと反撃が届きにくく、めり込みなら確反です。ガードしたら距離を見て反撃を判断しましょう。",
    },
    {
      id: "bison-charge",
      triggers: [
        { kind: "move", notations: ["[4]6LP", "[4]6MP", "[4]6HP", "[4]6PP"] },
      ],
      text: "[4]6P(サイコクラッシャー)は溜めが必要です。ベガが前に歩いている瞬間は撃てないので、歩き出しが差し込みのチャンスです。",
    },
  ],
  SAGAT: [
    {
      id: "sagat-tigers",
      triggers: [
        {
          kind: "move",
          notations: [
            "236LP",
            "236MP",
            "236HP",
            "236PP",
            "236LK",
            "236MK",
            "236HK",
            "236KK",
          ],
        },
      ],
      text: "上段(236P)と下段(236K)の 2 種の弾でジャンプを誘うのがサガットの型です。飛びにはアッパーカットが待っているので、パリィと歩きガードで地上から近づきましょう。",
    },
    dpPunish("sagat-dp", ["623LP", "623MP", "623HP", "623PP"]),
  ],
  TERRY: [
    dpPunish("terry-dp", ["623LP", "623MP", "623HP", "623PP"]),
    {
      id: "terry-burnknuckle",
      triggers: [
        {
          kind: "move",
          notations: ["214LP", "214MP", "214HP", "214PP", "214LPMP"],
        },
      ],
      text: "214P(バーンナックル)はめり込みをガードしたら確定反撃です。先端は反撃が届きにくいので、ガードしたら距離を見て判断しましょう。",
    },
    {
      id: "terry-powerwave",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "236P(パワーウェイブ)は地を這う弾で、ジャンプやパリィで越えやすい部類です。弾越えの選択肢を 1 つ決めておくと近づきやすくなります。",
    },
  ],
  A_K_I: [
    {
      id: "aki-poison",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "A.K.I. は毒を絡めた中距離戦が本体です。毒を付けられている間は被弾のリターンを上げられているので、その間だけ一段守りを厚くし、毒が切れてから仕掛け直しましょう。",
    },
    {
      id: "aki-weak-reversal",
      triggers: [{ kind: "card", id: "advantage_abandoned" }],
      text: "A.K.I. は無敵の切り返しが乏しく、触られる展開が苦手です。有利を取ったら手を止めず、攻めを継続しましょう。",
    },
  ],
  MAI: [
    {
      id: "mai-fan",
      triggers: [
        { kind: "move", notations: ["236LP", "236MP", "236HP", "236PP"] },
      ],
      text: "236P(花蝶扇)は溜め版で強化される弾です。弾の撃ち合いに付き合いすぎず、ガードとパリィで受けながら前に出る時間を作りましょう。",
    },
    dpPunish("mai-dp", ["623LK", "623MK", "623HK", "623KK"]),
    {
      id: "mai-reversal-sa",
      triggers: [{ kind: "move", notations: ["236236K"] }],
      text: "リバーサルの SA(236236K)は発生が非常に速く、起き攻めの重ねが甘いと割り込まれます。相手に SA ゲージがある間の起き攻めは、様子見や安全飛びを混ぜてゲージを吐かせましょう。",
    },
  ],
};

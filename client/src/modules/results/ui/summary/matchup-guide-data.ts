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

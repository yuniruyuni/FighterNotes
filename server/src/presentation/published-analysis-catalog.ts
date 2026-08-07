import type { CharacterId, FindingKind } from "../models/published-analysis";

export interface FindingPresentation {
  title: string;
  description: string;
  practice: string;
  observation?: {
    title: string;
    description: string;
    practice: string;
  };
  tone: "critical" | "warning" | "defense" | "resource";
}

const OBSERVATION_REVIEW_CAVEAT =
  "断定できませんが、検討の対象にしてもよいかもしれません";

const CHARACTER_NAMES: Record<CharacterId, string> = {
  A_K_I: "A.K.I.",
  AKUMA: "AKUMA",
  ALEX: "ALEX",
  BLANKA: "BLANKA",
  C_VIPER: "C.VIPER",
  CAMMY: "CAMMY",
  CHUN_LI: "CHUN-LI",
  DEE_JAY: "DEE JAY",
  DHALSIM: "DHALSIM",
  E_HONDA: "E.HONDA",
  ED: "ED",
  ELENA: "ELENA",
  GUILE: "GUILE",
  INGRID: "INGRID",
  JAMIE: "JAMIE",
  JP: "JP",
  JURI: "JURI",
  KEN: "KEN",
  KIMBERLY: "KIMBERLY",
  LILY: "LILY",
  LUKE: "LUKE",
  M_BISON: "M.BISON",
  MAI: "MAI",
  MANON: "MANON",
  MARISA: "MARISA",
  RASHID: "RASHID",
  RYU: "RYU",
  SAGAT: "SAGAT",
  TERRY: "TERRY",
  YASMINE: "YASMINE",
  ZANGIEF: "ZANGIEF",
};

export const FINDING_PRESENTATIONS: Record<FindingKind, FindingPresentation> = {
  layered_defense: {
    title: "複合攻撃への短いパリィが繰り返されている",
    description:
      "飛び道具とテレポート後の攻撃が重なる連係で、後段まで守り切れなかった場面です。",
    practice:
      "飛び道具を受けた後もパリィを維持し、反対側からの打撃まで受け切る練習が有効です。",
    observation: {
      title: "複合攻撃でパリィ後に被弾した場面",
      description: `後段までパリィを維持できなかった事実を示します。単発では投げを警戒した読みか、入力傾向かを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "パリィを離した理由を確認し、他の同状況でも同じ離し方をしているか見比べます。",
    },
    tone: "defense",
  },
  teleport_defense: {
    title: "裸テレポートへの迎撃が遅い",
    description:
      "飛び道具が重なっていないテレポート攻撃を、届く対空で迎撃できなかった場面です。",
    practice:
      "裸テレポートだけを対空し、飛び道具が残る連係ではパリィやガードへ切り替えます。",
    tone: "defense",
  },
  anti_air: {
    title: "飛び込みを繰り返し通している",
    description: "相手の前ジャンプを迎撃できず、飛び込みを通された場面です。",
    practice:
      "地上への意識を一度下げ、複数の前ジャンプへ対空を安定して出す練習が有効です。",
    observation: {
      title: "飛び込みを通した場面",
      description: `相手の飛び込みから被弾した事実です。単発または低い失敗率では対空の癖とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "別の行動中だったか、飛び自体を見落としたかをクリップで確認します。",
    },
    tone: "defense",
  },
  own_jumps: {
    title: "ジャンプを繰り返し落とされている",
    description: "自分の前ジャンプを相手の対空で迎撃された場面です。",
    practice:
      "飛んだ理由を見直し、歩きガードやドライブラッシュなど地上の接近手段を増やします。",
    observation: {
      title: "ジャンプを落とされた場面",
      description: `自分のジャンプが迎撃された事実です。単発では試した読みか、接近手段の偏りかを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "飛んだ理由と、普段も同じ距離・タイミングで飛んでいるかを確認します。",
    },
    tone: "warning",
  },
  burnout: {
    title: "バーンアウト管理",
    description:
      "バーンアウト中の時間、被ダメージ、与ダメージと突入原因を見直す項目です。",
    practice:
      "突入直前を確認し、攻めのための消費か、守りで削り切られたかを分けて整理します。",
    tone: "resource",
  },
  committed_button_vs_di: {
    title: "通常技の実行中にDIを繰り返し受けている",
    description:
      "通常技の実行中に相手DIがヒットした場面が複数確認されています。",
    practice:
      "技が出始めた時点とDI演出開始の順序、その技のDIキャンセル可否を確認します。",
    observation: {
      title: "通常技の実行中にDIを受けた場面",
      description: `通常技の実行中に相手DIがヒットした事実です。単発では相手が技の出始めを見てDIしたのか、先に選んだDIとかみ合ったのかを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "スロー再生で技とDI演出の開始順、その技のDIキャンセル可否を確認し、置く距離とDI返しを分けて練習します。",
    },
    tone: "warning",
  },
  mashing: {
    title: "守勢でボタンを押して繰り返し被弾している",
    description:
      "相手の攻めを受けている最中にボタンを押し、複数回大きく被弾した場面です。",
    practice:
      "まず連係をガードだけで受け切り、確認できた切れ目だけを打ち返します。",
    observation: {
      title: "守勢でボタンを押して被弾した場面",
      description: `相手の攻めの途中でボタンを押して被弾した事実です。投げを読んだ単発の回答か、暴れの偏りかは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "押した理由を確認し、他の守勢でも同じタイミングで押しているか見比べます。",
    },
    tone: "critical",
  },
  press_while_minus: {
    title: "不利フレーム後の最速打撃に偏っている",
    description:
      "確認できた不利状況の多くで最速打撃を選び、複数回狩られたパターンです。",
    practice:
      "ガードを基準にし、遅らせ投げ抜けや後退も混ぜて回答を散らします。",
    observation: {
      title: "不利フレーム後の最速打撃で被弾した場面",
      description: `不利状況で最速打撃を選んで被弾した事実です。この結果だけでは、投げを読んだ打撃が偶然負けたのか、回答が偏っているのかは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "投げを読んだ意図的な回答だったか、同じ不利状況で無意識に押したかを確認します。",
    },
    tone: "critical",
  },
  throw_while_minus: {
    title: "不利フレーム後の最速投げに偏っている",
    description:
      "確認できた不利状況の多くで最速投げを選び、複数回打撃に負けたパターンです。",
    practice:
      "ガード、遅らせ投げ抜け、最速投げの勝ち負けを同じ連係で比較します。",
    observation: {
      title: "不利フレーム後の最速投げで被弾した場面",
      description: `不利状況で自分から最速投げを選んで被弾した事実です。偏り条件を満たしていないため癖とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "相手の投げを読んだ回答だったか、同じ状況で投げへ偏っていないかを確認します。",
    },
    tone: "critical",
  },
  advantage_abandoned: {
    title: "ガードさせて有利を取った後に攻めを継続できていない",
    description:
      "確認できた有利フレームの多くで次の攻撃を始めず、複数回そのまま攻め返されたパターンです。",
    practice:
      "同じ技をガードさせた状況から繋がる打撃と投げを1つずつ決め、有利を確認したら必ず出す形を作ります。",
    observation: {
      title: "有利フレームを取った後にターンを渡した場面",
      description: `有利を取った直後に攻めず、続けて相手の攻撃を受けた事実です。距離やゲージ回復を優先した選択の可能性もあるため、攻めの止まる癖とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "その時点の距離とドライブゲージを確認し、密着で止まっている場合だけ攻め継続を用意します。",
    },
    tone: "critical",
  },
  whiff_punished: {
    title: "届かない技の硬直を繰り返し狩られている",
    description:
      "相手へ接触しなかった技の硬直を複数回反撃され、技を置く距離とタイミングが崩れているパターンです。",
    practice:
      "主力技が届く距離と届かない距離の境目を確認し、その手前で振る形を作ります。",
    observation: {
      title: "空振りした技の硬直を狩られた場面",
      description: `接触しなかった技の硬直を反撃された事実です。間合いを測る空振りは差し合いの一部なので、この件数だけでは技を置く距離の癖とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "間合いを測る意図の空振りだったか、届くつもりで外したかを確認します。",
    },
    tone: "critical",
  },
  guard_break: {
    title: "同じ方向へガード入力が繰り返し崩れている",
    description:
      "相手の連係中に同じ方向へガードを離し、複数回被弾した場面です。",
    practice:
      "反撃を急がず、連係を最後までガードしてから動ける位置を確認します。",
    observation: {
      title: "ガード入力が外れて被弾した場面",
      description: `ガード方向から入力を変えた瞬間に被弾した事実です。単発では意図した読みか入力癖かを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "移動や反撃を意図した入力かを確認し、他の場面でも同じ方向へ外しているか見比べます。",
    },
    tone: "defense",
  },
  reversal_punished: {
    title: "無敵技という防御回答を繰り返し狩られている",
    description: "無敵技をガードまたは回避され、大きな反撃を受けた場面です。",
    practice:
      "起き上がりはガードを基準にし、相手の攻めが空いた状況だけ無敵技を選びます。",
    observation: {
      title: "無敵技を狩られた場面",
      description: `無敵技が通らず後隙を狩られた事実です。単発では正しい読みが外れたのか、選択が偏っているのかを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "打撃重ねを読んで撃ったかを確認し、他の同状況でも無敵技を選んでいるか見比べます。",
    },
    tone: "critical",
  },
  low_scaling_super: {
    title: "低い補正率でSA/CAを組み込んだ場面",
    description: `SA/CA投入時の補正率が低く、KOには至らなかった場面です。残り体力、画面位置、起き攻めを含めて使用目的を確認する対象であり、この事実だけで使用ミスとは${OBSERVATION_REVIEW_CAVEAT}。`,
    practice:
      "同じ始動のSAなし安定ルートと比較し、KO・端到達・有利状況のどれに寄与した使用かを確認します。",
    tone: "resource",
  },
  punish_fail: {
    title: "同じ反撃入力が繰り返し届いていない",
    description:
      "相手の大きな後隙へ反撃したものの、距離により届かなかった場面です。",
    practice: "密着と先端の距離を分け、それぞれで届く反撃を用意します。",
    observation: {
      title: "ガード後の反撃が届かなかった場面",
      description: `反撃が間に合う状況で攻撃したものの届かなかった事実です。単発では距離固有の選択か、反撃の癖かを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "同じ距離を再現し、その場面で実際に確定して届く技を確認します。",
    },
    tone: "warning",
  },
  punish_missed: {
    title: "確定反撃を見逃した場面",
    description: "距離と硬直から確定する反撃を返せなかった場面です。",
    practice:
      "よく見る技を少数に絞り、ガード後に最速で反撃する形を反復します。",
    tone: "warning",
  },
  low_conversion: {
    title: "同じ確反入力が小さいリターンで終わっている",
    description:
      "確定反撃は成功したものの、得られたダメージが小さかった場面です。",
    practice:
      "画面中央と端で、確定反撃から完走できる基本コンボを一つずつ用意します。",
    observation: {
      title: "確反が小さいリターンで終わった場面",
      description: `確反は成功したもののダメージが小さかった事実です。ゲージ温存・位置・KO状況による選択かもしれないため、リターン不足の癖とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "残り体力・ゲージ・位置を確認し、意図した温存でなければ短い基本コンボを確認します。",
    },
    tone: "warning",
  },
  throw_interrupted_by_invincible: {
    title: "投げに無敵技を繰り返し合わせられている",
    description:
      "投げを実行した直後に相手の無敵技が始まり、被弾した場面が複数確認されています。",
    practice:
      "同じ起き攻めで投げ・様子見・後退の選択率を確認し、投げに偏っている場合だけ無敵技を待つ選択を混ぜます。",
    observation: {
      title: "投げが相手の無敵技に負けた場面",
      description: `投げ実行直後に相手の無敵技が始まり、被弾した事実です。単発では投げ選択が不適切だったのか、無敵技がかみ合った読み負けかを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "直前の起き攻めまで戻り、投げ・様子見・後退のどれを普段選んでいるか見比べます。",
    },
    tone: "warning",
  },
  throw_whiff_punished: {
    title: "投げ空振りを繰り返して反撃を受けている",
    description:
      "実行まで確認できた投げ空振りの後、短時間内に被弾した場面が複数確認されています。",
    practice:
      "投げ入力時の距離と相手の後退を確認し、投げ間合い外では歩きガードへ戻します。",
    observation: {
      title: "投げ空振り後に被弾した場面",
      description: `投げ空振り後の短時間内に被弾した事実です。単発では相手の後退を読めなかったのか、別の読み合いの結果かを${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "スロー再生で投げ入力時の距離と相手の後退開始を確認し、同じ距離で投げる場面と見比べます。",
    },
    tone: "warning",
  },
  throw_loop: {
    title: "投げを連続して受けている",
    description: "短い時間内に相手の投げが連続して成立した場面です。",
    practice: "打撃重ねと投げをランダム再生し、遅らせ投げ抜けを練習します。",
    observation: {
      title: "投げを受けた場面",
      description: `相手の投げが成立した場面です。投げは打撃との読み合いなので、3連続未満では守り方の問題とは${OBSERVATION_REVIEW_CAVEAT}。`,
      practice:
        "打撃を警戒してガードを選んだ結果か、同じ守り方が続いていたかを確認します。",
    },
    tone: "defense",
  },
  early_hits: {
    title: "開幕に被弾したラウンド",
    description: `開幕3秒以内の被弾を並べた確認項目です。同じ初手の偏りかは${OBSERVATION_REVIEW_CAVEAT}。`,
    practice:
      "各ラウンドの初手を記録し、共通する行動がある場合だけ選択率を下げます。",
    tone: "warning",
  },
  lead_loss: {
    title: "大きなリードから逆転された場面",
    description: `大きなHPリードから逆転された区間です。特定の行動が原因とは${OBSERVATION_REVIEW_CAVEAT}。`,
    practice:
      "最大リード以降を見直し、同じ行動からの被弾が繰り返されているか確認します。",
    tone: "critical",
  },
  big_hits: {
    title: "原因を分類できなかった大ダメージ",
    description:
      "一度のコンボや連係で大きくHPを失い、他の原因別項目へ分類できなかった場面です。",
    practice: "被弾直前の行動を分類し、最も多い大被弾の入り口から修正します。",
    tone: "critical",
  },
};

export function characterName(id: CharacterId): string {
  return CHARACTER_NAMES[id];
}

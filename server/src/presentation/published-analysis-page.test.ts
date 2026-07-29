import { describe, expect, test } from "bun:test";
import {
  createPersistablePublishedAnalysis,
  createPublishedAnalysisContent,
  type DeletePasswordHash,
  FINDING_KINDS,
  type FindingAssessment,
  type ShareId,
} from "../models/published-analysis";
import { FINDING_PRESENTATIONS } from "./published-analysis-catalog";
import {
  renderPublishedAnalysisNotFoundPage,
  renderPublishedAnalysisPage,
} from "./published-analysis-page";
import { publishedAnalysisPageStyles } from "./published-analysis-page-styles";
import { PublishedAnalysisPageView } from "./published-analysis-page-view";

function analysis(
  rulesetVersion = 3,
  findings: Array<{
    kind: (typeof FINDING_KINDS)[number];
    assessment?: FindingAssessment;
    occurrences: number;
    severityBp: number;
  }> = [
    { kind: "anti_air", occurrences: 2, severityBp: 1200 },
    { kind: "big_hits", occurrences: 1, severityBp: 2500 },
  ],
) {
  const content = createPublishedAnalysisContent({
    rulesetVersion,
    ownCharacter: "LUKE",
    opponentCharacter: "CHUN_LI",
    rounds: { detected: 2, won: 1, lost: 1, unresolved: 0 },
    findings,
    tactics: {
      antiAir: { opportunities: 3, successes: 1, jumpInsAllowed: 2 },
      driveImpact: {
        faced: 1,
        returned: 1,
        blocked: 0,
        parried: 0,
        hit: 0,
        avoided: 0,
        unconfirmed: 0,
      },
      rawDriveRush: { faced: 2, defended: 1, hit: 1, unconfirmed: 0 },
      dashThrow: { faced: 1 },
      throwWhiff: { count: 2 },
      fastestChallenge: {
        opportunities: 4,
        strikeAttempts: 2,
        strikeLosses: 1,
        throwAttempts: 1,
        throwLosses: 1,
      },
      burnout: {
        count: 1,
        durationDeciseconds: 125,
        hpLostBp: 2000,
        hpDealtBp: 500,
        selfInitiated: 0,
        forced: 1,
        mixed: 0,
        unknown: 0,
      },
    },
  });
  if (!content.ok) throw new Error("invalid fixture");
  return createPersistablePublishedAnalysis({
    id: "Abcdefghijklmnopqrstu_" as ShareId,
    content: content.value,
    deletePasswordHash:
      "$argon2id$v=19$m=19456,t=2,p=1$afqgXENr3y/WCxW5FclnyO6NDY/hIjW2oVS12hgu3b8$Tn12OEC62ylqoD4wLt+6ou9Hq7medNra44FzjO9DlRM" as DeletePasswordHash,
    now: new Date("2026-07-13T00:00:00.000Z"),
    retentionDays: 365,
  }).analysis;
}

function analysisWithUnconfirmedTactics() {
  const value = analysis();
  return {
    ...value,
    content: {
      ...value.content,
      tactics: {
        ...value.content.tactics,
        antiAir: {
          opportunities: 0,
          successes: 0,
          jumpInsAllowed: 0,
        },
        driveImpact: {
          faced: 0,
          returned: 0,
          blocked: 0,
          parried: 0,
          hit: 0,
          avoided: 0,
          unconfirmed: 2,
        },
        rawDriveRush: {
          faced: 2,
          defended: 1,
          hit: 1,
          unconfirmed: 3,
        },
      },
    },
  };
}

describe("published analysis presentation", () => {
  test("確認なしと未確認候補を共有ページでも区別する", () => {
    const view = PublishedAnalysisPageView.from(
      analysisWithUnconfirmedTactics(),
      {
        canonical: new URL("https://fighter.example/s/example"),
        home: new URL("https://fighter.example/"),
        image: new URL("https://fighter.example/ogp.jpg"),
      },
    );

    expect(view.tactics.slice(0, 3)).toEqual([
      {
        value: "確認なし",
        label: "対空 成功 / 機会",
        detail: "飛びを通された 0回",
      },
      {
        value: "未確認 2 件",
        label: "DI返し / 相手DI",
        detail: "ガード 0・パリィ 0・被弾 0・未確認候補 2件",
      },
      {
        value: "1 / 2",
        label: "生ラッシュ対処 / 相手ラッシュ",
        detail: "被弾 1回・未確認候補 3件",
      },
    ]);
  });

  test("共有ページもSPAと同じsystem font roleを使用する", () => {
    const styles = publishedAnalysisPageStyles().toString();

    expect(styles).toContain(
      '--font-body: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI"',
    );
    expect(styles).toContain('"Hiragino Sans"');
    expect(styles).toContain(
      '--font-head: "Bahnschrift", "DIN Condensed", "Arial Narrow"',
    );
    expect(styles).toContain('"DejaVu Sans Condensed"');
    expect(styles).toContain("font-family: var(--font-body)");
    expect(styles).toContain("font-family: var(--font-head)");
    expect(styles).toContain("font-stretch: condensed");
    expect(styles).not.toMatch(/@font-face/i);
    expect(styles).not.toMatch(/fonts\.(?:googleapis|gstatic)\.com/i);
  });

  test("全FindingKindに固定文言がある", () => {
    expect(Object.keys(FINDING_PRESENTATIONS).sort()).toEqual(
      [...FINDING_KINDS].sort(),
    );
  });

  test("断定を避ける確認場面も利用者へ検討を促す", () => {
    const observationKinds = [
      "layered_defense",
      "anti_air",
      "own_jumps",
      "committed_button_vs_di",
      "mashing",
      "press_while_minus",
      "throw_while_minus",
      "guard_break",
      "reversal_punished",
      "punish_fail",
      "low_conversion",
      "throw_interrupted_by_invincible",
      "throw_whiff_punished",
      "throw_loop",
    ] as const;
    const descriptions = observationKinds.map(
      (kind) => FINDING_PRESENTATIONS[kind].observation?.description,
    );
    descriptions.push(
      FINDING_PRESENTATIONS.early_hits.description,
      FINDING_PRESENTATIONS.lead_loss.description,
    );

    for (const description of descriptions) {
      expect(description).toContain(
        "断定できませんが、検討の対象にしてもよいかもしれません",
      );
    }
    expect(JSON.stringify(FINDING_PRESENTATIONS)).not.toContain("断定しません");
    expect(JSON.stringify(FINDING_PRESENTATIONS)).not.toContain(
      "断定していません",
    );
  });

  test("JavaScriptなしの本文と結果固有OGPを生成する", () => {
    const value = analysis();
    const canonical = new URL(`https://fighter.example/s/${value.id}`);
    const rendered = renderPublishedAnalysisPage(value, {
      canonical,
      home: new URL("https://fighter.example/"),
      image: new URL("https://fighter.example/images/fighter-notes-ogp.jpg"),
    }).toString();
    expect(rendered).toContain("LUKE vs CHUN-LI 分析結果");
    expect(rendered).toContain("原因を分類できなかった大ダメージ");
    expect(rendered).toContain("飛び込みを繰り返し通している");
    expect(rendered).toContain('property="og:title"');
    expect(rendered).toContain(`content="${canonical.toString()}"`);
    expect(rendered).toContain('name="twitter:card"');
    expect(rendered).toContain("https://x.com/intent/tweet");
    expect(rendered).toContain("hashtags=FighterNotes");
    expect(rendered).toContain("Xに投稿");
    expect(rendered).toContain("動画データは含まれていません");
    expect(rendered).toContain(
      "解析結果は映像からの推定です。正確な記録ではなく、見直しのための参考情報としてご利用ください。",
    );
    expect(rendered).toContain(`/manage/${value.id}`);
    expect(rendered).toContain("この共有を削除");
    expect(rendered).toContain(
      "共有URLの作成時に発行された削除コードで期限前に削除できます",
    );
    expect(rendered).not.toContain("解析時に決めた削除用パスワード");
    expect(rendered).toContain('aria-label="サイト情報"');
    expect(rendered).toContain('href="https://fighter.example/privacy"');
    expect(rendered).toContain(">プライバシーポリシー</a>");
    expect(rendered).not.toContain('href="https://fighter.example/terms"');
    expect(rendered).not.toContain('href="https://fighter.example/legal"');
    expect(rendered).toContain('href="https://fighter.example/licenses"');
    expect(rendered).toContain(">使用コンポーネントのライセンス</a>");
    expect(rendered).not.toContain(
      "非公式ツール・株式会社カプコンとは無関係です",
    );
    expect(rendered).toContain("Created by Yuniruyuni");
    expect(rendered).toContain('href="https://yuniruyuni.net"');
    expect(rendered).toContain('rel="noopener noreferrer"');
    expect(rendered).not.toContain("© 2026 yuniruyuni");
    expect(rendered).not.toContain("<script");
    expect(rendered).not.toContain("frame");
  });

  test("表示モデルがURL・日付・戦術指標をHTMLから独立して組み立てる", () => {
    const value = analysis();
    const view = PublishedAnalysisPageView.from(value, {
      canonical: new URL(`https://fighter.example/s/${value.id}`),
      home: new URL("https://fighter.example/"),
      image: new URL("https://fighter.example/images/ogp.jpg"),
    });

    expect(view).toMatchObject({
      ownCharacter: "LUKE",
      opponentCharacter: "CHUN-LI",
      createdDate: "2026-07-13",
      expiresDate: "2027-07-13",
      managementUrl: `https://fighter.example/manage/${value.id}`,
    });
    expect(view.tactics[0]).toEqual({
      value: "1 / 3",
      label: "対空 成功 / 機会",
      detail: "飛びを通された 2回",
    });
    expect(view.tactics.at(-1)?.detail).toBe("HP収支 -15%");
  });

  test("ruleset v4 は原因診断を高severityの確認場面より先に出す", () => {
    const value = analysis(4);
    const rendered = renderPublishedAnalysisPage(value, {
      canonical: new URL(`https://fighter.example/s/${value.id}`),
      home: new URL("https://fighter.example/"),
      image: new URL("https://fighter.example/images/fighter-notes-ogp.jpg"),
    }).toString();

    expect(rendered.indexOf("飛び込みを繰り返し通している")).toBeLessThan(
      rendered.indexOf("原因を分類できなかった大ダメージ"),
    );
    expect(rendered).toContain("優先項目: 飛び込みを繰り返し通している");
    expect(rendered).toContain("原因診断・2件");
    expect(rendered).toContain("確認場面・1件");
  });

  test("ruleset v5 は開幕被弾と逆転区間を原因診断ではなく確認場面にする", () => {
    const value = analysis(5, [
      { kind: "early_hits", occurrences: 2, severityBp: 1200 },
      { kind: "lead_loss", occurrences: 1, severityBp: 3000 },
    ]);
    const rendered = renderPublishedAnalysisPage(value, {
      canonical: new URL(`https://fighter.example/s/${value.id}`),
      home: new URL("https://fighter.example/"),
      image: new URL("https://fighter.example/images/fighter-notes-ogp.jpg"),
    }).toString();

    expect(rendered).toContain("開幕に被弾したラウンド");
    expect(rendered).toContain("大きなリードから逆転された場面");
    expect(rendered).toContain("確認場面・2件");
    expect(rendered).not.toContain("原因診断・2件");
  });

  test("ruleset v6 は同じfinding IDでも保存された判定区分で表示する", () => {
    const value = analysis(6, [
      {
        kind: "press_while_minus",
        assessment: "observation",
        occurrences: 1,
        severityBp: 5000,
      },
      {
        kind: "punish_missed",
        assessment: "diagnosis",
        occurrences: 1,
        severityBp: 100,
      },
    ]);
    const rendered = renderPublishedAnalysisPage(value, {
      canonical: new URL(`https://fighter.example/s/${value.id}`),
      home: new URL("https://fighter.example/"),
      image: new URL("https://fighter.example/images/fighter-notes-ogp.jpg"),
    }).toString();

    expect(rendered.indexOf("確定反撃を見逃した場面")).toBeLessThan(
      rendered.indexOf("不利フレーム後の最速打撃で被弾した場面"),
    );
    expect(rendered).toContain("原因診断・1件");
    expect(rendered).toContain("確認場面・1件");
    expect(rendered).toContain(
      "断定できませんが、検討の対象にしてもよいかもしれません",
    );
  });

  test("404ページは存在しないID・削除済み・期限切れを区別しない", () => {
    const rendered = renderPublishedAnalysisNotFoundPage(
      new URL("https://fighter.example/"),
    ).toString();
    expect(rendered).toContain("共有結果が見つかりません");
    expect(rendered).toContain('href="https://fighter.example/privacy"');
    expect(rendered).toContain('href="https://fighter.example/licenses"');
    expect(rendered).toContain('href="https://yuniruyuni.net"');
    expect(rendered).not.toContain("期限切れ");
    expect(rendered).not.toContain("存在しないID");
  });
});

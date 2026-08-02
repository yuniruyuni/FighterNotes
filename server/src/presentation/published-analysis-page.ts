import { html } from "hono/html";
import type { PublishedAnalysis } from "../models/published-analysis";
import {
  renderFindings,
  renderPublishedAnalysisErrorPage,
  renderRoundMetrics,
  renderSiteFooter,
  renderSiteHeader,
  renderTactics,
} from "./published-analysis-page-sections";
import { publishedAnalysisPageStyles } from "./published-analysis-page-styles";
import {
  type PublishedAnalysisPageUrls,
  PublishedAnalysisPageView,
} from "./published-analysis-page-view";

export function renderPublishedAnalysisPage(
  analysis: PublishedAnalysis,
  urls: PublishedAnalysisPageUrls,
) {
  const view = PublishedAnalysisPageView.from(analysis, urls);

  return html`<!doctype html>
    <html lang="ja">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>${view.title}</title>
        <meta name="description" content="${view.description}" />
        <meta name="robots" content="noindex, follow" />
        <link rel="canonical" href="${view.canonicalUrl}" />
        <meta property="og:type" content="website" />
        <meta property="og:site_name" content="Fighter Notes" />
        <meta property="og:title" content="${view.title}" />
        <meta property="og:description" content="${view.description}" />
        <meta property="og:url" content="${view.canonicalUrl}" />
        <meta property="og:image" content="${view.imageUrl}" />
        <meta property="og:image:type" content="image/jpeg" />
        <meta property="og:image:width" content="1200" />
        <meta property="og:image:height" content="630" />
        <meta
          property="og:image:alt"
          content="Fighter Notes SF6 Replay Analyzer"
        />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content="${view.title}" />
        <meta name="twitter:description" content="${view.description}" />
        <meta name="twitter:image" content="${view.imageUrl}" />
        <meta
          name="twitter:image:alt"
          content="Fighter Notes SF6 Replay Analyzer"
        />
        ${publishedAnalysisPageStyles()}
      </head>
      <body>
        ${renderSiteHeader(view.homeUrl)}

        <main>
          <section class="matchup" aria-labelledby="result-title">
            <p class="eyebrow">SHARED ANALYSIS</p>
            <h1 id="result-title">
              <span>${view.ownCharacter}</span>
              <small>VS</small>
              <span>${view.opponentCharacter}</span>
            </h1>
            <p class="result-lead">共有された対戦分析結果</p>
            <p class="result-caveat">
              解析結果は映像からの推定です。正確な記録ではなく、見直しのための参考情報としてご利用ください。
            </p>
            ${renderRoundMetrics(view.rounds)}
          </section>

          <section class="content-band" aria-labelledby="findings-title">
            <div class="section-heading">
              <div>
                <p class="eyebrow">PRIORITY REVIEW</p>
                <h2 id="findings-title">主要な指摘</h2>
              </div>
              <span class="section-count">${view.findings.length}項目</span>
            </div>
            <div class="finding-list">${renderFindings(view.findings)}</div>
          </section>

          <section class="content-band" aria-labelledby="tactics-title">
            <div class="section-heading">
              <div>
                <p class="eyebrow">TACTICAL BREAKDOWN</p>
                <h2 id="tactics-title">戦術別の結果</h2>
              </div>
            </div>
            ${renderTactics(view.tactics)}
          </section>

          ${
            view.superArts === undefined
              ? ""
              : html`<section
                class="content-band"
                aria-labelledby="super-arts-title"
              >
                <div class="section-heading">
                  <div>
                    <p class="eyebrow">SUPER ART BREAKDOWN</p>
                    <h2 id="super-arts-title">SA / CA 集計</h2>
                  </div>
                </div>
                ${renderTactics(view.superArts)}
              </section>`
          }

          <section class="share-note">
            <div>
              <p class="eyebrow">PRIVACY</p>
              <h2>動画データは含まれていません</h2>
              <p>
                このページに保存されているのは、キャラクター、件数、公開対象の集計値だけです。
                元動画、場面クリップ、入力履歴、ファイル名、フレーム番号、自由文、SA/CAの正確なダメージ値と最終ゲージ量は保存されていません。
              </p>
              <p class="date-line">
                作成 ${view.createdDate} ・ 公開期限 ${view.expiresDate}
              </p>
              <p class="date-line">
                作成者は共有URLの作成時に発行された削除コードで期限前に削除できます。
              </p>
            </div>
            <div class="share-actions">
              <a
                class="command command-secondary"
                href="${view.managementUrl}"
                >この共有を削除</a
              >
              <a
                class="command command-secondary"
                href="${view.xIntentUrl}"
                target="_blank"
                rel="noopener noreferrer"
                >Xに投稿</a
              >
              <a class="command command-primary" href="${view.homeUrl}"
                >自分の動画を解析</a
              >
            </div>
          </section>

          <figure class="brand-media">
            <img
              src="${view.imageUrl}"
              alt="Fighter Notes SF6 Replay Analyzer"
              width="1200"
              height="630"
            />
          </figure>
        </main>

        ${renderSiteFooter(urls.home, `Ruleset ${view.rulesetVersion}`)}
      </body>
    </html>`;
}

export function renderPublishedAnalysisNotFoundPage(home: URL) {
  return renderPublishedAnalysisErrorPage(home, {
    title: "共有結果が見つかりません",
    eyebrow: "SHARED ANALYSIS",
    message: "URLが無効か、削除済み、または共有期限が終了した結果です。",
  });
}

export function renderPublishedAnalysisUnavailablePage(home: URL) {
  return renderPublishedAnalysisErrorPage(home, {
    title: "共有結果を読み込めません",
    eyebrow: "TEMPORARILY UNAVAILABLE",
    message: "時間を置いて、同じURLをもう一度開いてください。",
  });
}

import { html } from "hono/html";
import type { PublishedFindingView } from "./published-analysis-finding-view";
import { publishedAnalysisPageStyles } from "./published-analysis-page-styles";
import type { PublishedAnalysisPageView } from "./published-analysis-page-view";
import type { PublishedTacticView } from "./published-analysis-tactic-view";

export function renderSiteHeader(homeUrl: string) {
  return html`<header class="site-header">
      <a class="brand" href="${homeUrl}">Fighter Notes</a>
      <span>SF6 REPLAY ANALYZER</span>
    </header>
    <div class="accent-line"></div>`;
}

export function renderSiteFooter(home: URL, detail?: string) {
  const links = [
    ["プライバシーポリシー", new URL("/privacy", home).toString()],
    ["使用コンポーネントのライセンス", new URL("/licenses", home).toString()],
  ] as const;
  return html`<footer>
    <div class="footer-meta">
      <span>
        Created by Yuniruyuni —
        <a
          href="https://yuniruyuni.net"
          target="_blank"
          rel="noopener noreferrer"
          >yuniruyuni.net</a
        >
      </span>
      ${detail ? html`<span>${detail}</span>` : ""}
    </div>
    <nav aria-label="サイト情報">
      ${links.map(([label, href]) => html`<a href="${href}">${label}</a>`)}
    </nav>
  </footer>`;
}

export function renderRoundMetrics(
  rounds: PublishedAnalysisPageView["rounds"],
) {
  return html`<dl class="round-strip">
    ${renderMetric("検出ラウンド", rounds.detected)}
    ${renderMetric("WIN", rounds.won, "positive")}
    ${renderMetric("LOSE", rounds.lost, "negative")}
    ${renderMetric("判定保留", rounds.unresolved)}
  </dl>`;
}

export function renderFindings(findings: readonly PublishedFindingView[]) {
  if (findings.length === 0) {
    return html`<p class="empty-state">
      顕著な改善ポイントは検出されませんでした。
    </p>`;
  }
  return findings.map(
    (finding) => html`<article class="finding finding-${finding.tone}">
      <div class="finding-index">${finding.index}</div>
      <div class="finding-content">
        <div class="finding-title-row">
          <h3>${finding.title}</h3>
          <span>${finding.count}</span>
        </div>
        <p>${finding.description}</p>
        <p class="practice">
          <strong>見直し方</strong>${finding.practice}
        </p>
      </div>
    </article>`,
  );
}

export function renderTactics(tactics: readonly PublishedTacticView[]) {
  return html`<div class="tactic-grid">
    ${tactics.map(
      (item) => html`<div class="tactic-item">
        <strong>${item.value}</strong>
        <span>${item.label}</span>
        ${item.detail ? html`<small>${item.detail}</small>` : ""}
      </div>`,
    )}
  </div>`;
}

interface ErrorPageOptions {
  readonly title: string;
  readonly eyebrow: string;
  readonly message: string;
}

export function renderPublishedAnalysisErrorPage(
  home: URL,
  options: ErrorPageOptions,
) {
  const homeUrl = home.toString();
  return html`<!doctype html>
    <html lang="ja">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="robots" content="noindex, nofollow" />
        <title>${options.title} | Fighter Notes</title>
        ${publishedAnalysisPageStyles()}
      </head>
      <body>
        ${renderSiteHeader(homeUrl)}
        <main class="error-layout">
          <p class="eyebrow">${options.eyebrow}</p>
          <h1>${options.title}</h1>
          <p>${options.message}</p>
          <a class="command command-primary" href="${homeUrl}"
            >Fighter Notesへ戻る</a
          >
        </main>
        ${renderSiteFooter(home)}
      </body>
    </html>`;
}

function renderMetric(label: string, value: number, tone = "") {
  return html`<div class="round-metric ${tone}">
    <dt>${label}</dt>
    <dd>${value}</dd>
  </div>`;
}

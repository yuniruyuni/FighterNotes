import { html } from "hono/html";

export function publishedAnalysisPageStyles() {
  return html`<style>
    :root {
      color-scheme: dark;
      --bg: #0a0a0b;
      --surface: #151518;
      --surface-2: #1c1c20;
      --line: #34343a;
      --text: #f2f2f4;
      --muted: #a0a0a8;
      --purple: #a65af5;
      --yellow: #dfbc23;
      --green: #00d889;
      --red: #ff4f79;
      --blue: #4da9ff;
      --font-body: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
        "Hiragino Sans", "Yu Gothic UI", "Yu Gothic", "Noto Sans CJK JP",
        sans-serif;
      --font-head: "Bahnschrift", "DIN Condensed", "Arial Narrow",
        "Aptos Narrow", "Roboto Condensed", "DejaVu Sans Condensed",
        "Helvetica Neue", Arial, system-ui, sans-serif;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      min-width: 320px;
      background: var(--bg);
      color: var(--text);
      font-family: var(--font-body);
      letter-spacing: 0;
    }
    button, input, select, textarea { font-family: var(--font-body); }
    a { color: inherit; }
    .brand, .eyebrow, h1, h2, .round-metric dd, .finding-index,
    .tactic-item strong, .command {
      font-family: var(--font-head);
      font-stretch: condensed;
    }
    .round-metric dd, .finding-index, .section-count,
    .finding-title-row span, .tactic-item strong, .date-line {
      font-variant-numeric: tabular-nums;
    }
    .site-header {
      min-height: 58px;
      padding: 0 28px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      border-bottom: 1px solid #202024;
      background: #050506;
    }
    .brand {
      color: #fff;
      font-size: 18px;
      font-weight: 800;
      text-decoration: none;
      text-transform: uppercase;
    }
    .site-header span, footer span {
      color: var(--muted);
      font-size: 11px;
      font-weight: 700;
    }
    .accent-line {
      height: 5px;
      background: linear-gradient(90deg, #5312c9, #9e45f5 45%, #dfbc23);
    }
    main { width: 100%; }
    .matchup {
      max-width: 1100px;
      min-height: 360px;
      margin: 0 auto;
      padding: 66px 28px 52px;
      display: flex;
      flex-direction: column;
      justify-content: center;
    }
    .eyebrow {
      margin: 0 0 10px;
      color: var(--yellow);
      font-size: 11px;
      font-weight: 800;
    }
    h1 {
      margin: 0;
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
      align-items: baseline;
      gap: 22px;
      font-size: 64px;
      line-height: 1.05;
      text-transform: uppercase;
    }
    h1 > span { min-width: 0; overflow-wrap: anywhere; }
    h1 > span:last-child { text-align: right; }
    h1 small { color: var(--purple); font-size: 20px; }
    .result-lead { margin: 16px 0 0; color: var(--muted); }
    .result-caveat {
      max-width: 720px;
      margin: 8px 0 0;
      color: var(--muted);
      font-size: 12px;
      line-height: 1.6;
    }
    .round-strip {
      width: min(100%, 720px);
      margin: 40px 0 0;
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      border-block: 1px solid var(--line);
    }
    .round-metric { padding: 16px; border-right: 1px solid var(--line); }
    .round-metric:last-child { border-right: 0; }
    .round-metric dt { color: var(--muted); font-size: 11px; }
    .round-metric dd { margin: 4px 0 0; font-size: 26px; font-weight: 800; }
    .round-metric.positive dd { color: var(--green); }
    .round-metric.negative dd { color: var(--red); }
    .content-band {
      padding: 54px max(28px, calc((100vw - 1044px) / 2));
      border-top: 1px solid var(--line);
      background: var(--surface);
    }
    .content-band:nth-of-type(3) { background: #101012; }
    .section-heading {
      margin-bottom: 24px;
      display: flex;
      align-items: end;
      justify-content: space-between;
      gap: 20px;
    }
    h2 { margin: 0; font-size: 27px; }
    .section-count { color: var(--muted); font-size: 13px; }
    .finding-list { display: grid; gap: 10px; }
    .finding {
      min-width: 0;
      display: grid;
      grid-template-columns: 62px minmax(0, 1fr);
      border: 1px solid var(--line);
      border-left: 4px solid var(--red);
      border-radius: 4px;
      background: var(--surface-2);
    }
    .finding-warning { border-left-color: var(--yellow); }
    .finding-defense { border-left-color: var(--blue); }
    .finding-resource { border-left-color: var(--green); }
    .finding-index {
      padding: 22px 10px;
      color: #74747c;
      font-size: 13px;
      font-weight: 800;
      text-align: center;
      border-right: 1px solid var(--line);
    }
    .finding-content { min-width: 0; padding: 19px 22px; }
    .finding-title-row {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      gap: 16px;
    }
    .finding h3 { margin: 0; font-size: 17px; }
    .finding-title-row span {
      flex: 0 0 auto;
      color: var(--muted);
      font-size: 12px;
    }
    .finding p { margin: 8px 0 0; color: #c4c4ca; font-size: 14px; line-height: 1.65; }
    .finding .practice { color: #a9d8c5; }
    .practice strong { margin-right: 10px; color: var(--green); font-size: 11px; }
    .empty-state { margin: 0; padding: 32px 0; color: var(--muted); }
    .tactic-grid {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 1px;
      background: var(--line);
      border: 1px solid var(--line);
    }
    .tactic-item {
      min-height: 126px;
      padding: 20px;
      display: flex;
      flex-direction: column;
      background: var(--surface);
    }
    .tactic-item strong { font-size: 24px; }
    .tactic-item span { margin-top: 8px; color: var(--muted); font-size: 12px; }
    .tactic-item small { margin-top: auto; padding-top: 12px; color: #74747c; font-size: 10px; }
    .share-note {
      max-width: 1044px;
      margin: 0 auto;
      padding: 54px 28px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 44px;
    }
    .share-note > div:first-child { max-width: 650px; }
    .share-note h2 { font-size: 22px; }
    .share-note p:not(.eyebrow) { color: var(--muted); font-size: 13px; line-height: 1.7; }
    .share-actions { display: flex; flex-wrap: wrap; gap: 10px; }
    .command {
      min-height: 44px;
      padding: 0 18px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border: 1px solid var(--line);
      border-radius: 3px;
      font-size: 13px;
      font-weight: 800;
      text-decoration: none;
    }
    .command-primary { background: var(--yellow); border-color: var(--yellow); color: #111; }
    .command-secondary { background: #fff; border-color: #fff; color: #111; }
    .brand-media {
      max-width: 1044px;
      margin: 0 auto 64px;
      padding: 0 28px;
    }
    .brand-media img {
      width: min(100%, 560px);
      height: auto;
      display: block;
      border: 1px solid var(--line);
    }
    footer {
      min-height: 92px;
      padding: 18px max(28px, calc((100vw - 988px) / 2));
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 24px;
      border-top: 1px solid var(--line);
      background: #050506;
    }
    footer a { color: var(--muted); text-decoration: none; }
    footer a:hover { color: var(--text); }
    .footer-meta {
      display: flex;
      align-items: center;
      flex-wrap: wrap;
      gap: 10px 18px;
    }
    footer nav {
      display: flex;
      flex-wrap: wrap;
      justify-content: flex-end;
      gap: 8px 14px;
      margin-left: auto;
      text-align: right;
      font-size: 11px;
    }
    .footer-meta { justify-content: flex-start; font-size: 11px; }
    .error-layout {
      max-width: 840px;
      min-height: calc(100vh - 63px);
      margin: 0 auto;
      padding: 90px 28px;
    }
    .error-layout h1 { display: block; font-size: 48px; }
    .error-layout > p:not(.eyebrow) { margin: 18px 0 30px; color: var(--muted); }
    @media (max-width: 760px) {
      .site-header { padding: 0 16px; }
      .site-header span { display: none; }
      .matchup { min-height: 320px; padding: 48px 18px 38px; }
      h1 { gap: 10px; font-size: 32px; }
      h1 small { font-size: 14px; }
      .round-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .round-metric:nth-child(2) { border-right: 0; }
      .round-metric:nth-child(-n + 2) { border-bottom: 1px solid var(--line); }
      .content-band { padding: 40px 18px; }
      .finding { grid-template-columns: 42px minmax(0, 1fr); }
      .finding-index { padding-inline: 5px; }
      .finding-content { padding: 16px; }
      .finding-title-row { align-items: flex-start; }
      .tactic-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
      .tactic-item { min-height: 116px; padding: 15px; }
      .share-note { padding: 42px 18px; align-items: flex-start; flex-direction: column; }
      .share-actions, .command { width: 100%; }
      .brand-media { margin-bottom: 42px; padding: 0 18px; }
      footer {
        padding: 18px;
        align-items: stretch;
        flex-direction: column;
      }
      footer nav { align-self: flex-end; margin-left: 0; }
    }
    @media (max-width: 480px) {
      h1 { grid-template-columns: minmax(0, 1fr); gap: 6px; font-size: 28px; }
      h1 > span:last-child { text-align: left; }
    }
    @media (max-width: 380px) {
      .error-layout h1 { font-size: 30px; }
    }
  </style>`;
}

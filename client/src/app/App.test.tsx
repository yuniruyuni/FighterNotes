import { describe, expect, test } from "bun:test";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Router } from "wouter";
import { memoryLocation } from "wouter/memory-location";
import { App } from "./App";

function renderAt(path: string) {
  const location = memoryLocation({ path, record: true });
  render(
    <Router hook={location.hook}>
      <App />
    </Router>,
  );
  return location;
}

describe("React frontend routes", () => {
  test("rootでは公開せずに開始できる解析セットアップを表示する", () => {
    renderAt("/");

    expect(
      screen.getByRole("heading", { name: /Fighter Notes/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "解析する" })).toBeDisabled();
    expect(
      screen.getByText(/解析はすべてこのブラウザの中だけ/),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/解析完了時に公開URLを自動で作成します/),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByText(/指摘と戦術統計を30日間公開/),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "サイト情報" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "利用規約" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "運営者・権利表記" }),
    ).not.toBeInTheDocument();
    expect(screen.getByText(/録画・利用してよい動画だけ/)).toBeInTheDocument();
    const privacyLinks = screen.getAllByRole("link", {
      name: "プライバシーポリシー",
    });
    expect(privacyLinks).toHaveLength(1);
    expect(
      privacyLinks.every((link) => link.getAttribute("href") === "/privacy"),
    ).toBe(true);
    expect(
      screen.getByText(
        /被弾場面・弱点・練習メニューを自動で抽出する、カプコン非公式の個人開発ツールです。/,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("非公式ツール・株式会社カプコンとは無関係です"),
    ).not.toBeInTheDocument();
    const authorLink = screen.getByRole("link", { name: "yuniruyuni.net" });
    expect(authorLink).toHaveAttribute("href", "https://yuniruyuni.net");
    expect(authorLink).toHaveAttribute("target", "_blank");
    expect(authorLink).toHaveAttribute("rel", "noopener noreferrer");
    expect(authorLink.parentElement).toHaveTextContent(
      "Created by Yuniruyuni — yuniruyuni.net",
    );
    expect(screen.queryByText(/© 2026 yuniruyuni/)).not.toBeInTheDocument();
    expect(screen.getByLabelText(/自分のキャラクター/)).toBeRequired();
    expect(screen.getByLabelText(/相手のキャラクター/)).toBeRequired();
  });

  test("動画と両キャラクターを指定すると解析を開始できる", () => {
    renderAt("/");
    const fileInput = document.querySelector<HTMLInputElement>("#file-input");
    expect(fileInput).not.toBeNull();

    fireEvent.change(fileInput!, {
      target: {
        files: [new File(["video"], "replay.mp4", { type: "video/mp4" })],
      },
    });
    fireEvent.change(document.querySelector("#side-select")!, {
      target: { value: "p1" },
    });
    fireEvent.change(document.querySelector("#char-select")!, {
      target: { value: "JURI" },
    });
    fireEvent.change(document.querySelector("#opponent-char-select")!, {
      target: { value: "KEN" },
    });
    expect(document.querySelector(".selected-file-name")).toHaveTextContent(
      "replay.mp4",
    );
    expect(
      document.querySelector<HTMLButtonElement>(".analyze-btn")?.disabled,
    ).toBe(false);
  });

  test("信頼されないHTTP接続では解析を無効化して理由を表示する", () => {
    const descriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      "isSecureContext",
    );
    Object.defineProperty(globalThis, "isSecureContext", {
      configurable: true,
      value: false,
    });

    try {
      renderAt("/");
      expect(screen.getByRole("alert")).toHaveTextContent(
        /HTTPSまたはlocalhost/,
      );
      expect(screen.getByRole("button", { name: "解析する" })).toBeDisabled();
    } finally {
      if (descriptor) {
        Object.defineProperty(globalThis, "isSecureContext", descriptor);
      } else {
        Reflect.deleteProperty(globalThis, "isSecureContext");
      }
    }
  });

  test("VideoDecoder非対応ブラウザでは解析を無効化して理由を表示する", () => {
    const descriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      "VideoDecoder",
    );
    Reflect.deleteProperty(globalThis, "VideoDecoder");

    try {
      renderAt("/");
      expect(screen.getByRole("alert")).toHaveTextContent(
        /WebCodecs VideoDecoder/,
      );
      expect(screen.getByRole("button", { name: "解析する" })).toBeDisabled();
    } finally {
      if (descriptor) {
        Object.defineProperty(globalThis, "VideoDecoder", descriptor);
      }
    }
  });

  test("manage routeで端末内共有と手動削除を提供する", () => {
    renderAt("/manage");

    expect(
      screen.getByRole("heading", { name: "公開した分析結果を管理" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "この端末で作成した共有" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("共有URLまたは共有ID")).toBeRequired();
    const deleteCode = screen.getByLabelText("削除コード");
    expect(deleteCode).toHaveAttribute("minlength", "12");
    expect(deleteCode).toHaveAttribute("maxlength", "128");
  });

  test("manage/:id routeで共有IDを入力済みにする", () => {
    const id = "Abcdefghijklmnopqrstu_";
    renderAt(`/manage/${id}`);

    expect(screen.getByLabelText("共有URLまたは共有ID")).toHaveValue(id);
  });

  test("privacy routeでブラウザ内解析と公開共有の取扱いを説明する", () => {
    renderAt("/privacy");

    expect(
      screen.getByRole("heading", { name: "プライバシーポリシー" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "1. アカウントと登録情報" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "3. 解析結果の共有" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "4. ブラウザに保存する情報" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "5. 接続情報と外部サービス" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "6. 本ポリシーの変更" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/利用者の端末内で解析します/)).toBeInTheDocument();
    expect(
      screen.getByText(/原則として作成から30日間保存し/),
    ).toBeInTheDocument();
    expect(screen.getByText(/文書バージョン: 1.0/)).toBeInTheDocument();
    expect(
      screen.getByText(/解析だけを行う場合、解析結果はサーバーへ送信されず/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/「共有URLを生成」を選んだ場合に限り/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/動画本体や動画ファイル名は履歴に保存されません/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /IPアドレス、アクセス日時、ブラウザ情報、閲覧したページ.*処理されます/,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("link", {
        name: "Cloudflareのプライバシーポリシー",
      }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", {
        name: "Google Cloudのプライバシーに関するお知らせ",
      }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(/サイトの配信と保護にCloudflare/),
    ).toHaveTextContent("Google Cloud");
    expect(
      screen.getByText(
        /行動追跡を目的とするアクセス解析ツールを導入していません/,
      ),
    ).toBeInTheDocument();
    expect(document.body).not.toHaveTextContent(
      /PostgreSQL|IndexedDB|localStorage|Argon2id|cf-connecting-ip/,
    );
    expect(screen.getByText("制定日: 2026年7月25日")).toBeInTheDocument();
    expect(screen.getByText(/最終更新:/)).toHaveTextContent("2026年7月25日");
    expect(document.body).not.toHaveTextContent("最終改定日");
    expect(document.body).not.toHaveTextContent(
      "削除コードそのものは、サーバーに保存しません",
    );
    expect(
      screen.queryByRole("link", { name: "GitHub の非公開報告フォーム" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: /連絡|問い合わせ/ }),
    ).not.toBeInTheDocument();
  });

  test("廃止したterms routeはnot foundを表示する", () => {
    renderAt("/terms");

    expect(
      screen.getByRole("heading", { name: "ページが見つかりません" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "利用規約" }),
    ).not.toBeInTheDocument();
  });

  test("廃止したlegal routeはnot foundを表示する", () => {
    renderAt("/legal");

    expect(
      screen.getByRole("heading", { name: "ページが見つかりません" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "運営者・権利表記" }),
    ).not.toBeInTheDocument();
  });

  test("licenses routeで各依存関係の直下にlicense全文を展開する", async () => {
    renderAt("/licenses");

    expect(
      screen.getByRole("heading", {
        name: "使用コンポーネントのライセンス",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText(/最終更新:/)).toHaveTextContent("2026年7月24日");
    expect(
      screen.getByRole("heading", { name: /コンポーネント一覧（\d+件）/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "このページについて" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/本サービスのアプリケーションには、画面表示、動画解析/),
    ).toBeInTheDocument();
    expect(screen.queryByText(/bun\.lock/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Cargo\.lock/)).not.toBeInTheDocument();
    expect(
      screen.getByRole("columnheader", { name: "著作者・権利表示" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: /ライセンス全文（\d+件）/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "THIRD_PARTY_NOTICES" }),
    ).toHaveAttribute("href", "/THIRD_PARTY_NOTICES.txt");
    expect(
      screen.queryByRole("heading", { name: "Fighter Notes 本体" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("link", { name: "LICENSE" }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "配布素材・生成物の棚卸し" }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/System font stacks/)).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Analyzer data and recognition models/),
    ).not.toBeInTheDocument();
    expect(
      screen.getAllByText(/Copyright \(c\) Meta Platforms/).length,
    ).toBeGreaterThan(0);
    const reactLink = screen.getByRole("link", { name: "react" });
    const reactRow = reactLink.closest("tr");
    expect(reactRow).not.toBeNull();
    const expandButton = within(reactRow as HTMLTableRowElement).getByRole(
      "button",
      { name: "MIT" },
    );
    expect(
      within(reactRow as HTMLTableRowElement).queryByRole("button", {
        name: "ライセンス全文",
      }),
    ).not.toBeInTheDocument();
    expect(
      expandButton.querySelector(".license-document-marker"),
    ).toHaveAttribute("aria-hidden", "true");
    expect(within(expandButton).getByText("MIT")).toHaveClass(
      "license-document-label",
    );
    const attributionCell = reactRow?.querySelector(
      ".license-attribution-cell",
    );
    expect(attributionCell).toHaveTextContent(
      "Copyright (c) Meta Platforms, Inc. and affiliates.",
    );
    expect(expandButton.closest("td")).not.toHaveTextContent(
      "Copyright (c) Meta Platforms, Inc. and affiliates.",
    );
    expect(expandButton).toHaveAttribute("aria-expanded", "false");
    expect(reactRow).not.toHaveTextContent("採用:");
    expect(document.querySelector(".license-text-panel")).toBeNull();

    const user = userEvent.setup();
    await user.click(expandButton);

    expect(expandButton).toHaveAttribute("aria-expanded", "true");
    const panelId = expandButton.getAttribute("aria-controls") ?? "";
    const panelRow = document.getElementById(panelId);
    expect(panelRow).not.toBeNull();
    expect(reactRow?.nextElementSibling).toBe(panelRow);
    expect(panelRow).toHaveTextContent("Permission is hereby granted");

    await user.click(
      within(panelRow as HTMLTableRowElement).getByRole("button", {
        name: "閉じる",
      }),
    );
    expect(document.getElementById(panelId)).toBeNull();

    const bumpaloRow = screen
      .getByRole("link", { name: "bumpalo" })
      .closest("tr");
    expect(bumpaloRow).not.toBeNull();
    await user.click(
      within(bumpaloRow as HTMLTableRowElement).getByRole("button", {
        name: "MIT OR Apache-2.0",
      }),
    );
    const bumpaloPanel = bumpaloRow?.nextElementSibling;
    expect(bumpaloPanel).toHaveTextContent("宣言ライセンス: MIT OR Apache-2.0");
    expect(bumpaloPanel).not.toHaveTextContent("採用ライセンス");
    expect(bumpaloPanel).toHaveTextContent(
      "パッケージ同梱ファイル: LICENSE-APACHE",
    );
    expect(bumpaloPanel).toHaveTextContent(
      "パッケージ同梱ファイル: LICENSE-MIT",
    );
    expect(bumpaloPanel).toHaveTextContent(
      "Copyright (c) 2019 Nick Fitzgerald",
    );
    expect(bumpaloPanel).toHaveTextContent(
      "Apache-2.0本文末尾に含まれる適用例",
    );
    expect(bumpaloPanel?.querySelectorAll("pre")).toHaveLength(2);

    const unicodeIdentRow = screen
      .getByRole("link", { name: "unicode-ident" })
      .closest("tr");
    expect(unicodeIdentRow).not.toBeNull();
    await user.click(
      within(unicodeIdentRow as HTMLTableRowElement).getByRole("button", {
        name: "(MIT OR Apache-2.0) AND Unicode-3.0",
      }),
    );
    expect(
      unicodeIdentRow?.nextElementSibling?.querySelectorAll("pre"),
    ).toHaveLength(3);
  });

  test("不明なrouteはnot foundを表示する", () => {
    renderAt("/not-found");
    expect(
      screen.getByRole("heading", { name: "ページが見つかりません" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", {
        name: "使用コンポーネントのライセンス",
      }),
    ).toHaveAttribute("href", "/licenses");
  });
});

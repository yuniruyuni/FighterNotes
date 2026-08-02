import { describe, expect, mock, test } from "bun:test";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Router } from "wouter";
import { memoryLocation } from "wouter/memory-location";
import type { AnalysisContext } from "~/modules/analysis/contracts.js";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import type { SharingServices } from "../application/ports.js";
import { PublicationProvider } from "./PublicationProvider.js";
import { SharePanel } from "./SharePanel.js";
import { SharingServicesProvider } from "./SharingServicesProvider.js";

const context: AnalysisContext = {
  ownSide: "p1",
  p1: { character: "JURI" },
  p2: { character: "KEN" },
};

describe("SharePanel", () => {
  test("解析結果は明示操作まで送信せず、共有URL生成時にだけ送信する", async () => {
    const create = mock(async () => ({
      id: "Abcdefghijklmnopqrstu_",
      url: "https://fighter.example/s/Abcdefghijklmnopqrstu_",
      expiresAt: "2026-08-23T00:00:00.000Z",
    }));
    const services: SharingServices = {
      gateway: {
        create,
        delete: async () => undefined,
        errorMessage: () => "共有URLを作成できませんでした。",
      },
      managedShares: {
        save: () => true,
        load: () => ({ available: true, shares: [] }),
        remove: () => true,
        subscribe: () => () => undefined,
      },
      capabilities: {
        copyText: async () => undefined,
        canShare: () => false,
        share: async () => undefined,
        confirm: () => true,
        origin: () => "https://fighter.example",
        isCancelledShare: () => false,
      },
      generateDeleteCode: () => "ABCD-EFGH-JKLM",
      now: () => new Date("2026-07-24T00:00:00.000Z"),
    };
    const location = memoryLocation({ path: "/", record: true });
    const user = userEvent.setup();

    render(
      <Router hook={location.hook}>
        <SharingServicesProvider services={services}>
          <PublicationProvider
            routes={{
              home: "/",
              share: (id) => `/s/${id}`,
            }}
          >
            <SharePanel
              context={context}
              manageHref="/manage"
              report={syntheticAdviceReport()}
            />
          </PublicationProvider>
        </SharingServicesProvider>
      </Router>,
    );

    expect(create).not.toHaveBeenCalled();
    expect(
      screen.getByText(
        "解析しただけでは、解析結果をサーバーへ送信・公開しません",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/「共有URLを生成」を押したときだけ/),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "プライバシーポリシー" }),
    ).toHaveAttribute("href", "/privacy");

    await user.click(screen.getByRole("button", { name: "共有URLを生成" }));

    await waitFor(() => expect(create).toHaveBeenCalledTimes(1));
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
      }),
      "ABCD-EFGH-JKLM",
    );
    expect(
      screen.getByText(
        "公開URLを準備しました。この端末では動画付きの解析画面を表示しています。",
      ),
    ).toBeInTheDocument();
  });

  test("ruleset v9は理由を表示して共有操作とAPI呼出を無効にする", () => {
    const create = mock(async () => ({
      id: "Abcdefghijklmnopqrstu_",
      url: "https://fighter.example/s/Abcdefghijklmnopqrstu_",
      expiresAt: "2026-08-23T00:00:00.000Z",
    }));
    const services: SharingServices = {
      gateway: {
        create,
        delete: async () => undefined,
        errorMessage: () => "共有URLを作成できませんでした。",
      },
      managedShares: {
        save: () => true,
        load: () => ({ available: true, shares: [] }),
        remove: () => true,
        subscribe: () => () => undefined,
      },
      capabilities: {
        copyText: async () => undefined,
        canShare: () => false,
        share: async () => undefined,
        confirm: () => true,
        origin: () => "https://fighter.example",
        isCancelledShare: () => false,
      },
      generateDeleteCode: () => "ABCD-EFGH-JKLM",
      now: () => new Date("2026-07-24T00:00:00.000Z"),
    };
    const location = memoryLocation({ path: "/", record: true });

    render(
      <Router hook={location.hook}>
        <SharingServicesProvider services={services}>
          <PublicationProvider
            routes={{ home: "/", share: (id) => `/s/${id}` }}
          >
            <SharePanel
              context={context}
              manageHref="/manage"
              report={syntheticAdviceReport({ ruleset_version: 9 })}
            />
          </PublicationProvider>
        </SharingServicesProvider>
      </Router>,
    );

    expect(
      screen.getByRole("button", { name: "共有URLを生成" }),
    ).toBeDisabled();
    expect(screen.getByRole("note")).toHaveTextContent(
      "共有形式の更新が完了するまで公開できません",
    );
    expect(create).not.toHaveBeenCalled();
  });
});

import { describe, expect, test } from "bun:test";
import { render, screen } from "@testing-library/react";
import { syntheticAdviceReport } from "~/test-support/analysis.js";
import { SummaryOverview } from "./SummaryOverview.js";

describe("SummaryOverview", () => {
  test("解析結果が映像からの推定であることを常に表示する", () => {
    render(
      <SummaryOverview
        report={syntheticAdviceReport({ summary: "解析結果の要約" })}
      />,
    );

    expect(screen.getByText("解析結果の要約")).toBeInTheDocument();
    expect(
      screen.getByText(
        "解析結果は映像からの推定です。正確な記録ではなく、見直しのための参考情報として利用してください。",
      ),
    ).toBeInTheDocument();
  });
});

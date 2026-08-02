import { Fragment, type ReactNode, useState } from "react";
import { Link } from "wouter";
import { paths } from "~/app/paths.js";
import {
  thirdPartyComponents,
  thirdPartyLicenseDocuments,
} from "~/generated/third-party-licenses.js";

const DEFAULT_LAST_UPDATED = "2026-07-24";
const DEFAULT_LAST_UPDATED_LABEL = "2026年7月24日";
const thirdPartyLicenseDocumentById = new Map(
  thirdPartyLicenseDocuments.map((document) => [document.id, document]),
);
const licenseDocumentOriginLabels = {
  "canonical-fallback": "標準本文による補完",
  "package-file": "パッケージ同梱ファイル",
  "reviewed-override": "確認済み上流文書による補完",
} as const;

function LegalDocument({
  title,
  summary,
  version,
  lastUpdated = DEFAULT_LAST_UPDATED,
  lastUpdatedLabel = DEFAULT_LAST_UPDATED_LABEL,
  children,
}: {
  title: string;
  summary: string;
  version?: string;
  lastUpdated?: string;
  lastUpdatedLabel?: string;
  children: ReactNode;
}) {
  return (
    <main className="legal-page">
      <article className="legal-document">
        <p className="legal-eyebrow">Fighter Notes</p>
        <h1>{title}</h1>
        <p className="legal-summary">{summary}</p>
        <p className="legal-updated">
          最終更新: <time dateTime={lastUpdated}>{lastUpdatedLabel}</time>
          {version ? ` / 文書バージョン: ${version}` : ""}
        </p>
        {children}
        <p className="legal-back-link">
          <Link href={paths.home}>Fighter Notes に戻る</Link>
        </p>
      </article>
    </main>
  );
}

export function PrivacyPage() {
  return (
    <LegalDocument
      title="プライバシーポリシー"
      summary="Fighter Notes が動画や解析結果などの情報をどのように取り扱うかを説明します。"
      version="1.1"
      lastUpdated="2026-08-03"
      lastUpdatedLabel="2026年8月3日"
    >
      <p>
        Fighter
        Notes（以下「本サービス」といいます）は、利用者の情報を以下のとおり取り扱います。このポリシーは、開発者が提供する公式ホスト版に適用され、第三者が複製、改変またはセルフホストした版には適用されません。
      </p>

      <section>
        <h2>1. アカウントと登録情報</h2>
        <p>
          本サービスにはアカウント機能がなく、氏名、住所、電話番号、メールアドレス、プレイヤーID、SNS
          ID等の登録や入力を求めません。
        </p>
      </section>

      <section>
        <h2>2. 動画の取り扱い</h2>
        <p>
          利用者が選択した動画は、利用者の端末内で解析します。動画、音声、動画から取り出した画像および動画ファイル名は、サーバーへ送信または保存されません。
        </p>
        <p>これらを第三者へ提供せず、機械学習の学習データにも利用しません。</p>
      </section>

      <section>
        <h2>3. 解析結果の共有</h2>
        <p>
          解析だけを行う場合、解析結果はサーバーへ送信されず、公開されません。
        </p>
        <p>
          解析後に利用者が「共有URLを生成」を選んだ場合に限り、キャラクター、ラウンド結果、対戦傾向および改善点等の集計結果をサーバーへ送信して保存し、公開URLを作成します。
        </p>
        <p>動画、動画ファイル名、入力履歴等の情報は、公開結果に含めません。</p>
        <p>
          公開URLには認証機能がなく、URLを知る人は誰でも閲覧できます。公開結果は原則として作成から30日間保存し、期限後に削除します。期限前でも、発行された削除コードを使って
          <Link href={paths.manage}>共有管理画面</Link>
          から削除できます。
        </p>
        <p>
          検索サービス、SNSまたは閲覧者が保存した公開結果のコピーは、公開結果を削除した後や保存期限を過ぎた後も残る場合があります。
        </p>
      </section>

      <section>
        <h2>4. ブラウザに保存する情報</h2>
        <p>
          キャラクター、ラウンド結果、対戦傾向および改善点等の解析履歴は、利用者のブラウザ内に最大200件保存されます。動画本体や動画ファイル名は履歴に保存されません。
        </p>
        <p>
          解析履歴の保存は初期状態で有効です。結果画面から今後の保存を停止または再開でき、保存済みの解析履歴を1件ずつ、またはすべて削除できます。保存を停止しても、それ以前の履歴は自動では削除されません。
        </p>
        <p>
          共有管理に必要な公開URLや削除コードも、利用者のブラウザ内に保存されます。これらの情報は他の端末へ同期されず、端末またはブラウザを変更しても引き継がれません。
        </p>
        <p>
          結果画面から解析履歴を削除しても、公開URLや削除コードは削除されません。公開結果は共有管理画面から別途削除してください。
        </p>
        <p>
          解析履歴が200件を超えると、古いものから削除されます。ブラウザのサイトデータを削除すると、解析履歴、公開URLおよび削除コードも削除され、復元できません。ブラウザの仕様により、自動的に削除される場合もあります。
        </p>
      </section>

      <section>
        <h2>5. 接続情報と外部サービス</h2>
        <p>
          本サービスへのアクセス時には、サイトの配信やサーバー処理のため、IPアドレス、アクセス日時、ブラウザ情報、閲覧したページ等の接続情報が処理されます。これらの情報は、不正利用の防止や障害対応のために記録される場合があります。
        </p>
        <p>
          本サービスは、サイトの配信と保護にCloudflareを、サーバーの実行にGoogle
          Cloudを利用しています。本サービスでは、接続情報を広告、マーケティングまたは販売に利用しません。
        </p>
        <p>
          本サービスには、広告配信サービスや、利用者の行動追跡を目的とするアクセス解析ツールを導入していません。
        </p>
      </section>

      <section>
        <h2>6. 本ポリシーの変更</h2>
        <p>
          機能や情報の取り扱いを変更した場合は、本ポリシーを改定し、このページの文書バージョンと最終更新日を更新します。
        </p>
        <p>制定日: 2026年7月25日</p>
        <p>改定日: 2026年8月3日</p>
      </section>
    </LegalDocument>
  );
}

export function LicensesPage() {
  const [expandedLicensePanel, setExpandedLicensePanel] = useState<
    string | null
  >(null);

  return (
    <LegalDocument
      title="使用コンポーネントのライセンス"
      summary="本サービスで使用しているオープンソースソフトウェアと、そのライセンス情報を掲載しています。"
    >
      <section>
        <h2>このページについて</h2>
        <p>
          本サービスのアプリケーションには、画面表示、動画解析および解析結果の共有などの機能を提供するため、第三者が提供するオープンソースソフトウェアが組み込まれています。このページでは、それらのソフトウェアの名称、バージョン、ライセンスおよび著作権表示を掲載します。
        </p>
        <p>
          一覧のライセンス名を選ぶと、各ソフトウェアのライセンス文書を確認できます。配布物に関する通知の完全版は、
          <a href="/THIRD_PARTY_NOTICES.txt">THIRD_PARTY_NOTICES</a>
          から確認できます。
        </p>
        <p>
          ライセンス名に「OR」が含まれる場合は、いずれかのライセンスを選択できることを示します。「AND」が含まれる場合は、複数のライセンスがあわせて適用されることを示します。
        </p>
      </section>

      <section>
        <h2>コンポーネント一覧（{thirdPartyComponents.length}件）</h2>
        <div className="license-table-wrap">
          <table className="license-table">
            <colgroup>
              <col className="license-component-column" />
              <col className="license-version-column" />
              <col className="license-target-column" />
              <col className="license-name-column" />
              <col className="license-attribution-column" />
            </colgroup>
            <thead>
              <tr>
                <th scope="col">コンポーネント</th>
                <th scope="col">版</th>
                <th scope="col">対象</th>
                <th scope="col">ライセンス</th>
                <th scope="col">著作者・権利表示</th>
              </tr>
            </thead>
            <tbody>
              {thirdPartyComponents.map((component, componentIndex) => {
                const componentKey = `${component.ecosystem}:${component.name}:${component.version}`;
                return (
                  <Fragment key={componentKey}>
                    <tr>
                      <th scope="row">
                        <a href={component.source}>{component.name}</a>
                        <small>{component.ecosystem}</small>
                      </th>
                      <td>{component.version}</td>
                      <td>{component.targets.join(" / ")}</td>
                      <td>
                        <button
                          aria-controls={`license-panel-${componentIndex}`}
                          aria-expanded={
                            expandedLicensePanel ===
                            `license-panel-${componentIndex}`
                          }
                          className="license-document-button"
                          onClick={() =>
                            setExpandedLicensePanel((current) =>
                              current === `license-panel-${componentIndex}`
                                ? null
                                : `license-panel-${componentIndex}`,
                            )
                          }
                          type="button"
                        >
                          <span
                            aria-hidden="true"
                            className="license-document-marker"
                          />
                          <span className="license-document-label">
                            {component.license}
                          </span>
                        </button>
                      </td>
                      <td className="license-attribution-cell">
                        {component.copyrights.map((copyright) => (
                          <small key={copyright}>{copyright}</small>
                        ))}
                      </td>
                    </tr>
                    {expandedLicensePanel ===
                      `license-panel-${componentIndex}` && (
                      <tr
                        className="license-text-row"
                        id={`license-panel-${componentIndex}`}
                      >
                        <td colSpan={5}>
                          <div className="license-text-panel">
                            <div className="license-text-heading">
                              <div>
                                <h3>
                                  {component.name} {component.version}
                                </h3>
                                <p>宣言ライセンス: {component.license}</p>
                              </div>
                              <button
                                className="license-text-close"
                                onClick={() => setExpandedLicensePanel(null)}
                                type="button"
                              >
                                閉じる
                              </button>
                            </div>
                            <div className="license-text-documents">
                              {component.documents.map((reference) => {
                                const document =
                                  thirdPartyLicenseDocumentById.get(
                                    reference.id,
                                  );
                                if (!document) return null;
                                return (
                                  <section
                                    className="license-text-document"
                                    key={`${reference.id}:${reference.name}`}
                                  >
                                    <h4>
                                      {
                                        licenseDocumentOriginLabels[
                                          reference.origin
                                        ]
                                      }
                                      : {reference.name}
                                    </h4>
                                    {document.text.includes(
                                      "Copyright [yyyy] [name of copyright owner]",
                                    ) && (
                                      <p className="license-template-note">
                                        本文中の
                                        <code>
                                          Copyright [yyyy] [name of copyright
                                          owner]
                                        </code>
                                        は、Apache-2.0本文末尾に含まれる適用例です。著作権表示の取得漏れを示すものではありません。
                                      </p>
                                    )}
                                    <pre>{document.text}</pre>
                                  </section>
                                );
                              })}
                            </div>
                          </div>
                        </td>
                      </tr>
                    )}
                  </Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
    </LegalDocument>
  );
}

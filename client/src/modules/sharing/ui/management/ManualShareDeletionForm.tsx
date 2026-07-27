import type { FormEvent } from "react";
import type { Feedback } from "../../domain/share-management.js";

interface ManualShareDeletionFormProps {
  reference: string;
  deleteCredential: string;
  busy: boolean;
  feedback: Feedback;
  onReferenceChange(value: string): void;
  onCredentialChange(value: string): void;
  onSubmit(): void;
}

export function ManualShareDeletionForm(props: ManualShareDeletionFormProps) {
  const submit = (event: FormEvent) => {
    event.preventDefault();
    props.onSubmit();
  };
  return (
    <section
      className="management-section"
      aria-labelledby="manual-share-heading"
    >
      <h2 id="manual-share-heading">別の端末で作成した共有を削除</h2>
      <p>
        公開URLと発行時に表示された削除コードを入力してください。以前自分で設定した削除用パスワードも利用できます。
      </p>
      <form id="share-management-form" onSubmit={submit}>
        <div className="field">
          <label htmlFor="share-management-reference">
            共有URLまたは共有ID
          </label>
          <input
            type="text"
            id="share-management-reference"
            autoComplete="url"
            required
            value={props.reference}
            onChange={(event) =>
              props.onReferenceChange(event.currentTarget.value)
            }
          />
        </div>
        <div className="field">
          <label htmlFor="share-management-password">削除コード</label>
          <input
            type="password"
            id="share-management-password"
            minLength={12}
            maxLength={128}
            autoComplete="off"
            autoCapitalize="characters"
            spellCheck={false}
            placeholder="ABCD-EFGH-JKLM"
            required
            value={props.deleteCredential}
            onChange={(event) =>
              props.onCredentialChange(event.currentTarget.value)
            }
          />
        </div>
        <button
          type="submit"
          className="analyze-btn share-command-danger"
          disabled={props.busy}
        >
          {props.busy ? "削除中…" : "共有結果を削除"}
        </button>
      </form>
      <span
        id="share-management-status"
        className="share-status"
        data-tone={props.feedback.tone}
        role="status"
        aria-live="polite"
      >
        {props.feedback.message}
      </span>
    </section>
  );
}

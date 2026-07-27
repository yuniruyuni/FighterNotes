import { LockKeyhole } from "lucide-react";
import { type DragEvent, useRef, useState } from "react";
import { useObjectUrl } from "../../../../shared/browser/use-object-url.js";

interface VideoFilePickerProps {
  file: File | null;
  disabled: boolean;
  onChange(file: File | null): void;
}

export function VideoFilePicker({
  file,
  disabled,
  onChange,
}: VideoFilePickerProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [dragging, setDragging] = useState(false);
  const previewUrl = useObjectUrl(file);
  const acceptDrop = (event: DragEvent<HTMLButtonElement>) => {
    event.preventDefault();
    setDragging(false);
    const dropped = event.dataTransfer.files[0];
    if (dropped) onChange(dropped);
  };

  return (
    <div className="card card--feature">
      <h2>動画ファイルを選択</h2>
      <button
        type="button"
        id="drop-zone"
        className={dragging ? "drag-over" : undefined}
        disabled={disabled}
        onClick={() => inputRef.current?.click()}
        onDragOver={(event) => {
          event.preventDefault();
          setDragging(true);
        }}
        onDragLeave={() => setDragging(false)}
        onDrop={acceptDrop}
      >
        <p>ここに動画をドロップ、または</p>
        <span className="file-label">ファイルを選択</span>
        <p className="muted-note selected-file-name">{file?.name}</p>
      </button>
      <input
        ref={inputRef}
        className="video-file-input"
        type="file"
        id="file-input"
        accept="video/*"
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.files?.[0] ?? null)}
      />
      <p className="privacy-note">
        <LockKeyhole size={16} aria-hidden="true" />
        <span>
          動画がサーバーへアップロードされることはありません。
          解析はすべてこのブラウザの中だけで行われ、動画データは外部に送信されません。
          録画・利用してよい動画だけを選択してください。
        </span>
      </p>
      {previewUrl && (
        <video
          id="video-preview"
          src={previewUrl}
          controls
          muted
          aria-label="選択した動画のプレビュー"
        />
      )}
    </div>
  );
}

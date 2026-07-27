import { Link } from "wouter";

export function AppFooter({ compact = false }: { compact?: boolean }) {
  return (
    <footer className={`app-footer${compact ? " app-footer--compact" : ""}`}>
      <p className="app-footer-author">
        Created by Yuniruyuni —{" "}
        <a
          href="https://yuniruyuni.net"
          target="_blank"
          rel="noopener noreferrer"
        >
          yuniruyuni.net
        </a>
      </p>
      <nav className="app-footer-links" aria-label="サイト情報">
        <Link href="/privacy">プライバシーポリシー</Link>
        <Link href="/licenses">使用コンポーネントのライセンス</Link>
      </nav>
    </footer>
  );
}

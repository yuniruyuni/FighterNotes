import { Link } from "wouter";
import { paths } from "~/app/paths.js";
import { PageLayout } from "~/shared/ui/PageLayout.js";

export function NotFoundPage() {
  return (
    <PageLayout>
      <main className="route-status">
        <h1>ページが見つかりません</h1>
        <Link href={paths.home}>Fighter Notes に戻る</Link>
      </main>
    </PageLayout>
  );
}

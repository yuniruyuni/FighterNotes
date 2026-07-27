# License notice overrides

このdirectoryは、`license-checker-rseidelsohn`が配布されたnpm package内から専用のlicense・NOTICE本文を
取得できない場合に限り、上流のtagged sourceまたは同一projectの正式なlicense表示から確認した
本文を保持します。README全体がlicense候補として返されても、license全文の代わりには使用しません。

- component固有overrideは`package名@version`へ固定する。license、source、copyright等の
  metadataを補う場合も同じversion固定mapで管理する。
- license共通fallbackは、宣言されたSPDX expressionと本文が一意に対応する場合だけ使用する。
- 追加・更新時は`generate-third-party-notices.ts`のmapへ明示的に登録し、取得元と対象versionを
  commit messageまたはreview記録で確認する。
- generatorは、未登録、未使用、欠落したoverrideを失敗として扱う。

通常は`license-checker-rseidelsohn`が取得したpackage同梱のlicense、COPYING、COPYRIGHT、NOTICEを優先し、
このdirectoryの内容で上書きしません。

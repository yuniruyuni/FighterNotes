# Third-Party Notices

This file is generated. Do not edit it by hand.

It covers production application-package dependency closures resolved
from bun.lock and Cargo.lock for the browser bundle, WebAssembly analyzer,
and server application. Development/test packages, the Bun runtime embedded
in the compiled server, compiler toolchains, and base-container operating-
system packages are outside this application-package inventory and remain
subject to their upstream license notices; they are identified below.
Identical license documents are stored once and referenced by hash.
Declared SPDX expressions are preserved without selecting an OR alternative.
Every packaged license document is retained for inspection; AND denotes terms
that apply together, while OR preserves the alternatives offered upstream.

- bun.lock SHA-256: `3a297dce7d12888f29c8af8585b88e31025f8f926061f7d799749db4fd1510f5`
- Cargo.lock SHA-256: `d24dfe87b4e3112095bba881eb5a6da9da8323297637f23e7dcb47b124eea50c`
- npm scanner: `license-checker-rseidelsohn 5.0.1`
- Cargo scanner: `cargo-about 0.9.1`

## Distributed material inventory

### System font stacks

- Category: Font
- Location: CSS font-family declarations
- Treatment: No font binary or external Web font is distributed; glyphs come from the user’s OS/browser.

### Lucide icons (lucide-react 0.575.0)

- Category: Icon
- Location: Browser JavaScript bundle
- Treatment: ISC; tracked in the component index and complete notices.
- Reference: https://github.com/lucide-icons/lucide

### Fighter Notes OGP image

- Category: Image
- Location: client/src/shared/assets/images/fighter-notes-ogp.jpg → /images/fighter-notes-ogp.jpg
- Treatment: Project-specific media asset; not part of the third-party software inventory. Its reuse terms are documented in the source repository.
- Reference: https://github.com/yuniruyuni/FighterNotes#ライセンス

### Analyzer data and recognition models

- Category: Data/model
- Location: crates/video-analyzer/data, input_history/templates.rs, round_start/fight_template.bin, and meter_digits.bin
- Treatment: Project data/model; not part of the third-party software inventory. DATA_NOTICE applies.
- Reference: /DATA_NOTICE.txt

### Browser bundles and WebAssembly analyzer

- Category: Generated output
- Location: index.js, analyzer-worker.js, wasm_bridge_bg.wasm, HTML, and CSS
- Treatment: Bundled npm/Cargo portions retain the licenses listed below; the build does not replace those terms.
- Reference: /THIRD_PARTY_NOTICES.txt

### Bun 1.3.14 runtime

- Category: Runtime/platform
- Location: Compiled server executable
- Treatment: Embedded by bun build --compile. It is outside the bun.lock application-package inventory and remains subject to Bun’s runtime and linked-library notices.
- Reference: https://bun.sh/docs/project/license

### Distroless/Debian runtime image

- Category: Runtime/platform
- Location: gcr.io/distroless/cc-debian12:nonroot@sha256:ce0d66bc0f64aae46e6a03add867b07f42cc7b8799c949c2e898057b7f75a151
- Treatment: Base-container operating-system packages are outside the application-package inventory and remain subject to their upstream notices.
- Reference: https://github.com/GoogleContainerTools/distroless

## Component index

- bumpalo 3.20.3 (Cargo; browser/WASM) — MIT OR Apache-2.0
- cfg-if 1.0.4 (Cargo; browser/WASM) — MIT OR Apache-2.0
- itoa 1.0.18 (Cargo; browser/WASM) — MIT OR Apache-2.0
- memchr 2.8.2 (Cargo; browser/WASM) — Unlicense OR MIT
- once_cell 1.21.4 (Cargo; browser/WASM) — MIT OR Apache-2.0
- proc-macro2 1.0.106 (Cargo; browser/WASM) — MIT OR Apache-2.0
- quote 1.0.45 (Cargo; browser/WASM) — MIT OR Apache-2.0
- serde 1.0.228 (Cargo; browser/WASM) — MIT OR Apache-2.0
- serde_core 1.0.228 (Cargo; browser/WASM) — MIT OR Apache-2.0
- serde_derive 1.0.228 (Cargo; browser/WASM) — MIT OR Apache-2.0
- serde_json 1.0.150 (Cargo; browser/WASM) — MIT OR Apache-2.0
- syn 2.0.117 (Cargo; browser/WASM) — MIT OR Apache-2.0
- unicode-ident 1.0.24 (Cargo; browser/WASM) — (MIT OR Apache-2.0) AND Unicode-3.0
- wasm-bindgen 0.2.125 (Cargo; browser/WASM) — MIT OR Apache-2.0
- wasm-bindgen-macro 0.2.125 (Cargo; browser/WASM) — MIT OR Apache-2.0
- wasm-bindgen-macro-support 0.2.125 (Cargo; browser/WASM) — MIT OR Apache-2.0
- wasm-bindgen-shared 0.2.125 (Cargo; browser/WASM) — MIT OR Apache-2.0
- zmij 1.0.21 (Cargo; browser/WASM) — MIT
- @hono/trpc-server 0.4.2 (npm; server) — MIT
- @trpc/client 11.18.0 (npm; browser) — MIT
- @trpc/server 11.18.0 (npm; server) — MIT
- hono 4.13.0 (npm; server) — MIT
- lucide-react 0.575.0 (npm; browser) — ISC
- mitt 3.0.1 (npm; browser) — MIT
- mp4box 2.4.1 (npm; browser) — BSD-3-Clause
- pg 8.22.0 (npm; server) — MIT
- pg-cloudflare 1.4.0 (npm; server) — MIT
- pg-connection-string 2.14.0 (npm; server) — MIT
- pg-int8 1.0.1 (npm; server) — ISC
- pg-pool 3.14.0 (npm; server) — MIT
- pg-protocol 1.15.0 (npm; server) — MIT
- pg-types 2.2.0 (npm; server) — MIT
- pgpass 1.0.5 (npm; server) — MIT
- postgres-array 2.0.0 (npm; server) — MIT
- postgres-bytea 1.0.1 (npm; server) — MIT
- postgres-date 1.0.7 (npm; server) — MIT
- postgres-interval 1.2.0 (npm; server) — MIT
- react 19.2.7 (npm; browser) — MIT
- react-dom 19.2.7 (npm; browser) — MIT
- regexparam 3.0.0 (npm; browser) — MIT
- scheduler 0.27.0 (npm; browser) — MIT
- split2 4.2.0 (npm; server) — ISC
- use-sync-external-store 1.6.0 (npm; browser) — MIT
- wouter 3.10.0 (npm; browser) — Unlicense
- xtend 4.0.2 (npm; server) — MIT
- zod 4.4.3 (npm; server) — MIT

---

## bumpalo 3.20.3

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/fitzgen/bumpalo
- Copyright / attribution:
  - Copyright (c) 2019 Nick Fitzgerald

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 65f94e99ddaf](#license-text-65f94e99ddaf4f5d1782a6dae23f35d4293a9a01444a13135a6887017d353cee)

---

## cfg-if 1.0.4

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/rust-lang/cfg-if
- Copyright / attribution:
  - Copyright (c) 2014 Alex Crichton

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 378f5840b258](#license-text-378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397)

---

## itoa 1.0.18

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/dtolnay/itoa
- Copyright / attribution:
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## memchr 2.8.2

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: Unlicense OR MIT
- Source: https://github.com/BurntSushi/memchr
- Copyright / attribution:
  - Copyright (c) 2015 Andrew Gallant

- COPYING (package file): [license text 01c266bced4a](#license-text-01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f)
- LICENSE-MIT (package file): [license text 0f96a83840e1](#license-text-0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f)

---

## once_cell 1.21.4

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/matklad/once_cell
- Copyright / attribution:
  - Package author: Aleksey Kladov <aleksey.kladov@gmail.com>

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## proc-macro2 1.0.106

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/dtolnay/proc-macro2
- Copyright / attribution:
  - Package author: David Tolnay <dtolnay@gmail.com>
  - Package author: Alex Crichton <alex@alexcrichton.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## quote 1.0.45

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/dtolnay/quote
- Copyright / attribution:
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## serde 1.0.228

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/serde-rs/serde
- Copyright / attribution:
  - Package author: Erick Tryzelaar <erick.tryzelaar@gmail.com>
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## serde_core 1.0.228

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/serde-rs/serde
- Copyright / attribution:
  - Package author: Erick Tryzelaar <erick.tryzelaar@gmail.com>
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## serde_derive 1.0.228

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/serde-rs/serde
- Copyright / attribution:
  - Package author: Erick Tryzelaar <erick.tryzelaar@gmail.com>
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## serde_json 1.0.150

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/serde-rs/json
- Copyright / attribution:
  - Package author: Erick Tryzelaar <erick.tryzelaar@gmail.com>
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## syn 2.0.117

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/dtolnay/syn
- Copyright / attribution:
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## unicode-ident 1.0.24

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: (MIT OR Apache-2.0) AND Unicode-3.0
- Source: https://github.com/dtolnay/unicode-ident
- Copyright / attribution:
  - Copyright © 1991-2023 Unicode, Inc.

- LICENSE-APACHE (package file): [license text b30df9a48463](#license-text-b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e)
- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)
- LICENSE-UNICODE (package file): [license text f7db81051789](#license-text-f7db81051789b729fea528a63ec4c938fdcb93d9d61d97dc8cc2e9df6d47f2a1)

---

## wasm-bindgen 0.2.125

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/wasm-bindgen/wasm-bindgen
- Copyright / attribution:
  - Copyright (c) 2014 Alex Crichton

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 378f5840b258](#license-text-378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397)

---

## wasm-bindgen-macro 0.2.125

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro
- Copyright / attribution:
  - Copyright (c) 2014 Alex Crichton

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 378f5840b258](#license-text-378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397)

---

## wasm-bindgen-macro-support 0.2.125

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support
- Copyright / attribution:
  - Copyright (c) 2014 Alex Crichton

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 378f5840b258](#license-text-378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397)

---

## wasm-bindgen-shared 0.2.125

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT OR Apache-2.0
- Source: https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared
- Copyright / attribution:
  - Copyright (c) 2014 Alex Crichton

- LICENSE-APACHE (package file): [license text 143368af9701](#license-text-143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4)
- LICENSE-MIT (package file): [license text 378f5840b258](#license-text-378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397)

---

## zmij 1.0.21

- Ecosystem: Cargo
- Used by: browser/WASM
- Declared license: MIT
- Source: https://github.com/dtolnay/zmij
- Copyright / attribution:
  - Package author: David Tolnay <dtolnay@gmail.com>

- LICENSE-MIT (package file): [license text 23f18e03dc49](#license-text-23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3)

---

## @hono/trpc-server 0.4.2

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/honojs/middleware
- Copyright / attribution:
  - Copyright (c) 2021 - present, Yusuke Wada and Hono contributors

- Reviewed license notice (reviewed upstream override): [license text a6ab98e5c77b](#license-text-a6ab98e5c77b9070c443eaff2ff81034a6f8cc05a7524d5098eb0f24defa0115)

---

## @trpc/client 11.18.0

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/trpc/trpc
- Copyright / attribution:
  - Copyright (c) 2023 Alex Johansson

- LICENSE (package file): [license text e714dd84c8fa](#license-text-e714dd84c8fa242600844b05d317a31003423723178c1f1603dbfad1bc68d906)

---

## @trpc/server 11.18.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/trpc/trpc
- Copyright / attribution:
  - Copyright (c) 2023 Alex Johansson

- LICENSE (package file): [license text e714dd84c8fa](#license-text-e714dd84c8fa242600844b05d317a31003423723178c1f1603dbfad1bc68d906)

---

## hono 4.13.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/honojs/hono
- Copyright / attribution:
  - Copyright (c) 2021 - present, Yusuke Wada and Hono contributors

- LICENSE (package file): [license text a6ab98e5c77b](#license-text-a6ab98e5c77b9070c443eaff2ff81034a6f8cc05a7524d5098eb0f24defa0115)

---

## lucide-react 0.575.0

- Ecosystem: npm
- Used by: browser
- Declared license: ISC
- Source: https://github.com/lucide-icons/lucide
- Copyright / attribution:
  - Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2026 as part of Feather (MIT). All other copyright (c) for Lucide are held by Lucide Contributors 2026.
  - Copyright (c) 2013-2026 Cole Bemis

- LICENSE (package file): [license text 668dcc528034](#license-text-668dcc52803480e0a026b31140a4cae668772663cd764e5991d252eef03f98db)

---

## mitt 3.0.1

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/developit/mitt
- Copyright / attribution:
  - Copyright (c) 2021 Jason Miller

- LICENSE (package file): [license text 1cab22f19626](#license-text-1cab22f196264195a4caec8ca5630170fdde76ee8f43346e47021d087332d3b0)

---

## mp4box 2.4.1

- Ecosystem: npm
- Used by: browser
- Declared license: BSD-3-Clause
- Source: https://github.com/gpac/mp4box.js
- Copyright / attribution:
  - Copyright (c) 2012. Telecom ParisTech/TSI/MM/GPAC Cyril Concolato

- LICENSE (package file): [license text ebad0332150a](#license-text-ebad0332150a08f37389158289d93ab2f70b0ee8717d1db9b3d002febc6c5047)

---

## pg 8.22.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-postgres
- Copyright / attribution:
  - Copyright (c) 2010 - 2021 Brian Carlson

- LICENSE (package file): [license text 192b8f5c9690](#license-text-192b8f5c96900f04a1271dec39688655d7416c1c6ea84a508e18b50d2b6751f3)

---

## pg-cloudflare 1.4.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-postgres
- Copyright / attribution:
  - Copyright (c) 2010 - 2021 Brian Carlson

- LICENSE (package file): [license text 192b8f5c9690](#license-text-192b8f5c96900f04a1271dec39688655d7416c1c6ea84a508e18b50d2b6751f3)

---

## pg-connection-string 2.14.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-postgres
- Copyright / attribution:
  - Copyright (c) 2014 Iced Development

- LICENSE (package file): [license text 85747ad4bba3](#license-text-85747ad4bba34e96e5055af5994796ec2a8525b4cecb14bc1bb257199dc29566)

---

## pg-int8 1.0.1

- Ecosystem: npm
- Used by: server
- Declared license: ISC
- Source: https://github.com/charmander/pg-int8
- Copyright / attribution:
  - Copyright © 2017, Charmander <~@charmander.me>

- LICENSE (package file): [license text 4e8e87ccdfc7](#license-text-4e8e87ccdfc7e4b47fd89015f78468aa53b6bf43ab6e6e12d43e8f55294911de)

---

## pg-pool 3.14.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-postgres
- Copyright / attribution:
  - Copyright (c) 2017 Brian M. Carlson

- LICENSE (package file): [license text 4f15ee7fc2a7](#license-text-4f15ee7fc2a72082859d7e0d12dfa4bcdd70b1c744ad3850d07780730ac08557)

---

## pg-protocol 1.15.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-postgres
- Copyright / attribution:
  - Copyright (c) 2010 - 2021 Brian Carlson

- LICENSE (package file): [license text 192b8f5c9690](#license-text-192b8f5c96900f04a1271dec39688655d7416c1c6ea84a508e18b50d2b6751f3)

---

## pg-types 2.2.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/brianc/node-pg-types
- Copyright / attribution:
  - Copyright (c) 2014 Brian M. Carlson

- Reviewed license notice (reviewed upstream override): [license text e9175c300e0b](#license-text-e9175c300e0b6dfe281de13e9071166106b67b6e05e9d7156af2e032ffb3d31b)

---

## pgpass 1.0.5

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/hoegaarden/pgpass
- Copyright / attribution:
  - Copyright (c) 2013-2016 Hannes Hörl

- Reviewed license notice (reviewed upstream override): [license text 187ea0c4a3b3](#license-text-187ea0c4a3b35d429f8e08b66a80387c3be270e4fadb9fc9634928cb569a29c2)

---

## postgres-array 2.0.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/bendrucker/postgres-array
- Copyright / attribution:
  - Copyright (c) Ben Drucker <bvdrucker@gmail.com> (bendrucker.me)

- license (package file): [license text f057f36739d5](#license-text-f057f36739d53d228a746de4440c1e0c644ecde06d6beab45337d39c9d12a393)

---

## postgres-bytea 1.0.1

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/bendrucker/postgres-bytea
- Copyright / attribution:
  - Copyright (c) Ben Drucker <bvdrucker@gmail.com> (bendrucker.me)

- license (package file): [license text f057f36739d5](#license-text-f057f36739d53d228a746de4440c1e0c644ecde06d6beab45337d39c9d12a393)

---

## postgres-date 1.0.7

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/bendrucker/postgres-date
- Copyright / attribution:
  - Copyright (c) Ben Drucker <bvdrucker@gmail.com> (bendrucker.me)

- license (package file): [license text f057f36739d5](#license-text-f057f36739d53d228a746de4440c1e0c644ecde06d6beab45337d39c9d12a393)

---

## postgres-interval 1.2.0

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/bendrucker/postgres-interval
- Copyright / attribution:
  - Copyright (c) Ben Drucker <bvdrucker@gmail.com> (bendrucker.me)

- license (package file): [license text f057f36739d5](#license-text-f057f36739d53d228a746de4440c1e0c644ecde06d6beab45337d39c9d12a393)

---

## react 19.2.7

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/facebook/react
- Copyright / attribution:
  - Copyright (c) Meta Platforms, Inc. and affiliates.

- LICENSE (package file): [license text da6d3703ed11](#license-text-da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93)

---

## react-dom 19.2.7

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/facebook/react
- Copyright / attribution:
  - Copyright (c) Meta Platforms, Inc. and affiliates.

- LICENSE (package file): [license text da6d3703ed11](#license-text-da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93)

---

## regexparam 3.0.0

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/lukeed/regexparam
- Copyright / attribution:
  - Copyright (c) Luke Edwards <luke.edwards05@gmail.com> (lukeed.com)

- license (package file): [license text 306fa513e39b](#license-text-306fa513e39b23a6e8747520de761809d206b99800ef41907b530226574c59ae)

---

## scheduler 0.27.0

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/facebook/react
- Copyright / attribution:
  - Copyright (c) Meta Platforms, Inc. and affiliates.

- LICENSE (package file): [license text da6d3703ed11](#license-text-da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93)

---

## split2 4.2.0

- Ecosystem: npm
- Used by: server
- Declared license: ISC
- Source: https://github.com/mcollina/split2
- Copyright / attribution:
  - Copyright (c) 2014-2018, Matteo Collina <hello@matteocollina.com>

- LICENSE (package file): [license text c372ef2fa1df](#license-text-c372ef2fa1dfcb124ed807609751e75e3a009f108c5724916b489288fcb88a0c)

---

## use-sync-external-store 1.6.0

- Ecosystem: npm
- Used by: browser
- Declared license: MIT
- Source: https://github.com/facebook/react
- Copyright / attribution:
  - Copyright (c) Meta Platforms, Inc. and affiliates.

- LICENSE (package file): [license text da6d3703ed11](#license-text-da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93)

---

## wouter 3.10.0

- Ecosystem: npm
- Used by: browser
- Declared license: Unlicense
- Source: https://github.com/molefrog/wouter
- Copyright / attribution:
  - Not separately stated in the distributed files; see the complete license and notice text.

- Unlicense license text (canonical fallback): [license text 8666aaf379a6](#license-text-8666aaf379a6509e2714d56f1276b186760db9a695054737761ace47d10fa0a4)

---

## xtend 4.0.2

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/Raynos/xtend
- Copyright / attribution:
  - Copyright (c) 2012-2014 Raynos.

- LICENSE (package file): [license text 82e67379203d](#license-text-82e67379203d5794e7c44549847d8d64ae6904591381682360470898bd306821)

---

## zod 4.4.3

- Ecosystem: npm
- Used by: server
- Declared license: MIT
- Source: https://github.com/colinhacks/zod
- Copyright / attribution:
  - Copyright (c) 2025 Colin McDonnell

- LICENSE (package file): [license text 3f1189b28e38](#license-text-3f1189b28e3866e0d979968d466b78f813f76827cfdca1fbb124cc0a5c8841f8)

# License texts

---

## License text 01c266bced4a434da0051174d6bee16a4c82cf634e2679b6155d40d75012390f

Document names: COPYING

Referenced by:

- memchr 2.8.2 — COPYING

    This project is dual-licensed under the Unlicense and MIT licenses.

    You may use this code under the terms of either license.

---

## License text 0f96a83840e146e43c0ec96a22ec1f392e0680e6c1226e6f3ba87e0740af850f

Document names: LICENSE-MIT

Referenced by:

- memchr 2.8.2 — LICENSE-MIT

    The MIT License (MIT)

    Copyright (c) 2015 Andrew Gallant

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.

---

## License text 143368af9701a24ebaf89fe5310ad8116ca71e8e99f17415c0dd893f6a256ae4

Document names: LICENSE-APACHE

Referenced by:

- bumpalo 3.20.3 — LICENSE-APACHE
- cfg-if 1.0.4 — LICENSE-APACHE
- once_cell 1.21.4 — LICENSE-APACHE
- wasm-bindgen 0.2.125 — LICENSE-APACHE
- wasm-bindgen-macro 0.2.125 — LICENSE-APACHE
- wasm-bindgen-macro-support 0.2.125 — LICENSE-APACHE
- wasm-bindgen-shared 0.2.125 — LICENSE-APACHE

    Apache License
                            Version 2.0, January 2004
                         http://www.apache.org/licenses/

    TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

    1. Definitions.

       "License" shall mean the terms and conditions for use, reproduction,
       and distribution as defined by Sections 1 through 9 of this document.

       "Licensor" shall mean the copyright owner or entity authorized by
       the copyright owner that is granting the License.

       "Legal Entity" shall mean the union of the acting entity and all
       other entities that control, are controlled by, or are under common
       control with that entity. For the purposes of this definition,
       "control" means (i) the power, direct or indirect, to cause the
       direction or management of such entity, whether by contract or
       otherwise, or (ii) ownership of fifty percent (50%) or more of the
       outstanding shares, or (iii) beneficial ownership of such entity.

       "You" (or "Your") shall mean an individual or Legal Entity
       exercising permissions granted by this License.

       "Source" form shall mean the preferred form for making modifications,
       including but not limited to software source code, documentation
       source, and configuration files.

       "Object" form shall mean any form resulting from mechanical
       transformation or translation of a Source form, including but
       not limited to compiled object code, generated documentation,
       and conversions to other media types.

       "Work" shall mean the work of authorship, whether in Source or
       Object form, made available under the License, as indicated by a
       copyright notice that is included in or attached to the work
       (an example is provided in the Appendix below).

       "Derivative Works" shall mean any work, whether in Source or Object
       form, that is based on (or derived from) the Work and for which the
       editorial revisions, annotations, elaborations, or other modifications
       represent, as a whole, an original work of authorship. For the purposes
       of this License, Derivative Works shall not include works that remain
       separable from, or merely link (or bind by name) to the interfaces of,
       the Work and Derivative Works thereof.

       "Contribution" shall mean any work of authorship, including
       the original version of the Work and any modifications or additions
       to that Work or Derivative Works thereof, that is intentionally
       submitted to Licensor for inclusion in the Work by the copyright owner
       or by an individual or Legal Entity authorized to submit on behalf of
       the copyright owner. For the purposes of this definition, "submitted"
       means any form of electronic, verbal, or written communication sent
       to the Licensor or its representatives, including but not limited to
       communication on electronic mailing lists, source code control systems,
       and issue tracking systems that are managed by, or on behalf of, the
       Licensor for the purpose of discussing and improving the Work, but
       excluding communication that is conspicuously marked or otherwise
       designated in writing by the copyright owner as "Not a Contribution."

       "Contributor" shall mean Licensor and any individual or Legal Entity
       on behalf of whom a Contribution has been received by Licensor and
       subsequently incorporated within the Work.

    2. Grant of Copyright License. Subject to the terms and conditions of
       this License, each Contributor hereby grants to You a perpetual,
       worldwide, non-exclusive, no-charge, royalty-free, irrevocable
       copyright license to reproduce, prepare Derivative Works of,
       publicly display, publicly perform, sublicense, and distribute the
       Work and such Derivative Works in Source or Object form.

    3. Grant of Patent License. Subject to the terms and conditions of
       this License, each Contributor hereby grants to You a perpetual,
       worldwide, non-exclusive, no-charge, royalty-free, irrevocable
       (except as stated in this section) patent license to make, have made,
       use, offer to sell, sell, import, and otherwise transfer the Work,
       where such license applies only to those patent claims licensable
       by such Contributor that are necessarily infringed by their
       Contribution(s) alone or by combination of their Contribution(s)
       with the Work to which such Contribution(s) was submitted. If You
       institute patent litigation against any entity (including a
       cross-claim or counterclaim in a lawsuit) alleging that the Work
       or a Contribution incorporated within the Work constitutes direct
       or contributory patent infringement, then any patent licenses
       granted to You under this License for that Work shall terminate
       as of the date such litigation is filed.

    4. Redistribution. You may reproduce and distribute copies of the
       Work or Derivative Works thereof in any medium, with or without
       modifications, and in Source or Object form, provided that You
       meet the following conditions:

       (a) You must give any other recipients of the Work or
           Derivative Works a copy of this License; and

       (b) You must cause any modified files to carry prominent notices
           stating that You changed the files; and

       (c) You must retain, in the Source form of any Derivative Works
           that You distribute, all copyright, patent, trademark, and
           attribution notices from the Source form of the Work,
           excluding those notices that do not pertain to any part of
           the Derivative Works; and

       (d) If the Work includes a "NOTICE" text file as part of its
           distribution, then any Derivative Works that You distribute must
           include a readable copy of the attribution notices contained
           within such NOTICE file, excluding those notices that do not
           pertain to any part of the Derivative Works, in at least one
           of the following places: within a NOTICE text file distributed
           as part of the Derivative Works; within the Source form or
           documentation, if provided along with the Derivative Works; or,
           within a display generated by the Derivative Works, if and
           wherever such third-party notices normally appear. The contents
           of the NOTICE file are for informational purposes only and
           do not modify the License. You may add Your own attribution
           notices within Derivative Works that You distribute, alongside
           or as an addendum to the NOTICE text from the Work, provided
           that such additional attribution notices cannot be construed
           as modifying the License.

       You may add Your own copyright statement to Your modifications and
       may provide additional or different license terms and conditions
       for use, reproduction, or distribution of Your modifications, or
       for any such Derivative Works as a whole, provided Your use,
       reproduction, and distribution of the Work otherwise complies with
       the conditions stated in this License.

    5. Submission of Contributions. Unless You explicitly state otherwise,
       any Contribution intentionally submitted for inclusion in the Work
       by You to the Licensor shall be under the terms and conditions of
       this License, without any additional terms or conditions.
       Notwithstanding the above, nothing herein shall supersede or modify
       the terms of any separate license agreement you may have executed
       with Licensor regarding such Contributions.

    6. Trademarks. This License does not grant permission to use the trade
       names, trademarks, service marks, or product names of the Licensor,
       except as required for reasonable and customary use in describing the
       origin of the Work and reproducing the content of the NOTICE file.

    7. Disclaimer of Warranty. Unless required by applicable law or
       agreed to in writing, Licensor provides the Work (and each
       Contributor provides its Contributions) on an "AS IS" BASIS,
       WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
       implied, including, without limitation, any warranties or conditions
       of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
       PARTICULAR PURPOSE. You are solely responsible for determining the
       appropriateness of using or redistributing the Work and assume any
       risks associated with Your exercise of permissions under this License.

    8. Limitation of Liability. In no event and under no legal theory,
       whether in tort (including negligence), contract, or otherwise,
       unless required by applicable law (such as deliberate and grossly
       negligent acts) or agreed to in writing, shall any Contributor be
       liable to You for damages, including any direct, indirect, special,
       incidental, or consequential damages of any character arising as a
       result of this License or out of the use or inability to use the
       Work (including but not limited to damages for loss of goodwill,
       work stoppage, computer failure or malfunction, or any and all
       other commercial damages or losses), even if such Contributor
       has been advised of the possibility of such damages.

    9. Accepting Warranty or Additional Liability. While redistributing
       the Work or Derivative Works thereof, You may choose to offer,
       and charge a fee for, acceptance of support, warranty, indemnity,
       or other liability obligations and/or rights consistent with this
       License. However, in accepting such obligations, You may act only
       on Your own behalf and on Your sole responsibility, not on behalf
       of any other Contributor, and only if You agree to indemnify,
       defend, and hold each Contributor harmless for any liability
       incurred by, or claims asserted against, such Contributor by reason
       of your accepting any such warranty or additional liability.

    END OF TERMS AND CONDITIONS

    APPENDIX: How to apply the Apache License to your work.

       To apply the Apache License to your work, attach the following
       boilerplate notice, with the fields enclosed by brackets "[]"
       replaced with your own identifying information. (Don't include
       the brackets!)  The text should be enclosed in the appropriate
       comment syntax for the file format. We also recommend that a
       file or class name and description of purpose be included on the
       same "printed page" as the copyright notice for easier
       identification within third-party archives.

    Copyright [yyyy] [name of copyright owner]

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

    	http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.

---

## License text 187ea0c4a3b35d429f8e08b66a80387c3be270e4fadb9fc9634928cb569a29c2

Document names: Reviewed license notice

Referenced by:

- pgpass 1.0.5 — Reviewed license notice

    Copyright (c) 2013-2016 Hannes Hörl

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software without restriction, including without limitation the rights to
    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software is furnished to do so,
    subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
    COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
    IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
    CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

## License text 192b8f5c96900f04a1271dec39688655d7416c1c6ea84a508e18b50d2b6751f3

Document names: LICENSE

Referenced by:

- pg 8.22.0 — LICENSE
- pg-cloudflare 1.4.0 — LICENSE
- pg-protocol 1.15.0 — LICENSE

    MIT License

    Copyright (c) 2010 - 2021 Brian Carlson

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 1cab22f196264195a4caec8ca5630170fdde76ee8f43346e47021d087332d3b0

Document names: LICENSE

Referenced by:

- mitt 3.0.1 — LICENSE

    MIT License

    Copyright (c) 2021 Jason Miller

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 23f18e03dc49df91622fe2a76176497404e46ced8a715d9d2b67a7446571cca3

Document names: LICENSE-MIT

Referenced by:

- itoa 1.0.18 — LICENSE-MIT
- once_cell 1.21.4 — LICENSE-MIT
- proc-macro2 1.0.106 — LICENSE-MIT
- quote 1.0.45 — LICENSE-MIT
- serde 1.0.228 — LICENSE-MIT
- serde_core 1.0.228 — LICENSE-MIT
- serde_derive 1.0.228 — LICENSE-MIT
- serde_json 1.0.150 — LICENSE-MIT
- syn 2.0.117 — LICENSE-MIT
- unicode-ident 1.0.24 — LICENSE-MIT
- zmij 1.0.21 — LICENSE-MIT

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.

---

## License text 306fa513e39b23a6e8747520de761809d206b99800ef41907b530226574c59ae

Document names: license

Referenced by:

- regexparam 3.0.0 — license

    The MIT License (MIT)

    Copyright (c) Luke Edwards <luke.edwards05@gmail.com> (lukeed.com)

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.

---

## License text 378f5840b258e2779c39418f3f2d7b2ba96f1c7917dd6be0713f88305dbda397

Document names: LICENSE-MIT

Referenced by:

- cfg-if 1.0.4 — LICENSE-MIT
- wasm-bindgen 0.2.125 — LICENSE-MIT
- wasm-bindgen-macro 0.2.125 — LICENSE-MIT
- wasm-bindgen-macro-support 0.2.125 — LICENSE-MIT
- wasm-bindgen-shared 0.2.125 — LICENSE-MIT

    Copyright (c) 2014 Alex Crichton

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.

---

## License text 3f1189b28e3866e0d979968d466b78f813f76827cfdca1fbb124cc0a5c8841f8

Document names: LICENSE

Referenced by:

- zod 4.4.3 — LICENSE

    MIT License

    Copyright (c) 2025 Colin McDonnell

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 4e8e87ccdfc7e4b47fd89015f78468aa53b6bf43ab6e6e12d43e8f55294911de

Document names: LICENSE

Referenced by:

- pg-int8 1.0.1 — LICENSE

    Copyright © 2017, Charmander <~@charmander.me>

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted, provided that the above
    copyright notice and this permission notice appear in all copies.

    THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
    REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
    FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
    INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
    LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
    OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
    PERFORMANCE OF THIS SOFTWARE.

---

## License text 4f15ee7fc2a72082859d7e0d12dfa4bcdd70b1c744ad3850d07780730ac08557

Document names: LICENSE

Referenced by:

- pg-pool 3.14.0 — LICENSE

    MIT License

    Copyright (c) 2017 Brian M. Carlson

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 65f94e99ddaf4f5d1782a6dae23f35d4293a9a01444a13135a6887017d353cee

Document names: LICENSE-MIT

Referenced by:

- bumpalo 3.20.3 — LICENSE-MIT

    Copyright (c) 2019 Nick Fitzgerald

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.

---

## License text 668dcc52803480e0a026b31140a4cae668772663cd764e5991d252eef03f98db

Document names: LICENSE

Referenced by:

- lucide-react 0.575.0 — LICENSE

    ISC License

    Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2026 as part of Feather (MIT). All other copyright (c) for Lucide are held by Lucide Contributors 2026.

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted, provided that the above
    copyright notice and this permission notice appear in all copies.

    THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
    WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
    MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
    ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
    WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
    ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
    OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

    ---

    The MIT License (MIT) (for portions derived from Feather)

    Copyright (c) 2013-2026 Cole Bemis

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 82e67379203d5794e7c44549847d8d64ae6904591381682360470898bd306821

Document names: LICENSE

Referenced by:

- xtend 4.0.2 — LICENSE

    The MIT License (MIT)
    Copyright (c) 2012-2014 Raynos.

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.

---

## License text 85747ad4bba34e96e5055af5994796ec2a8525b4cecb14bc1bb257199dc29566

Document names: LICENSE

Referenced by:

- pg-connection-string 2.14.0 — LICENSE

    The MIT License (MIT)

    Copyright (c) 2014 Iced Development

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text 8666aaf379a6509e2714d56f1276b186760db9a695054737761ace47d10fa0a4

Document names: Unlicense license text

Referenced by:

- wouter 3.10.0 — Unlicense license text

    This is free and unencumbered software released into the public domain.

    Anyone is free to copy, modify, publish, use, compile, sell, or
    distribute this software, either in source code form or as a compiled
    binary, for any purpose, commercial or non-commercial, and by any
    means.

    In jurisdictions that recognize copyright laws, the author or authors
    of this software dedicate any and all copyright interest in the
    software to the public domain. We make this dedication for the benefit
    of the public at large and to the detriment of our heirs and
    successors. We intend this dedication to be an overt act of
    relinquishment in perpetuity of all present and future rights to this
    software under copyright law.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
    EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
    MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
    IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
    OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
    ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
    OTHER DEALINGS IN THE SOFTWARE.

    For more information, please refer to <https://unlicense.org/>.

---

## License text a6ab98e5c77b9070c443eaff2ff81034a6f8cc05a7524d5098eb0f24defa0115

Document names: LICENSE, Reviewed license notice

Referenced by:

- @hono/trpc-server 0.4.2 — Reviewed license notice
- hono 4.13.0 — LICENSE

    MIT License

    Copyright (c) 2021 - present, Yusuke Wada and Hono contributors

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text b30df9a48463d1c99c6a66cdee623b1b0832c3811d58a80b338268d71cea190e

Document names: LICENSE-APACHE

Referenced by:

- itoa 1.0.18 — LICENSE-APACHE
- proc-macro2 1.0.106 — LICENSE-APACHE
- quote 1.0.45 — LICENSE-APACHE
- serde 1.0.228 — LICENSE-APACHE
- serde_core 1.0.228 — LICENSE-APACHE
- serde_derive 1.0.228 — LICENSE-APACHE
- serde_json 1.0.150 — LICENSE-APACHE
- syn 2.0.117 — LICENSE-APACHE
- unicode-ident 1.0.24 — LICENSE-APACHE

    Apache License
                            Version 2.0, January 2004
                         http://www.apache.org/licenses/

    TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

    1. Definitions.

       "License" shall mean the terms and conditions for use, reproduction,
       and distribution as defined by Sections 1 through 9 of this document.

       "Licensor" shall mean the copyright owner or entity authorized by
       the copyright owner that is granting the License.

       "Legal Entity" shall mean the union of the acting entity and all
       other entities that control, are controlled by, or are under common
       control with that entity. For the purposes of this definition,
       "control" means (i) the power, direct or indirect, to cause the
       direction or management of such entity, whether by contract or
       otherwise, or (ii) ownership of fifty percent (50%) or more of the
       outstanding shares, or (iii) beneficial ownership of such entity.

       "You" (or "Your") shall mean an individual or Legal Entity
       exercising permissions granted by this License.

       "Source" form shall mean the preferred form for making modifications,
       including but not limited to software source code, documentation
       source, and configuration files.

       "Object" form shall mean any form resulting from mechanical
       transformation or translation of a Source form, including but
       not limited to compiled object code, generated documentation,
       and conversions to other media types.

       "Work" shall mean the work of authorship, whether in Source or
       Object form, made available under the License, as indicated by a
       copyright notice that is included in or attached to the work
       (an example is provided in the Appendix below).

       "Derivative Works" shall mean any work, whether in Source or Object
       form, that is based on (or derived from) the Work and for which the
       editorial revisions, annotations, elaborations, or other modifications
       represent, as a whole, an original work of authorship. For the purposes
       of this License, Derivative Works shall not include works that remain
       separable from, or merely link (or bind by name) to the interfaces of,
       the Work and Derivative Works thereof.

       "Contribution" shall mean any work of authorship, including
       the original version of the Work and any modifications or additions
       to that Work or Derivative Works thereof, that is intentionally
       submitted to Licensor for inclusion in the Work by the copyright owner
       or by an individual or Legal Entity authorized to submit on behalf of
       the copyright owner. For the purposes of this definition, "submitted"
       means any form of electronic, verbal, or written communication sent
       to the Licensor or its representatives, including but not limited to
       communication on electronic mailing lists, source code control systems,
       and issue tracking systems that are managed by, or on behalf of, the
       Licensor for the purpose of discussing and improving the Work, but
       excluding communication that is conspicuously marked or otherwise
       designated in writing by the copyright owner as "Not a Contribution."

       "Contributor" shall mean Licensor and any individual or Legal Entity
       on behalf of whom a Contribution has been received by Licensor and
       subsequently incorporated within the Work.

    2. Grant of Copyright License. Subject to the terms and conditions of
       this License, each Contributor hereby grants to You a perpetual,
       worldwide, non-exclusive, no-charge, royalty-free, irrevocable
       copyright license to reproduce, prepare Derivative Works of,
       publicly display, publicly perform, sublicense, and distribute the
       Work and such Derivative Works in Source or Object form.

    3. Grant of Patent License. Subject to the terms and conditions of
       this License, each Contributor hereby grants to You a perpetual,
       worldwide, non-exclusive, no-charge, royalty-free, irrevocable
       (except as stated in this section) patent license to make, have made,
       use, offer to sell, sell, import, and otherwise transfer the Work,
       where such license applies only to those patent claims licensable
       by such Contributor that are necessarily infringed by their
       Contribution(s) alone or by combination of their Contribution(s)
       with the Work to which such Contribution(s) was submitted. If You
       institute patent litigation against any entity (including a
       cross-claim or counterclaim in a lawsuit) alleging that the Work
       or a Contribution incorporated within the Work constitutes direct
       or contributory patent infringement, then any patent licenses
       granted to You under this License for that Work shall terminate
       as of the date such litigation is filed.

    4. Redistribution. You may reproduce and distribute copies of the
       Work or Derivative Works thereof in any medium, with or without
       modifications, and in Source or Object form, provided that You
       meet the following conditions:

       (a) You must give any other recipients of the Work or
           Derivative Works a copy of this License; and

       (b) You must cause any modified files to carry prominent notices
           stating that You changed the files; and

       (c) You must retain, in the Source form of any Derivative Works
           that You distribute, all copyright, patent, trademark, and
           attribution notices from the Source form of the Work,
           excluding those notices that do not pertain to any part of
           the Derivative Works; and

       (d) If the Work includes a "NOTICE" text file as part of its
           distribution, then any Derivative Works that You distribute must
           include a readable copy of the attribution notices contained
           within such NOTICE file, excluding those notices that do not
           pertain to any part of the Derivative Works, in at least one
           of the following places: within a NOTICE text file distributed
           as part of the Derivative Works; within the Source form or
           documentation, if provided along with the Derivative Works; or,
           within a display generated by the Derivative Works, if and
           wherever such third-party notices normally appear. The contents
           of the NOTICE file are for informational purposes only and
           do not modify the License. You may add Your own attribution
           notices within Derivative Works that You distribute, alongside
           or as an addendum to the NOTICE text from the Work, provided
           that such additional attribution notices cannot be construed
           as modifying the License.

       You may add Your own copyright statement to Your modifications and
       may provide additional or different license terms and conditions
       for use, reproduction, or distribution of Your modifications, or
       for any such Derivative Works as a whole, provided Your use,
       reproduction, and distribution of the Work otherwise complies with
       the conditions stated in this License.

    5. Submission of Contributions. Unless You explicitly state otherwise,
       any Contribution intentionally submitted for inclusion in the Work
       by You to the Licensor shall be under the terms and conditions of
       this License, without any additional terms or conditions.
       Notwithstanding the above, nothing herein shall supersede or modify
       the terms of any separate license agreement you may have executed
       with Licensor regarding such Contributions.

    6. Trademarks. This License does not grant permission to use the trade
       names, trademarks, service marks, or product names of the Licensor,
       except as required for reasonable and customary use in describing the
       origin of the Work and reproducing the content of the NOTICE file.

    7. Disclaimer of Warranty. Unless required by applicable law or
       agreed to in writing, Licensor provides the Work (and each
       Contributor provides its Contributions) on an "AS IS" BASIS,
       WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
       implied, including, without limitation, any warranties or conditions
       of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
       PARTICULAR PURPOSE. You are solely responsible for determining the
       appropriateness of using or redistributing the Work and assume any
       risks associated with Your exercise of permissions under this License.

    8. Limitation of Liability. In no event and under no legal theory,
       whether in tort (including negligence), contract, or otherwise,
       unless required by applicable law (such as deliberate and grossly
       negligent acts) or agreed to in writing, shall any Contributor be
       liable to You for damages, including any direct, indirect, special,
       incidental, or consequential damages of any character arising as a
       result of this License or out of the use or inability to use the
       Work (including but not limited to damages for loss of goodwill,
       work stoppage, computer failure or malfunction, or any and all
       other commercial damages or losses), even if such Contributor
       has been advised of the possibility of such damages.

    9. Accepting Warranty or Additional Liability. While redistributing
       the Work or Derivative Works thereof, You may choose to offer,
       and charge a fee for, acceptance of support, warranty, indemnity,
       or other liability obligations and/or rights consistent with this
       License. However, in accepting such obligations, You may act only
       on Your own behalf and on Your sole responsibility, not on behalf
       of any other Contributor, and only if You agree to indemnify,
       defend, and hold each Contributor harmless for any liability
       incurred by, or claims asserted against, such Contributor by reason
       of your accepting any such warranty or additional liability.

    END OF TERMS AND CONDITIONS

---

## License text c372ef2fa1dfcb124ed807609751e75e3a009f108c5724916b489288fcb88a0c

Document names: LICENSE

Referenced by:

- split2 4.2.0 — LICENSE

    Copyright (c) 2014-2018, Matteo Collina <hello@matteocollina.com>

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted, provided that the above
    copyright notice and this permission notice appear in all copies.

    THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
    WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
    MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
    ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
    WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
    ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR
    IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

---

## License text da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93

Document names: LICENSE

Referenced by:

- react 19.2.7 — LICENSE
- react-dom 19.2.7 — LICENSE
- scheduler 0.27.0 — LICENSE
- use-sync-external-store 1.6.0 — LICENSE

    MIT License

    Copyright (c) Meta Platforms, Inc. and affiliates.

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text e714dd84c8fa242600844b05d317a31003423723178c1f1603dbfad1bc68d906

Document names: LICENSE

Referenced by:

- @trpc/client 11.18.0 — LICENSE
- @trpc/server 11.18.0 — LICENSE

    MIT License

    Copyright (c) 2023 Alex Johansson

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.

---

## License text e9175c300e0b6dfe281de13e9071166106b67b6e05e9d7156af2e032ffb3d31b

Document names: Reviewed license notice

Referenced by:

- pg-types 2.2.0 — Reviewed license notice

    The MIT License (MIT)

    Copyright (c) 2014 Brian M. Carlson

    Permission is hereby granted, free of charge, to any person obtaining a copy of
    this software and associated documentation files (the "Software"), to deal in
    the Software without restriction, including without limitation the rights to
    use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software is furnished to do so,
    subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
    FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
    COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
    IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
    CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

## License text ebad0332150a08f37389158289d93ab2f70b0ee8717d1db9b3d002febc6c5047

Document names: LICENSE

Referenced by:

- mp4box 2.4.1 — LICENSE

    Copyright (c) 2012. Telecom ParisTech/TSI/MM/GPAC Cyril Concolato
    All rights reserved.

    Redistribution and use in source and binary forms, with or without
    modification, are permitted provided that the following conditions are met:
        * Redistributions of source code must retain the above copyright
          notice, this list of conditions and the following disclaimer.
        * Redistributions in binary form must reproduce the above copyright
          notice, this list of conditions and the following disclaimer in the
          documentation and/or other materials provided with the distribution.
        * Neither the name of the copyright holder nor the
          names of its contributors may be used to endorse or promote products
          derived from this software without specific prior written permission.

    THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
    ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
    WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
    DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
    DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
    (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
    LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
    ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
    (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
    SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

---

## License text f057f36739d53d228a746de4440c1e0c644ecde06d6beab45337d39c9d12a393

Document names: license

Referenced by:

- postgres-array 2.0.0 — license
- postgres-bytea 1.0.1 — license
- postgres-date 1.0.7 — license
- postgres-interval 1.2.0 — license

    The MIT License (MIT)

    Copyright (c) Ben Drucker <bvdrucker@gmail.com> (bendrucker.me)

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
    THE SOFTWARE.

---

## License text f7db81051789b729fea528a63ec4c938fdcb93d9d61d97dc8cc2e9df6d47f2a1

Document names: LICENSE-UNICODE

Referenced by:

- unicode-ident 1.0.24 — LICENSE-UNICODE

    UNICODE LICENSE V3

    COPYRIGHT AND PERMISSION NOTICE

    Copyright © 1991-2023 Unicode, Inc.

    NOTICE TO USER: Carefully read the following legal agreement. BY
    DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
    SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
    TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
    DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of data files and any associated documentation (the "Data Files") or
    software and any associated documentation (the "Software") to deal in the
    Data Files or Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, and/or sell
    copies of the Data Files or Software, and to permit persons to whom the
    Data Files or Software are furnished to do so, provided that either (a)
    this copyright and permission notice appear with all copies of the Data
    Files or Software, or (b) this copyright and permission notice appear in
    associated Documentation.

    THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
    KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
    MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
    THIRD PARTY RIGHTS.

    IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
    BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
    OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
    WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
    ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
    FILES OR SOFTWARE.

    Except as contained in this notice, the name of a copyright holder shall
    not be used in advertising or otherwise to promote the sale, use or other
    dealings in these Data Files or Software without prior written
    authorization of the copyright holder.

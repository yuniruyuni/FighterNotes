# ---- WASM build (Rust → wasm-pack) --------------------------------------
FROM rust:1.99@sha256:e259494b14672950a909b4c54527e19d29cae6ebf980672114c8bb85e5594e4a AS build-wasm
WORKDIR /work
RUN rustup target add wasm32-unknown-unknown \
 && cargo install wasm-pack --version 0.15.0 --locked
COPY Cargo.toml Cargo.lock /work/
COPY crates/ /work/crates/
# SHA を読むのは advice-report の option_env! だけ。commit ごとに変わる値なので、
# toolchain の層より後ろに置く。前に置くと以降の層が毎 commit 無効になる。
ARG FIGHTER_NOTES_BUILD_SHA=dev
ENV FIGHTER_NOTES_BUILD_SHA=${FIGHTER_NOTES_BUILD_SHA}
RUN wasm-pack build crates/wasm-bridge --target web

# ---- Third-party notices (bun.lock + Cargo.lock → license inventory) ----
# 通知は commit せず、image を作るたびに lockfile から生成する。npm 側は
# node_modules を、Cargo 側は cargo-about を読むので、Rust と Bun の両方が要る。
# 許可されていない license が混ざっていればここで build が失敗する。
FROM rust:1.99@sha256:e259494b14672950a909b4c54527e19d29cae6ebf980672114c8bb85e5594e4a AS build-notices
WORKDIR /work
COPY --from=oven/bun:1.4.3@sha256:ec06c3b6cea04192ae6770c434f668ca41d343ad19fa6472216c7b48be39c598 /usr/local/bin/bun /usr/local/bin/bun
# node_modules/.bin の shim (biome など) は `#!/usr/bin/env node` で起動する。
# oven/bun の image と同じく node を bun に向けておく。
RUN ln -s bun /usr/local/bin/node
RUN cargo install cargo-about --version 0.9.1 --locked --features cli
COPY package.json bun.lock /work/
COPY client/package.json /work/client/
COPY server/package.json /work/server/
RUN bun install --frozen-lockfile
COPY . /work/
RUN cargo fetch --locked \
 && bun scripts/generate-third-party-notices.ts

# ---- Client build (Bun bundler; consumes wasm pkg) ----------------------
FROM oven/bun:1.4.3@sha256:ec06c3b6cea04192ae6770c434f668ca41d343ad19fa6472216c7b48be39c598 AS build-client
WORKDIR /work
COPY package.json bun.lock /work/
COPY client/package.json /work/client/
COPY server/package.json /work/server/
RUN bun install --frozen-lockfile
COPY client/ /work/client/
COPY DATA_NOTICE.md /work/DATA_NOTICE.md
COPY --from=build-wasm /work/crates/wasm-bridge/pkg/ /work/crates/wasm-bridge/pkg/
COPY --from=build-notices /work/THIRD_PARTY_NOTICES.md /work/THIRD_PARTY_NOTICES.md
COPY --from=build-notices /work/client/src/generated/ /work/client/src/generated/
# 認識の shader。参照実装である Rust の隣に置いてあり、束ねる際に読み込む。
COPY crates/hud-vision/shaders/ /work/crates/hud-vision/shaders/
WORKDIR /work/client
RUN bun run build:app

# ---- Server build (Hono → single binary) --------------------------------
FROM oven/bun:1.4.3@sha256:ec06c3b6cea04192ae6770c434f668ca41d343ad19fa6472216c7b48be39c598 AS build-server
WORKDIR /work
COPY package.json bun.lock /work/
COPY client/package.json /work/client/
COPY server/package.json /work/server/
RUN bun install --frozen-lockfile
COPY server/ /work/server/
WORKDIR /work/server
RUN NODE_ENV=production bun build src/index.ts --compile --outfile /work/dist/server

# ---- Runtime (distroless) -----------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
WORKDIR /app
COPY --from=build-server /work/dist/server /app/server
COPY --from=build-client /work/client/static/ /app/static/
ENV STATIC_DIR=./static NODE_ENV=production
EXPOSE 3000
CMD ["./server"]

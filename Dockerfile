# ---- WASM build (Rust → wasm-pack) --------------------------------------
FROM rust:1.95@sha256:f49565f188ee00bc2a18dd418183f2c5f23ef7d6e691890517ed341a598f67c3 AS build-wasm
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

# ---- Client build (Bun bundler; consumes wasm pkg) ----------------------
FROM oven/bun:1.3.14@sha256:e10577f0db68676a7024391c6e5cb4b879ebd17188ab750cf10024a6d700e5c4 AS build-client
WORKDIR /work
COPY package.json bun.lock /work/
COPY client/package.json /work/client/
COPY server/package.json /work/server/
RUN bun install --frozen-lockfile
COPY client/ /work/client/
COPY DATA_NOTICE.md /work/DATA_NOTICE.md
COPY THIRD_PARTY_NOTICES.md /work/THIRD_PARTY_NOTICES.md
COPY --from=build-wasm /work/crates/wasm-bridge/pkg/ /work/crates/wasm-bridge/pkg/
WORKDIR /work/client
RUN bun run build:app

# ---- Server build (Hono → single binary) --------------------------------
FROM oven/bun:1.3.14@sha256:e10577f0db68676a7024391c6e5cb4b879ebd17188ab750cf10024a6d700e5c4 AS build-server
WORKDIR /work
COPY package.json bun.lock /work/
COPY client/package.json /work/client/
COPY server/package.json /work/server/
RUN bun install --frozen-lockfile
COPY server/ /work/server/
WORKDIR /work/server
RUN NODE_ENV=production bun build src/index.ts --compile --outfile /work/dist/server

# ---- Runtime (distroless) -----------------------------------------------
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:ce0d66bc0f64aae46e6a03add867b07f42cc7b8799c949c2e898057b7f75a151
WORKDIR /app
COPY --from=build-server /work/dist/server /app/server
COPY --from=build-client /work/client/static/ /app/static/
ENV STATIC_DIR=./static NODE_ENV=production
EXPOSE 3000
CMD ["./server"]

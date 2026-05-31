# vacation 🌊🐕

A little **egui + walkers (Rust → WebAssembly)** site for sharing our August 2026
Outer Banks road trip — Rob, Rachael & Poppy. Immediate-mode UI (à la Dear ImGui)
with an interactive OpenStreetMap map, a driving-route line, and a labeled pin for
each stop. Runs natively *and* in the browser from a single codebase.

## The trip

Arlington, VA → **Duck, OBX** (4 nights, dog-friendly beach all day) → day trips to
**Corolla** (wild horses) and **Nags Head** (Jockey's Ridge) → **Duke / Durham**
(2 nights with friends) → home. Aug 10–16, 2026.

## Run it

One-time setup (Rust toolchain + WASM target + Trunk):

```sh
# 1. Install Rust (if you don't have it): https://rustup.rs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add the web target + the Trunk bundler
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Develop in the browser with hot reload:

```sh
trunk serve --open      # http://127.0.0.1:8080
```

Run the native desktop version (same app):

```sh
cargo run
```

## Deploy

```sh
trunk build --release   # outputs a static site to ./dist
```

Upload `dist/` to any static host (GitHub Pages, Netlify, Cloudflare Pages) — same
workflow as the animal-intelligence site. Buy the domain after it's live.

> Note: GitHub Pages serves from a subpath, so for that host build with
> `trunk build --release --public-url /<repo-name>/`.

## Layout

- `src/trip.rs` — the itinerary data (stops, coordinates, the driving route).
- `src/app.rs` — the egui app: side panel + map + custom `walkers` plugins (route line, pins).
- `src/main.rs` — native and web (`wasm32`) entry points.
- `index.html` / `Trunk.toml` — web build config.

## Versions

`egui`/`eframe` 0.34 · `walkers` 0.53. The `glow` (WebGL) renderer is used for the
widest browser support, including Safari.

# vacation ⛰️🐕

A little **egui + walkers (Rust → WebAssembly)** site for sharing our August 2026
Blue Ridge road trip — Rob, Rachael & Poppy. Immediate-mode UI (à la Dear ImGui)
with an interactive OpenStreetMap map, a driving-route line, and a labeled pin for
each stop. Runs natively *and* in the browser from a single codebase. A static
`itinerary.html` (the pretty day-by-day page) is served alongside the app.

## The trip

Arlington, VA → **Damascus** (trail-town night) → backpack the **Grayson
Highlands / Mount Rogers** high country (wild ponies, one night camped) →
**West Jefferson, NC** river base (2 nights) with a **New River** canoe day →
one night camped on **Grassy Ridge Bald** in the Roan Highlands → home.
Aug 7–13, 2026. (The earlier OBX plan this site once showed was cancelled.)

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

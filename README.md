# Conway's Game of Life

This is a small Rust + WebAssembly project that implements the 
classic Conway's Game of Life cellular automaton in Rust, compiled to WebAssembly (Wasm) and run in the browser

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack`
- Node.js (for the webpack/TypeScript frontend)

## Build and run (development)

1. Build the wasm package (from the repo root):

```
wasm-pack build --target web
```

2. Start the web dev server:

```
cd web
npm install
npm run start
```

3. Open `http://localhost:8080` in your browser.

If you change the Rust code, re-run `wasm-pack build --target web` to refresh `pkg/`.

## Build (production)

```
cd web
npm install
npm run build
```

The compiled assets are emitted to `web/dist`.

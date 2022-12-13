## Introduction
Example full stack Rust project, using actix-web and dioxus and bulma-css.

Adapted from https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/.

Attempts to implement BFF (backend for frontend) pattern for login using oidc. Inspired by Duendesoftware's implementation.

### Instructions
1. Build frontend distribution using Trunk
```bash
cd frontend
trunk build
```
2. This will build the wasm bundle in the root ./dist folder
3. Next, compile the backend from the root folder
```bash
cargo build --release
```
4. Run the binary
```
target/release/server
```
5. Open your browser at localhost:8080
## Introduction
Example full stack Rust project, using actix-web and dioxus and bulma-css.

Adapted from https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/.

Attempts to implement BFF (backend for frontend) pattern for login using oidc. Inspired by Duendesoftware's implementation.

### Prerequisites
1. Set up google oidc client at https://console.cloud.google.com/apis/credentials
2. Get GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET and set them as environment variables before you run the server binary
3. Make sure to set authorised redirect uri to http://localhost:8080/api/token_exchange

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
# Fathom

A quick entry for the ludum dare 57 game jam (48-hour compo rules), entirely in rust + WASM

# Building and running

You'll need a rust toolchain, the wasm32-unknown-unknown target, and [trunk](https://trunkrs.dev)

Then run `trunk serve`, and you should be able to play the game at http://127.0.0.1:8080 with hot reload as you edit the source code
```sh
> trunk serve
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
2025-04-06T21:02:43.096798Z  INFO applying new distribution
2025-04-06T21:02:43.097326Z  INFO ✅ success
2025-04-06T21:02:43.098768Z  INFO 📡 serving static assets at -> /
2025-04-06T21:02:43.098820Z  INFO 📡 server listening at:
2025-04-06T21:02:43.098834Z  INFO     🏠 http://127.0.0.1:8080/
2025-04-06T21:02:43.098837Z  INFO     🏠 http://[::1]:8080/
```

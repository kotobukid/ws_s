# websocket server & client sample

and Rust practice

## dev

```bash
$ cargo run --bin server 127.0.0.1:8080
```

```bash
$ cargo run --bin client ws://127.0.0.1:8080/ws
```

```bash
cargo test -p message-pack
```

## generate
```bash
cd message-pack-wasm
build.bat

cd ../front
npm run generate

cd ..
cargo build --release --bin server
```

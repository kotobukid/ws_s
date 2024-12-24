# websocket server & client sample

and Rust practice

## dev

```bash
$ cargo run --bin server -- --hostname 127.0.0.1:8080
```

```bash
$ cargo run --bin client -- --hostname 127.0.0.1:8080
```

```bash
cargo test -p message-pack
```

## build

```bash
cd message-pack-wasm
build.bat

cd ../front
npm run generate -- --hostname 127.0.0.1:8080

cd ..
cargo build --release --bin server
```

set RUSTFLAGS=-C link-arg=-s
cargo build --target wasm32-unknown-unknown --release
xcopy .\target\wasm32-unknown-unknown\release\hex_game.wasm .\res\hex_game.wasm

if not defined in_subprocess (cmd /k set in_subprocess=y ^& %0 %*) & exit )
set RUSTFLAGS=-C link-arg=-s
cargo build --target wasm32-unknown-unknown --release
xcopy .\target\wasm32-unknown-unknown\release\hex_game.wasm .\res\hex_game.wasm

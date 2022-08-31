if not defined in_subprocess (cmd /k set in_subprocess=y ^& %0 %*) & exit )
near deploy --accountId cryptoneonhex_d954mas.testnet --wasmFile res/hex_game.wasm --initFunction new --initArgs '{}'
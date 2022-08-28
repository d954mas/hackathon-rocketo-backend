The Game of Hex
===============
Hex is a two player abstract strategy board game in which players attempt to connect opposite sides of a rhombus-shaped board made of hexagonal cells. Hex was invented by mathematician and poet Piet Hein in 1942 and later rediscovered and popularized by John Nash. 

## Description
This contract implements hex-game backed by storage on NEAR blockchain.
Contract in `src/lib.rs` provides methods to create new game, make an allowed move in one of already existing games or view information about game by index.

The project is divided into separate files, each file contains one of the structures, that is used to keep information about the game, and tests for this structure.

## Interacting with contract

Deployed game contract in testnet: `cryptoneonhex_d954mas.testnet`

#### `create_game(first_player: AccountId, second_player: AccountId, field_size: usize) -> GameIndex`

Creates new game with given parameters and returns index of created game. For example:
```console
➜ near call hex-game.klimoza.testnet create_game '{"first_player": "crossword.klimoza.testnet", "second_player": "klimoza.testnet", "field_size": 2}' --accountId hex-game.klimoza.testnet --amount 2
Scheduling a call: hex-game.klimoza.testnet.create_game({"first_player": "crossword.klimoza.testnet", "second_player": "klimoza.testnet", "field_size": 2})
Doing account.functionCall()

	Log [hex-game.klimoza.testnet]: Created board:
	Log [hex-game.klimoza.testnet]: . .
	Log [hex-game.klimoza.testnet]:  . .

4
```

#### `make_move(index: GameIndex, move_type: MoveType, cell: Option<Cell>) -> Game`
Tries to make a move in the game at the given index and returns Game if move is correct(panics otherwise). Used structures:
```rust
pub type GameIndex = u64;

pub enum MoveType {
    PLACE,
    SWAP,
}

pub struct Cell {
    pub x: usize,
    pub y: usize,
}
```
You can omit the `cell` parameter if `move_type` is `SWAP`(i.e. applying swap rule on the current move). For example:
```console
➜ near call hex-game.klimoza.testnet make_move '{"index": 4, "move_type": "SWAP"}' --accountId klimoza.testnet
Scheduling a call: hex-game.klimoza.testnet.make_move({"index": 4, "move_type": "SWAP"})
Doing account.functionCall()

	Log [hex-game.klimoza.testnet]: Old board:
	Log [hex-game.klimoza.testnet]: . R
	Log [hex-game.klimoza.testnet]:  . .
	Log [hex-game.klimoza.testnet]: New board:
	Log [hex-game.klimoza.testnet]: . .
	Log [hex-game.klimoza.testnet]:  B .

{
  first_player: 'crossword.klimoza.testnet',
  second_player: 'klimoza.testnet',
  turn: 2,
  board: { size: 2, field: 'IA==' },
  current_block_height: 96244955,
  prev_block_height: 96244934,
  is_finished: false
}
```

#### `get_game(index: GameIndex) -> Option<Game>`
Returns the game at the given index(if there is one). For example:
```console
➜ near call hex-game.klimoza.testnet get_game '{"index": 4}' --accountId hex-game.klimoza.testnet
Scheduling a call: hex-game.klimoza.testnet.get_game({"index": 4})
Doing account.functionCall()

	Log [hex-game.klimoza.testnet]: Game board:
	Log [hex-game.klimoza.testnet]: R B
	Log [hex-game.klimoza.testnet]:  B .

{
  first_player: 'crossword.klimoza.testnet',
  second_player: 'klimoza.testnet',
  turn: 4,
  board: { size: 2, field: 'KQ==' },
  current_block_height: 96244985,
  prev_block_height: 96244971,
  is_finished: true
}
```

#### `check_premium_account(account_id: AccountId) -> bool`
Checks for a locked, expirable, active Roketo stream going from `account_id` to `hex_game_account`. Returns Promise. For example:
```console
➜ near call wrap.testnet ft_transfer_call '{"receiver_id": "streaming-r-v2.dcversus.testnet",  "amount": "2200000000000000000000000", "memo": "Roketo transfer", "msg": "{\"Create\":{\"request\":{\"balance\":\"2000000000000000000000000\", \"owner_id\": \"klimoza.testnet\",\"receiver_id\":\"hex-game.klimoza.testnet\",\"token_name\": \"wrap.testnet\", \"tokens_per_sec\":\"6666666666666666666667\", \"is_locked\": true, \"is_expirable\": true}}}"}' --accountId klimoza.testnet --depositYocto 1 --gas 200000000000000
Doing account.functionCall()

	Log [wrap.testnet]: Transfer 2200000000000000000000000 from klimoza.testnet to streaming-r-v2.dcversus.testnet
	Log [wrap.testnet]: Memo: Roketo transfer

	Log [wrap.testnet]: Transfer 2100000000000000000000000 from streaming-r-v2.dcversus.testnet to finance-r-v2.dcversus.testnet

'2200000000000000000000000'

➜ near call hex-game.klimoza.testnet check_premium_account '{"account_id": "klimoza.testnet"}' --accountId klimoza.testnet
Scheduling a call: hex-game.klimoza.testnet.check_premium_account({"account_id": "klimoza.testnet"})

true
```

## Testing
At the moment, the projects contains 33 tests, each of which lies in the file of the structure it is testing. You can run them all using following command:
```console
cargo test
```
Alternatively, you can specify test group you want to run, for example:
```console
cargo test cell_tests
```

## Demonstration

[![Video](https://img.youtube.com/vi/mwgUEafpeow/0.jpg)](https://youtu.be/mwgUEafpeow)
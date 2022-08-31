use cell::Cell;
use external::{Stream, StreamStatus};
use game::{Game, GameIndex, MoveType};
use game_with_data::GameWithData;
use near_contract_standards::non_fungible_token::refund_deposit;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault, Promise};
use roketo::get_account_outgoing_streams;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Games,
    Field { game_id: GameIndex },
}



#[derive(BorshDeserialize, BorshSerialize)]
pub struct PlayerData {
    pub id:AccountId,
    pub games:Vector<GameIndex>,
}

impl PlayerData {
pub fn new(player_id: AccountId) -> Self {
      let prefix: Vec<u8> = [
                        b"s".as_slice(),
                        &near_sdk::env::sha256_array(player_id.as_bytes()),
                   ].concat();
           PlayerData {
               id: player_id,
                // Constructing a unique prefix for a nested UnorderedSet from a concatenation
                // of a prefix and a hash of the account id.
               games: Vector::new(prefix),
           }
       }
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub games: Vector<GameWithData>,
    pub players: UnorderedMap<AccountId,PlayerData>,
    pub roketo_acc: Option<AccountId>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(roketo_acc: Option<AccountId>) -> Self {
        Self {
            games: Vector::new(StorageKey::Games),
            //https://docs.near.org/sdk/rust/contract-structure/collections
            players:  UnorderedMap::new(b"s".to_vec()), // // Initializing `players` with unique key prefix.
            roketo_acc,
        }
    }

    #[payable]
    pub fn create_game(
        &mut self,
        first_player: AccountId,
        second_player: AccountId,
        field_size: Option<usize>,
    ) -> GameIndex {
        let initial_storage_usage = env::storage_usage();

         require!(
                    first_player==env::signer_account_id(),
                    "You are not first player"
                );
         require!(
                first_player!=second_player,
                "Can't play with yourself"
            );
        let index = self.games.len();


        let playerData1Check = self.players.get(&first_player);
        if(playerData1Check.is_none()){
            self.players.insert(&first_player, &PlayerData::new(AccountId::new_unchecked(first_player.to_string())));
        }

        let playerData2Check= self.players.get(&second_player);
        if(playerData2Check.is_none()){
            self.players.insert(&second_player, &PlayerData::new(AccountId::new_unchecked(second_player.to_string())));
        }

        let mut player1 = self.players.get(&first_player).unwrap();
        player1.games.push(&index);
        self.players.insert(&first_player,&player1);

        //near_sdk::env::state_write(&self.players.get(&first_player).unwrap().games);


        let mut player2 = self.players.get(&second_player).unwrap();
        player2.games.push(&index);
        self.players.insert(&second_player,&player2);


        let size = field_size.unwrap_or(11);
        self.games
            .push(&GameWithData::new(first_player, second_player, size));

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
        refund_deposit(required_storage_in_bytes);

        env::log_str("Created board:");
        self.games.get(index).unwrap().game.board.debug_logs();
        index
    }

    pub fn get_game(&self, index: GameIndex) -> Option<Game> {
        let game = self.games.get(index).map(|x| x.game);
        if game.is_some() {
            env::log_str("Game board:");
            game.clone().unwrap().board.debug_logs();
        }
        game
    }

    pub fn make_move(&mut self, index: GameIndex, move_type: MoveType, cell: Option<Cell>) -> Game {
        let mut game_with_data = self.games.get(index).expect("Game doesn't exist.");
        require!(
            !game_with_data.game.is_finished,
            "Game is already finished!"
        );

        let old_board = game_with_data.game.board.clone();
        game_with_data.make_move(move_type, cell);

        env::log_str("Old board:");
        old_board.debug_logs();

        env::log_str("New board:");
        game_with_data.game.board.debug_logs();

        if game_with_data.game.is_finished {
            if game_with_data.game.turn % 2 == 1 {
                env::log_str("First player wins!");
            } else {
                env::log_str("Second player wins!");
            }
        }

        self.games.replace(index, &game_with_data);
        return self.games.get(index).unwrap().game;
    }

     pub fn get_games_list(&self, player: AccountId) -> Vec<GameIndex> {
        let mut vector = Vec::new();
        let playerData = self.players.get(&player);
        if(playerData.is_some()){
             for f in playerData.unwrap().games.iter() {
               vector.push(f);
            }
        }
        vector
    }

     pub fn get_games_active_list(&self, player: AccountId) -> Vec<GameIndex> {
            let mut vector = Vec::new();
            let playerData = self.players.get(&player);
            if(playerData.is_some()){
                 for f in playerData.unwrap().games.iter() {
                    let game = self.games.get(f).map(|x| x.game);
                    if game.is_some() {
                        if(!game.unwrap().is_finished){
                            vector.push(f);
                        }
                    }
                }
            }
            vector
        }




    pub fn check_premium_account(&self, account_id: AccountId) -> Promise {
        get_account_outgoing_streams(
            account_id,
            self.roketo_acc
                .clone()
                .expect("No Roketo account to check premium."),
        )
        .then(Self::ext(env::current_account_id()).check_premium_account_internal())
    }

    #[private]
    pub fn check_premium_account_internal(&self, #[callback_unwrap] streams: Vec<Stream>) -> bool {
        streams.iter().any(|stream| {
            stream.is_locked
                && stream.is_expirable
                && stream.status == StreamStatus::Active
                && stream.receiver_id == env::current_account_id()
                && stream.available_to_withdraw() != stream.balance
        })
    }
}

pub mod board;
pub mod cell;
pub mod external;
pub mod game;
pub mod game_with_data;
pub mod roketo;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod contract_tests {
    use core::fmt::Debug;
    use near_sdk::{
        test_utils::{accounts, VMContextBuilder},
        testing_env, AccountId, ONE_NEAR,
    };

    use crate::{
        board::Board, cell::Cell, game::Game, game_with_data::GameWithData, Contract, MoveType,
    };

    fn get_context(account: AccountId) -> near_sdk::VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(account)
            .attached_deposit(10 * ONE_NEAR)
            .build()
    }

    impl PartialEq for Game {
        fn eq(&self, other: &Self) -> bool {
            self.first_player == other.first_player
                && self.second_player == other.second_player
                && self.turn == other.turn
                && self.board == other.board
                && self.current_block_height == other.current_block_height
                && self.prev_block_height == other.prev_block_height
                && self.is_finished == other.is_finished
        }
    }

    impl PartialEq for GameWithData {
        fn eq(&self, other: &Self) -> bool {
            self.game == other.game && self.data == other.data
        }
    }

    impl Debug for Game {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Game")
                .field("first_player", &self.first_player)
                .field("second_player", &self.second_player)
                .field("turn", &self.turn)
                .field("board", &self.board)
                .field("current_block_height", &self.current_block_height)
                .field("prev_block_height", &self.prev_block_height)
                .field("is_finished", &self.is_finished)
                .finish()
        }
    }

    impl Debug for GameWithData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("GameWithData")
                .field("game", &self.game)
                .field("data", &self.data)
                .finish()
        }
    }

    #[test]
    fn test_create_get() {
        testing_env!(get_context(accounts(2)));
        let mut contract = Contract::new(None);
        contract.create_game(accounts(1), accounts(2), Some(3));
        contract.create_game(accounts(4), accounts(3), Some(4));
        let id = contract.create_game(accounts(0), accounts(1), None);
        assert_eq!(id, 2);
        let game = contract.get_game(id);

        assert!(contract.get_game(id + 1).is_none());
        assert!(game.is_some());
        assert_eq!(game.clone().unwrap().first_player, accounts(0));
        assert_eq!(game.clone().unwrap().second_player, accounts(1));
        assert_eq!(game.unwrap().board, Board::new(11));
    }

    #[test]
    fn test_make_move() {
        testing_env!(get_context(accounts(2)));
        let mut contract = Contract::new(None);
        let id = contract.create_game(accounts(0), accounts(1), Some(5));

        testing_env!(get_context(accounts(0)));
        let mut test_game = GameWithData::new(accounts(0), accounts(1), 5);
        assert_eq!(test_game, contract.games.get(id).unwrap());

        let game = contract.make_move(id, MoveType::PLACE, Some(Cell::new(4, 0)));
        test_game.make_move(MoveType::PLACE, Some(Cell::new(4, 0)));
        assert_eq!(test_game.game, game);
        assert_eq!(test_game, contract.games.get(id).unwrap());

        testing_env!(get_context(accounts(1)));
        let game = contract.make_move(id, MoveType::SWAP, Some(Cell::new(4, 0)));
        test_game.make_move(MoveType::SWAP, Some(Cell::new(4, 0)));
        assert_eq!(test_game.game, game);
        assert_eq!(test_game, contract.games.get(id).unwrap());
    }
}

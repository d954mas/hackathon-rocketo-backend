use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, require, AccountId, BlockHeight};

use crate::board::Board;
use crate::cell::Cell;
use near_sdk::collections::Vector;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MoveHistory {
    pub cell:Cell,
    pub move_type: MoveType
}

impl MoveHistory {
    pub fn new(cell: Cell, move_type: MoveType) -> Self {
        Self { cell, move_type }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum MoveType {
    PLACE,
    SWAP,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Game {
    pub first_player: AccountId,
    pub second_player: AccountId,
    pub turn: usize,
    pub board: Board,
    pub current_block_height: BlockHeight,
    pub prev_block_height: BlockHeight,
    pub is_finished: bool,
    pub give_up: i32, //0-no give_up 1-first_player, 2-second_player
    pub history:Vec<MoveHistory>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GameForFront {
    pub first_player: AccountId,
    pub second_player: AccountId,
    pub turn: usize,
    pub board: Vec<String>,
    pub current_block_height: BlockHeight,
    pub prev_block_height: BlockHeight,
    pub is_finished: bool,
    pub give_up: i32, //0-no give_up 1-first_player, 2-second_player
    pub history:Vec<MoveHistory>
}

impl GameForFront {
    pub fn new(game:&mut  Game) -> Self {
        GameForFront {
            first_player:AccountId::new_unchecked(game.first_player.to_string()),
            second_player:AccountId::new_unchecked(game.second_player.to_string()),
            turn:game.turn,
            board: game.board.get_board_as_strings(),
            current_block_height: game.current_block_height,
            prev_block_height: game.prev_block_height,
            is_finished: game.is_finished,
            give_up: game.give_up,
            history:  game.history.clone()
        }
    }
}

pub type GameIndex = u64;

impl Game {
    pub fn new(first_player: AccountId, second_player: AccountId, field_size: usize) -> Self {
        Game {
            first_player,
            second_player,
            turn: 0,
            board: Board::new(field_size),
            current_block_height: env::block_height(),
            prev_block_height: 0,
            is_finished: false,
            give_up:0,
            history:Vec::new()
        }
    }

    pub fn place_counter(&mut self, cell: &Cell, player: u8) {
        require!(self.board.get_cell(cell) == 0, "Cell is already filled.");
        require!(
            self.turn % 2 + 1 == player as usize,
            "It's another player turn now."
        );
        self.board.set_cell(cell, player);
        self.turn += 1;
        if env::block_height() != self.current_block_height {
            self.prev_block_height = self.current_block_height;
            self.current_block_height = env::block_height();
        }
    }

    pub fn serializeForFront(&mut self) -> GameForFront {
        let result = GameForFront::new(self);

        result
    }

    pub fn swap_rule(&mut self) -> Cell {
        require!(
            self.turn == 1,
            "Swap rule can be applied only on the second player first turn"
        );

        let non_zero_byte = self
            .board
            .field
            .0
            .iter()
            .enumerate()
            .find(|(_, &x)| x != 0)
            .unwrap();
        let mut bit_number = 8 * non_zero_byte.0;
        if non_zero_byte.1 & 4 == 4 {
            bit_number += 2;
        } else if non_zero_byte.1 & 16 == 16 {
            bit_number += 4;
        } else if non_zero_byte.1 & 64 == 64 {
            bit_number += 6;
        }
        let cell = self.board.get_coords(bit_number);

        self.board.set_cell(&cell, 0);
        self.board.set_cell(&cell.symm(), 2);
        self.turn += 1;
        if env::block_height() != self.current_block_height {
            self.prev_block_height = self.current_block_height;
            self.current_block_height = env::block_height();
        }

        cell
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod game_tests {
    use near_sdk::{
        test_utils::{accounts, VMContextBuilder},
        testing_env,
    };

    use crate::cell::Cell;

    use super::Game;

    fn get_context() -> VMContextBuilder {
        VMContextBuilder::new()
    }

    #[test]
    #[should_panic]
    fn test_place_counter_wrong_player_1() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(1, 1), 2);
    }

    #[test]
    #[should_panic]
    fn test_place_counter_wrong_player_2() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(1, 1), 1);
        game.place_counter(&Cell::new(2, 1), 1);
    }

    #[test]
    #[should_panic]
    fn test_place_counter_cell_is_already_filled() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(1, 1), 1);
        game.place_counter(&Cell::new(1, 1), 2);
    }

    #[test]
    fn test_place_counter() {
        testing_env!(get_context().block_index(0).build());

        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(1, 1), 1);
        game.place_counter(&Cell::new(1, 2), 2);
        game.place_counter(&Cell::new(10, 7), 1);
        assert_eq!(game.current_block_height, 0);
        assert_eq!(game.prev_block_height, 0);

        testing_env!(get_context().block_index(100).build());
        game.place_counter(&Cell::new(5, 9), 2);
        game.place_counter(&Cell::new(3, 7), 1);
        assert_eq!(game.current_block_height, 100);
        assert_eq!(game.prev_block_height, 0);

        assert_eq!(game.first_player, accounts(0));
        assert_eq!(game.second_player, accounts(1));
        assert_eq!(game.turn, 5);
        for i in 0..11 {
            for j in 0..11 {
                let cell = Cell::new(i, j);
                if cell == Cell::new(1, 1) || cell == Cell::new(10, 7) || cell == Cell::new(3, 7) {
                    assert_eq!(game.board.get_cell(&cell), 1);
                } else if cell == Cell::new(1, 2) || cell == Cell::new(5, 9) {
                    assert_eq!(game.board.get_cell(&cell), 2);
                } else {
                    assert_eq!(game.board.get_cell(&cell), 0);
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_swap_rule_too_early() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.swap_rule();
    }

    #[test]
    #[should_panic]
    fn test_swap_rule_too_late() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(2, 5), 1);
        game.place_counter(&Cell::new(10, 7), 2);
        game.swap_rule();
    }

    #[test]
    fn test_swap_rule() {
        let mut game = Game::new(accounts(0), accounts(1), 11);
        game.place_counter(&Cell::new(10, 7), 1);

        let c = game.swap_rule();
        assert_eq!(Cell::new(10, 7), c);
        assert_eq!(0, game.board.get_cell(&Cell::new(10, 7)));
        assert_eq!(2, game.board.get_cell(&Cell::new(7, 10)));

        game.place_counter(&Cell::new(1, 1), 1);
        game.place_counter(&Cell::new(5, 9), 2);
        game.place_counter(&Cell::new(3, 7), 1);

        assert_eq!(game.turn, 5);
        for i in 0..11 {
            for j in 0..11 {
                let cell = Cell::new(i, j);
                if cell == Cell::new(1, 1) || cell == Cell::new(3, 7) {
                    assert_eq!(game.board.get_cell(&cell), 1);
                } else if cell == Cell::new(7, 10) || cell == Cell::new(5, 9) {
                    assert_eq!(game.board.get_cell(&cell), 2);
                } else {
                    assert_eq!(game.board.get_cell(&cell), 0);
                }
            }
        }
    }
}

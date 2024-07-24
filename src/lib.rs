#![no_std]

use gstd::{debug, exec, msg, prelude::*};
use pebbles_game_io::*;

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct PebbleGame {
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub pebbles_remaining: u32,
    pub program_lastmove: u32,
    pub difficulty: DifficultyLevel,
    pub first_player: Player,
    pub winner: Option<Player>,
}

impl PebbleGame {
    fn user_move(&mut self, count: u32) {
        if count < 1 || count > self.max_pebbles_per_turn {
            panic!(
                "Invalid move. You can remove between 1 and {} pebbles.",
                self.max_pebbles_per_turn
            );
        }

        if count > self.pebbles_remaining {
            panic!("Invalid move. Not enough pebbles remaining.");
        }

        self.pebbles_remaining -= count;

        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::User);
            msg::reply(PebblesEvent::Won(Player::User), 0).unwrap();
            return;
        }
        self.program_move();
    }
    fn program_move(&mut self) {
        let mut count = 1;
        if self.max_pebbles_per_turn != 1 {
            count = program_turn_gen(self.difficulty.clone(), self.max_pebbles_per_turn,self.pebbles_remaining);
        }
        if count > self.pebbles_remaining {
            count = self.pebbles_remaining;
        }
        self.pebbles_remaining -= count;
        self.program_lastmove = count;
        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).unwrap();
            return;
        }
        debug!("turn count=={}", self.pebbles_count - self.pebbles_remaining);
        msg::reply(PebblesEvent::CounterTurn(count), 0).unwrap();
    }
    fn restart(
        &mut self,
        difficulty: DifficultyLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
    ) {
        self.difficulty = difficulty;
        self.pebbles_count = pebbles_count;
        self.max_pebbles_per_turn = max_pebbles_per_turn;
        self.pebbles_remaining = self.pebbles_count;
        self.winner = None;
        self.program_lastmove = 0;
        self.first_play();
    }
    fn first_play(&mut self) {
        if get_random_u32() % 2 == 0 {
            self.first_player = Player::User;
            msg::reply(PebblesEvent::CounterTurn(0), 0).unwrap();
        } else {
            self.first_player = Player::Program;
            self.program_move();
        };
    }
}

static mut PEBBLE_GAME: Option<PebbleGame> = None;

#[no_mangle]
extern "C" fn init() {
    let config: PebblesInit = msg::load().expect("Unable to decode InitConfig");
    if config.max_pebbles_per_turn > config.pebbles_count {
        panic!("invalid pebbles init.");
    }
    let mut game = PebbleGame {
        pebbles_count: config.pebbles_count,
        max_pebbles_per_turn: config.max_pebbles_per_turn,
        difficulty: config.difficulty,
        pebbles_remaining: config.pebbles_count,
        ..Default::default()
    };
    game.first_play();
    debug!(
        "game==init=={},{},{}",
        game.pebbles_count, game.max_pebbles_per_turn, game.pebbles_remaining
    );
    unsafe { PEBBLE_GAME = Some(game) };
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Could not load Action");
    let game = unsafe { PEBBLE_GAME.get_or_insert(Default::default()) };
    game.program_lastmove = 0;
    match action {
        PebblesAction::Turn(count) => {
            game.user_move(count);
        }
        PebblesAction::GiveUp => {
            game.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program), 0).unwrap();
        }
        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
        } => {
            game.restart(difficulty, pebbles_count, max_pebbles_per_turn);
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let gmst = unsafe { PEBBLE_GAME.get_or_insert(Default::default()) };
    msg::reply(gmst.clone(), 0).expect("Failed to share state");
}

pub fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}


pub fn program_turn_gen(difficulty: DifficultyLevel, max_per_turn: u32,remain_num:u32) -> u32 {
    match difficulty {
        DifficultyLevel::Easy => {
            let mut count = get_random_u32() % max_per_turn;
            count += 1;
            count
        }
        DifficultyLevel::Hard => {
            // 尽量让剩余的量为N+1,则用户选择任何数都会使剩余的量小于N。
            let mut count = 0;
            if remain_num - max_per_turn < max_per_turn {
                count =  remain_num - max_per_turn - 1;
                return count;
            }
            count = get_random_u32() % max_per_turn;
            if count / 2 < max_per_turn {
                count = get_random_u32() % max_per_turn;
            }
            count += 1;
            count
        }
    }
}
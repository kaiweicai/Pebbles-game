#![no_std]

use gstd::{debug, exec, msg, prelude::*};
use pebble_game_io::{DifficultyLevel, PebblesAction, PebblesEvent, PebblesInit, Player};


#[derive(Debug, Default, Encode, Decode, TypeInfo)]
pub struct PebbleGame {
    pub pebbles_count:u32,
    pub max_pebbles_per_turn:u32,
    pub pebbles_remaining:u32,
    pub program_lastmove:u32,
    pub difficulty:DifficultyLevel,
    pub first_player:Player,
    pub winner:Option<Player>,
}

impl PebbleGame {
    fn user_move(&mut self, count:u32){
        assert!(count > 0 && count <= self.pebbles_remaining, "Invalid move. You can remove between 1 and {} pebbles.", self.pebbles_remaining);
        assert!(count <= self.pebbles_remaining,"Invalid move. Not enough pebbles remaining.");

        self.pebbles_remaining -= count;
        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::User);
            msg::reply(PebblesEvent::Won(Player::User),0).expect("Failed to reply from `state()`");
            return;
        }
        self.program_move();
    }
    fn program_move(&mut self){
        let mut count:u32 = 1;
        if self.max_pebbles_per_turn!=1{
            count = program_turn_gen(self.difficulty.clone(),self.max_pebbles_per_turn);
        }
        if count > self.pebbles_remaining{
            count = self.pebbles_remaining;
        }
        self.user_move(count);

        self.pebbles_remaining -= count;
        self.program_lastmove = count;
        if self.pebbles_remaining == 0 {
            self.winner = Some(Player::Program);
            msg::reply(PebblesEvent::Won(Player::Program),0).expect("Failed to reply from `state()`");
            return;
        }
        debug!("turn count=={}", self.pebbles_count - self.pebbles_remaining);
        msg::reply(PebblesEvent::CounterTurn(count), 0).unwrap();
    }

    fn restart(&mut self,difficulty:DifficultyLevel,pebbles_count:u32,max_pebbles_per_turn:u32){
        self.difficulty = difficulty;
        self.pebbles_count = pebbles_count;
        self.max_pebbles_per_turn = max_pebbles_per_turn;
        self.pebbles_remaining = pebbles_count;
        self.program_lastmove = 0;
        self.winner = None;
        self.first_play();
    }

    fn first_play(&mut self){
        if get_random_u32()%2 == 0{
            self.first_player = Player::User;
            msg::reply(PebblesEvent::CounterTurn(0),0).expect("Failed to reply from `state()`");
        }else {
            self.first_player = Player::Program;
            self.program_move();
        }
    }

    
}
static mut PEBBLE_GAME:Option<PebbleGame> = None;


// The `init()` entry point.
#[no_mangle]
extern fn init() {
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

// The `handle()` entry point.
#[no_mangle]
extern fn handle() {
    let action: PebblesAction = msg::load().expect("Could not load Action");
    let game = unsafe { PEBBLE_GAME.as_mut().expect("State isn't initialized") };
    game.program_lastmove = 0;
    match action {
        PebblesAction::Turn(count) => {
            game.user_move(count);
        }
        PebblesAction::GiveUp => {
            game.program_move();
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

// The `state()` entry point.
#[no_mangle]
extern fn state() {
    let gmst = unsafe { PEBBLE_GAME.as_ref().expect("State isn't initialized") };
    msg::reply(gmst.clone(), 0).expect("Failed to share state");
}

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}
pub fn program_turn_gen(difficulty: DifficultyLevel, max_per_turn: u32) -> u32 {
    match difficulty {
        DifficultyLevel::Easy => {
            let mut count = get_random_u32() % max_per_turn;
            count += 1;
            count
        }
        DifficultyLevel::Hard => {
            let mut count = get_random_u32() % max_per_turn;
            if count / 2 < max_per_turn {
                count = get_random_u32() % max_per_turn;
            }
            count += 1;
            count
        }
    }
}
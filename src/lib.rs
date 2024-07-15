#![no_std]

use gstd::{exec, msg, prelude::*};

use game_io::*;

static mut STATE:Option<GameStatus> = None;

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

// The `init()` entry point.
#[no_mangle]
extern fn init() {
    let payload:PebblesInit = msg::load().expect("Failed to load Init info");
    assert!(payload.pebbles_count > payload.max_pebbles_per_turn,"Pebbles count must be greater than max pebbles per turn");
    let random_num = get_random_u32();
    let player = match random_num % 2 {
        0 => Player::User,
        1 => {
            Player::Program
        },
        _ => unreachable!(),
    };
    unsafe {
        let mut game_state = GameStatus{
            pebbles_count: payload.pebbles_count,
            max_pebbles_per_turn: payload.max_pebbles_per_turn,
            pebbles_remaining: payload.pebbles_count,
            difficult_level: Default::default(),
            first_player: player.clone(),
            winner: None,
        };
        if let Player::Program = player {
            let pebbles_to_remove = if game_state.difficult_level == DifficultLevel::Easy {
                get_random_u32() % game_state.max_pebbles_per_turn + 1
            } else {
                1
            };
            game_state.pebbles_remaining -= pebbles_to_remove;
            msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0).expect("Unable to reply");
        }
        STATE = Some(game_state);
    }
}

// The `handle()` entry point.
#[no_mangle]
extern fn handle() {
    let payload = msg::load().expect("Failed to load payload");
    // 获取链上状态
    let mut game_state = unsafe { STATE.take().expect("State isn't initialized") };
    // 根据用户动作执行相应的操作
    match payload {
        PebblesAction::Turn(count) => {
            assert!(count > 0 && count <= game_state.max_pebbles_per_turn, "Invalid pebbles count");
            game_state.pebbles_remaining = game_state.pebbles_remaining.saturating_sub(count);
            if game_state.pebbles_remaining == 0 {
                game_state.winner = Some(game_state.first_player.clone());
                msg::reply(PebblesEvent::Won(game_state.first_player.clone()), 0).expect("Failed to reply from `handle()`");
            }else {
                game_state.first_player = if game_state.first_player == Player::User { Player::Program } else { Player::User };
                msg::reply(PebblesEvent::CounterTurn(count), 0).expect("Failed to reply from `handle()`");
                //如果下一个是program，则需要根据难度生成一个数，并提交给自己进行处理。
                if let Player::Program = game_state.first_player {
                    let pebbles_to_remove = if game_state.difficult_level == DifficultLevel::Easy {
                        get_random_u32() % game_state.max_pebbles_per_turn + 1
                    } else {
                        1
                    };
                    game_state.pebbles_remaining -= pebbles_to_remove;
                    msg::reply(PebblesEvent::CounterTurn(pebbles_to_remove), 0).expect("Unable to reply");
                }
            }
            unsafe { STATE = Some(game_state) }
        }
        PebblesAction::GiveUp => {
            // 当前用户放弃自己的轮次，交给下一个用户动作
            game_state.first_player = if game_state.first_player == Player::User { Player::Program } else { Player::User };
            unsafe { STATE = Some(game_state) }

        }
        PebblesAction::Restart { difficult_level, pebbles_count, max_pebbles_per_turn } => {
            let mut state = unsafe { STATE.take().expect("State isn't initialized") };
            state.pebbles_count = pebbles_count;
            state.max_pebbles_per_turn = max_pebbles_per_turn;
            state.pebbles_remaining = pebbles_count;
            state.difficult_level = difficult_level;
            state.first_player = Player::User;
            state.winner = None;
            unsafe { STATE = Some(state) }
            msg::reply(PebblesEvent::CounterTurn(0), 0).expect("Failed to reply from `handle()`");
        }
    }
}

// The `state()` entry point.
#[no_mangle]
extern fn state() {
    let state = unsafe { STATE.take().expect("State isn't initialized") };
    msg::reply(state, 0).expect("Failed to reply from `state()`");
}

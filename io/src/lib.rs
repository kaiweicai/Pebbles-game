#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{prelude::*, ActorId};

/// The contract metadata. Used by frontend apps & for describing the types of messages that can be
/// sent in contract's entry points. See also [`Metadata`].
pub struct ContractMetadata;

impl Metadata for ContractMetadata {
    type Init = In<PebblesInit>;
    type Handle = InOut<PebblesAction, PebblesEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<GameStatus>;
}

//初始化游戏时，需要传递一些初始信息。 例如，鹅卵石的数量（ N ), 每转移除的最大卵石数 ( K ），难度级别
#[derive(Encode, Decode, TypeInfo, Debug, Default, Clone)]
pub struct PebblesInit {
    pub difficult_level: DifficultLevel,
    pub pebbles_count: u32,        // 可以移除总数N
    pub max_pebbles_per_turn: u32, // 每次移除的最大数量K
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo,PartialEq)]
pub enum DifficultLevel {
    Hard,
    #[default]
    Easy,
}

/// 它需要为每个用户的移动发送操作消息并从程序接收一些事件。
/// 该动作可以是一个回合，其中有一些卵石需要被移除或放弃。
/// 此外，还有一个重新启动操作，而不是重置游戏状态。
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesAction {
    Turn(u32),
    GiveUp,
    Restart {
        difficult_level: DifficultLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
    },
}

// 该事件反映了 用户 删除的卵石数 移动后的游戏状态：程序 或游戏结束并显示有关获胜者的信息。
#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesEvent {
    CounterTurn(u32),
    Won(Player),
}

#[derive(Debug, Decode, Encode, TypeInfo, Clone, Default,PartialEq)]
pub enum Player {
    Program,
    #[default]
    User,
}

// 内部游戏状态应保留与游戏当前状态相关的所有信息。 一些信息是在初始化期间设置的，第一个玩家是随机选择的，一些数据在游戏过程中会改变。
#[derive(Debug, Decode, Encode, TypeInfo, Clone,PartialEq)]
pub struct GameStatus {
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub pebbles_remaining: u32,
    pub difficult_level: DifficultLevel,
    pub first_player: Player, // 当前用户是用户还是程序
    pub winner: Option<Player>,
}

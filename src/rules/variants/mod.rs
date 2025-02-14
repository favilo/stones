use std::fmt::Debug;

use bevy::prelude::*;
use bevy_sequential_actions::BoxedAction;

use crate::game::{Hole, Player};

use self::kalah::HOLE_COUNT;

pub mod kalah;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Index {
    Player(Player, Hole),
    Score(Player),
}

impl Index {
    fn next(self, Player(start): Player) -> Self {
        match self {
            Index::Player(Player(p), Hole(h)) => {
                if h >= 5 {
                    if p == start {
                        Index::Score(Player(p))
                    } else {
                        Index::Player(Player::next(p), Hole(0))
                    }
                } else {
                    Index::Player(Player(p), Hole(h + 1))
                }
            }
            Index::Score(Player(p)) => Index::Player(Player::next(p), Hole(0)),
        }
    }

    fn opposite_bucket(&self) -> Option<Self> {
        match self {
            Index::Player(player, Hole(h)) => {
                // The hole that is opposite the current hole has a different index than ours.
                Some(Index::Player(
                    Player::next(*player),
                    Hole(HOLE_COUNT - h - 1),
                ))
            }
            Index::Score(_) => None,
        }
    }

    fn player(&self) -> usize {
        match self {
            Index::Player(Player(p), _) | Index::Score(Player(p)) => *p,
        }
    }

    pub fn hole(&self) -> Option<Hole> {
        match self {
            Index::Player(_, h) => Some(*h),
            Index::Score(_) => None,
        }
    }
}

pub trait Variant: Send + Sync + Debug + Reflect {
    fn hole_count(&self) -> usize;
    fn starting_pieces(&self) -> usize;

    fn bucket_position(&self, index: Index) -> Vec3;

    fn get_bucket_entities(&self, index: Index) -> &[Entity];
    fn get_bucket_entities_mut(&mut self, index: Index) -> &mut Vec<Entity>;

    fn push_entity(&mut self, index: Index, entity: Entity);

    fn perform_move(&mut self, index: Index, turn: Player) -> Vec<BoxedAction>;
}

#[derive(Debug, Resource)]
pub enum ChosenVariant {
    /// The Kalah variant. Simple, considered a childs game.
    Kalah(kalah::Kalah),
    // Oware variant.
    // Oware
}

impl Default for ChosenVariant {
    fn default() -> Self {
        Self::Kalah(kalah::Kalah::default())
    }
}

impl ChosenVariant {
    pub fn to_variant(&self) -> Box<dyn Variant> {
        match self {
            ChosenVariant::Kalah(v) => Box::new(v.clone()),
        }
    }
}

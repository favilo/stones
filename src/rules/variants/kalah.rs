use avian3d::math::Vector;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::text::cosmic_text::Action;
use bevy_sequential_actions::BoxedAction;

use super::{Index, Variant};
use crate::game::actions::turn::{MovePiece, NextPlayer, Winner};
use crate::game::actions::ui::DeclareWinner;
use crate::game::{Hole, Player, PlayerTurn, BALL_RADIUS};
use crate::PLAYER_COUNT;

pub const HOLE_COUNT: usize = 6;
pub const STARTING_PIECES: usize = 4;

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Reflect)]
pub struct Side {
    buckets: [Vec<Entity>; HOLE_COUNT],
    home: Vec<Entity>,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Reflect)]
pub struct Kalah {
    players: [Side; PLAYER_COUNT],
}

impl Variant for Kalah {
    fn hole_count(&self) -> usize {
        HOLE_COUNT
    }

    fn starting_pieces(&self) -> usize {
        STARTING_PIECES
    }

    fn bucket_position(&self, index: Index) -> Vec3 {
        match index {
            Index::Player(Player(p), Hole(h)) => {
                assert!(p < PLAYER_COUNT, "Invalid player index");
                assert!(h < HOLE_COUNT, "Invalid hole index");
                Self::HOLE_DROP_POSITIONS[p][h]
            }
            Index::Score(Player(p)) => {
                assert!(p < PLAYER_COUNT, "Invalid player index");
                Self::BUCKET_POSITIONS[p]
            }
        }
    }

    fn get_bucket_entities(&self, index: Index) -> &[Entity] {
        match index {
            Index::Player(Player(p), Hole(h)) => &self.players[p].buckets[h],
            Index::Score(Player(p)) => &self.players[p].home,
        }
    }

    fn get_bucket_entities_mut(&mut self, index: Index) -> &mut Vec<Entity> {
        match index {
            Index::Player(Player(p), Hole(h)) => &mut self.players[p].buckets[h],
            Index::Score(Player(p)) => &mut self.players[p].home,
        }
    }

    fn push_entity(&mut self, index: Index, entity: Entity) {
        self.players[index.player()].buckets[*index.hole().expect("Invalid index")].push(entity);
    }

    fn perform_move(&mut self, mut index: Index, mut player: Player) -> Vec<BoxedAction> {
        assert!(matches!(index, Index::Player(_, _)));
        let entities = std::mem::take(self.get_bucket_entities_mut(index));
        let start_player = index.player();
        let mut actions = Vec::<BoxedAction>::new();

        entities.into_iter().for_each(|stone| {
            index = index.next(Player(start_player));

            self.get_bucket_entities_mut(index).push(stone);
            actions.push(Box::new(MovePiece::new(stone, self.bucket_position(index))));
        });

        if !matches!(index, Index::Score(_)) {
            if let Some(opposite) = index.opposite_bucket() {
                // If the opposite bucket contains a stone, and the current bucket was empty, AND
                // the bucket is on the current player's side; capture the stones in both buckets.
                if index.player() == start_player
                    && self.get_bucket_entities(index).len() == 1
                    && !self.get_bucket_entities(opposite).is_empty()
                {
                    let ours = std::mem::take(self.get_bucket_entities_mut(index));
                    let theirs = std::mem::take(self.get_bucket_entities_mut(opposite));
                    tracing::info!("Captured {} stones", ours.len() + theirs.len());
                    let score_index = Index::Score(Player(start_player));
                    actions.extend(ours.iter().copied().chain(theirs.iter().copied()).map(
                        |stone| -> BoxedAction {
                            Box::new(MovePiece::new(stone, self.bucket_position(score_index)))
                        },
                    ));
                    self.get_bucket_entities_mut(score_index)
                        .extend(ours.into_iter().chain(theirs.into_iter()));
                }
            }
            player = Player::next(player);
        }

        if self
            .players
            .iter()
            .any(|side| side.buckets.iter().all(Vec::is_empty))
        {
            let winner = self
                .players
                .iter()
                .enumerate()
                .max_by_key(|(_, side)| side.home.len())
                .unwrap()
                .0;
            actions.push(Box::new(DeclareWinner::with_input(Player(winner))));
        } else {
            actions.push(Box::new(NextPlayer(player)));
        }

        actions
    }
}

impl Kalah {
    const HOLE_DROP_POSITIONS: [[Vec3; HOLE_COUNT]; PLAYER_COUNT] = [
        // Top Row
        [
            Vec3::new(-0.215, 0.075, -0.035),
            Vec3::new(-0.130, 0.075, -0.035),
            Vec3::new(-0.040, 0.075, -0.035),
            Vec3::new(00.042, 0.075, -0.035),
            Vec3::new(00.130, 0.075, -0.035),
            Vec3::new(00.215, 0.075, -0.035),
        ],
        // Bottom Row
        [
            Vec3::new(00.215, 0.075, 0.035),
            Vec3::new(00.130, 0.075, 0.035),
            Vec3::new(00.042, 0.075, 0.035),
            Vec3::new(-0.040, 0.075, 0.035),
            Vec3::new(-0.130, 0.075, 0.035),
            Vec3::new(-0.215, 0.075, 0.035),
        ],
    ];
    const BUCKET_POSITIONS: [Vec3; PLAYER_COUNT] =
        [Vec3::new(0.276, 0.075, 0.0), Vec3::new(-0.276, 0.075, 0.0)];
}

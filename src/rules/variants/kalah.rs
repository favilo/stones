use avian3d::math::Vector;
use avian3d::prelude::*;
use bevy::prelude::*;

use super::{Index, Variant};
use crate::game::{Hole, Player, PlayerTurn, ToSleep, Winner};
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

    fn perform_move(
        &mut self,
        mut index: Index,
        query: &mut Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
        turn: &mut ResMut<PlayerTurn>,
        to_sleep: &mut Option<ResMut<ToSleep>>,
        commands: &mut Commands,
    ) {
        assert!(matches!(index, Index::Player(_, _)));
        let entities =
            std::mem::take(&mut self.players[index.player()].buckets[*index.hole().unwrap()]);
        let start_player = index.player();
        if let Some(t) = to_sleep.as_mut() {
            t.reset();
        }
        entities.into_iter().for_each(|stone| {
            let mut e = commands.entity(stone);
            e.remove::<Sleeping>();
            index = index.next(Player(start_player));

            self.get_bucket_entities_mut(index).push(stone);
            let (mut transform, mut linear_velocity, mut angular_velocity) =
                query.get_mut(stone).unwrap();
            transform.translation = self.bucket_position(index);
            transform.rotation = Quat::from_rotation_x(90.0);
            **linear_velocity = Vector::ZERO;
            **angular_velocity = Vector::ZERO;
        });

        if !matches!(index, Index::Score(_)) {
            turn.0 = (turn.0 + 1) % 2;
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
            commands.trigger(Winner(winner));
        }
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

use crate::board::CORNERS_AND_CENTER;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PieceType {
    Defender,
    Attacker,
    King,
    #[default]
    None,
    Wall,
    Castle,
}

impl PieceType {
    pub fn is_enemy(&self, other: PieceType) -> bool {
        match self {
            PieceType::None | PieceType::Wall => false,
            PieceType::Attacker => matches!(other, PieceType::Defender | PieceType::King),
            PieceType::Defender | PieceType::King => matches!(other, PieceType::Attacker),
            PieceType::Castle => true,
        }
    }

    pub fn is_friendly(&self, other: PieceType) -> bool {
        !self.is_enemy(other)
    }
}

impl Piece {
    pub fn to_piecetype(self) -> PieceType {
        if self.is_king {
            PieceType::King
        } else if self.player == Player::Defender {
            PieceType::Defender
        } else {
            PieceType::Attacker
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Defender,
    Attacker,
}

#[derive(Clone, Copy, Component, Debug)]
pub struct Piece {
    pub player: Player,
    // Current position
    pub x: i8,
    pub y: i8,
    pub is_king: bool,
}
impl Piece {
    /// Returns the possible_positions that are available
    pub fn is_move_valid(&self, new_position: (i8, i8), pieces: Vec<Piece>) -> bool {
        is_path_empty((self.x, self.y), new_position, &pieces)
            && ((self.x == new_position.0 && self.y != new_position.1)
                || (self.y == new_position.1 && self.x != new_position.0))
            && (self.is_king || !CORNERS_AND_CENTER.contains(&new_position))
    }
}

fn is_path_empty(begin: (i8, i8), end: (i8, i8), pieces: &Vec<Piece>) -> bool {
    // Same column
    if begin.0 == end.0 {
        for piece in pieces {
            if piece.x == begin.0
                && ((piece.y > begin.1 && piece.y <= end.1)
                    || (piece.y >= end.1 && piece.y < begin.1))
            {
                return false;
            }
        }
    }
    // Same row
    if begin.1 == end.1 {
        for piece in pieces {
            if piece.y == begin.1
                && ((piece.x > begin.0 && piece.x <= end.0)
                    || (piece.x >= end.0 && piece.x < begin.0))
            {
                return false;
            }
        }
    }

    true
}

#[derive(Default, Resource)]
pub struct Moving(pub bool);

pub fn is_moving(moving: Res<Moving>) -> ShouldRun {
    if moving.0 {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

fn move_pieces(
    time_opt: Option<Res<Time>>,
    mut moving: ResMut<Moving>,
    mut query: Query<(&mut Transform, &Piece)>,
) {
    moving.0 = false;

    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        // Only move if the piece isn't already there (distance is big)
        if direction.length() > 0.1 {
            if let Some(time) = &time_opt {
                transform.translation += direction.normalize() * time.delta_seconds();
            } else {
                transform.translation += direction.normalize() * 0.001;
            }
            moving.0 = true;
        }
    }
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let mut king_handle: Option<Handle<Mesh>> = None;
    let mut king_cross_handle: Option<Handle<Mesh>> = None;
    let mut pawn_handle: Option<Handle<Mesh>> = None;

    let mut white_material = None;
    let mut black_material = None;

    if !cfg!(test) {
        // Load all the meshes
        king_handle = Some(
            asset_server
                .as_ref()
                .unwrap()
                .load("models/chess_kit/pieces.glb#Mesh0/Primitive0"),
        );
        king_cross_handle = Some(
            asset_server
                .as_ref()
                .unwrap()
                .load("models/chess_kit/pieces.glb#Mesh1/Primitive0"),
        );
        pawn_handle = Some(
            asset_server
                .as_ref()
                .unwrap()
                .load("models/chess_kit/pieces.glb#Mesh2/Primitive0"),
        );

        // Add some materials
        white_material = Some(
            materials
                .as_mut()
                .unwrap()
                .add(Color::rgb(1., 0.8, 0.8).into()),
        );
        black_material = Some(
            materials
                .as_mut()
                .unwrap()
                .add(Color::rgb(0.3, 0.3, 0.3).into()),
        );
    }

    spawn_king(
        &mut commands,
        white_material.clone(),
        king_handle,
        king_cross_handle,
        (5, 5),
    );

    // White pawns aka defenders
    for x in 4..7 {
        for y in 4..7 {
            if x == 5 && y == 5 {
                continue;
            }
            spawn_defender(
                &mut commands,
                white_material.clone(),
                pawn_handle.clone(),
                (x, y),
            );
        }
    }

    for location in &[(5, 3), (5, 7), (7, 5), (3, 5)] {
        spawn_defender(
            &mut commands,
            white_material.clone(),
            pawn_handle.clone(),
            *location,
        );
    }

    // Black pawns aka attackers
    for location in &[(5, 9), (5, 1), (9, 5), (1, 5)] {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            *location,
        );
    }

    for i in 3..8 {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            (0, i),
        );
    }

    for i in 3..8 {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            (10, i),
        );
    }

    for i in 3..8 {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            (i, 0),
        );
    }

    for i in 3..8 {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            (i, 10),
        );
    }
}

fn spawn_king(
    commands: &mut Commands,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    mesh_cross: Option<Handle<Mesh>>,
    position: (i8, i8),
) {
    let mut bindings = commands
        // Spawn parent entity
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        });
    bindings.insert(Piece {
        player: Player::Defender,
        x: position.0,
        y: position.1,
        is_king: true,
    });
    if !cfg!(test) {
        // Add children to the parent
        bindings.with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh.unwrap(),
                material: material.as_ref().unwrap().clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
            parent.spawn(PbrBundle {
                mesh: mesh_cross.unwrap(),
                material: material.unwrap(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
        });
    }
}

fn spawn_attacker(
    commands: &mut Commands,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    position: (i8, i8),
) {
    spawn_pawn(commands, material, Player::Attacker, mesh, position);
}

fn spawn_defender(
    commands: &mut Commands,
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Handle<Mesh>>,
    position: (i8, i8),
) {
    spawn_pawn(commands, material, Player::Defender, mesh, position);
}

fn spawn_pawn(
    commands: &mut Commands,
    material: Option<Handle<StandardMaterial>>,
    player: Player,
    mesh: Option<Handle<Mesh>>,
    position: (i8, i8),
) {
    let mut binding = commands
        // Spawn parent entity
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        });

    binding.insert(Piece {
        player,
        x: position.0,
        y: position.1,
        is_king: false,
    });
    if !cfg!(test) {
        binding.with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh.unwrap(),
                material: material.unwrap(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 2.6));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..Default::default()
            });
        });
    }
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces)
            .init_resource::<Moving>()
            .add_system(move_pieces);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn is_enemy_test() {
        let truth_table = [
            (PieceType::Attacker, PieceType::Defender, true),
            (PieceType::Attacker, PieceType::Attacker, false),
            (PieceType::Defender, PieceType::Defender, false),
            (PieceType::King, PieceType::Defender, false),
            (PieceType::Attacker, PieceType::King, true),
            (PieceType::Attacker, PieceType::None, false),
            (PieceType::Defender, PieceType::None, false),
            (PieceType::King, PieceType::None, false),
            (PieceType::Attacker, PieceType::Wall, false),
            (PieceType::Defender, PieceType::Wall, false),
            (PieceType::King, PieceType::Wall, false),
        ];

        for entry in truth_table {
            assert_eq!(entry.0.is_enemy(entry.1), entry.2);
            assert_eq!(entry.1.is_enemy(entry.0), entry.2);
        }
    }
}

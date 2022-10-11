use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Player {
    Defender,
    Attacker,
}

#[derive(Clone, Copy, Component)]
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
            && (self.is_king || new_position != (5, 5))
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

#[derive(Default)]
pub struct Moving(pub bool);

pub fn is_moving(moving: Res<Moving>) -> ShouldRun {
    if moving.0 {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

#[derive(Default)]
pub struct LastDestination {
    pub x: i8,
    pub y: i8,
}

fn move_pieces(
    time: Res<Time>,
    mut moving: ResMut<Moving>,
    mut last_dest: ResMut<LastDestination>,
    mut query: Query<(&mut Transform, &Piece)>,
) {
    moving.0 = false;
    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        // Only move if the piece isn't already there (distance is big)
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
            moving.0 = true;
            last_dest.x = piece.x;
            last_dest.y = piece.y;
        }
    }
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load all the meshes
    let king_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh0/Primitive0");
    let king_cross_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh1/Primitive0");
    let pawn_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh2/Primitive0");
    // let knight_1_handle: Handle<Mesh> =
    //     asset_server.load("models/chess_kit/pieces.glb#Mesh3/Primitive0");
    // let knight_2_handle: Handle<Mesh> =
    //     asset_server.load("models/chess_kit/pieces.glb#Mesh4/Primitive0");
    // let rook_handle: Handle<Mesh> =
    //     asset_server.load("models/chess_kit/pieces.glb#Mesh5/Primitive0");
    // let bishop_handle: Handle<Mesh> =
    //     asset_server.load("models/chess_kit/pieces.glb#Mesh6/Primitive0");
    // let queen_handle: Handle<Mesh> =
    //     asset_server.load("models/chess_kit/pieces.glb#Mesh7/Primitive0");

    // Add some materials
    let white_material = materials.add(Color::rgb(1., 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0.3, 0.3, 0.3).into());

    spawn_king(
        &mut commands,
        white_material.clone(),
        king_handle.clone(),
        king_cross_handle.clone(),
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

    for location in vec![(5, 3), (5, 7), (7, 5), (3, 5)] {
        spawn_defender(
            &mut commands,
            white_material.clone(),
            pawn_handle.clone(),
            location,
        );
    }

    // Black pawns aka attackers
    for location in vec![(5, 9), (5, 1), (9, 5), (1, 5)] {
        spawn_attacker(
            &mut commands,
            black_material.clone(),
            pawn_handle.clone(),
            location,
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
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    mesh_cross: Handle<Mesh>,
    position: (i8, i8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            player: Player::Defender,
            x: position.0,
            y: position.1,
            is_king: true,
        })
        // Add children to the parent
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
            parent.spawn_bundle(PbrBundle {
                mesh: mesh_cross,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_attacker(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: (i8, i8),
) {
    spawn_pawn(commands, material, Player::Attacker, mesh, position);
}

fn spawn_defender(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    position: (i8, i8),
) {
    spawn_pawn(commands, material, Player::Defender, mesh, position);
}

fn spawn_pawn(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    player: Player,
    mesh: Handle<Mesh>,
    position: (i8, i8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            player,
            x: position.0,
            y: position.1,
            is_king: false,
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 2.6));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces)
            .init_resource::<Moving>()
            .init_resource::<LastDestination>()
            .add_system(move_pieces);
        //.insert_resource
    }
}

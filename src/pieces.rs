use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    // Queen,
    // Bishop,
    // Knight,
    // Rook,
    Pawn,
}

#[derive(Clone, Copy, Component)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    // Current position
    pub x: u8,
    pub y: u8,
}
impl Piece {
    /// Returns the possible_positions that are available
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: Vec<Piece>) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if color_of_square(new_position, &pieces) == Some(self.color) {
            return false;
        }

        is_path_empty((self.x, self.y), new_position, &pieces)
            && ((self.x == new_position.0 && self.y != new_position.1)
                || (self.y == new_position.1 && self.x != new_position.0))
            && (self.piece_type == PieceType::King || new_position != (5, 5))
    }
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> bool {
    // Same column
    if begin.0 == end.0 {
        for piece in pieces {
            if piece.x == begin.0
                && ((piece.y > begin.1 && piece.y < end.1)
                    || (piece.y > end.1 && piece.y < begin.1))
            {
                return false;
            }
        }
    }
    // Same row
    if begin.1 == end.1 {
        for piece in pieces {
            if piece.y == begin.1
                && ((piece.x > begin.0 && piece.x < end.0)
                    || (piece.x > end.0 && piece.x < begin.0))
            {
                return false;
            }
        }
    }

    true
}

/// Returns None if square is empty, returns a Some with the color if not
fn color_of_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<PieceColor> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        // Only move if the piece isn't already there (distance is big)
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
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

    // spawn_rook(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     rook_handle.clone(),
    //     (0, 0),
    // );
    // spawn_knight(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     knight_1_handle.clone(),
    //     knight_2_handle.clone(),
    //     (0, 1),
    // );
    // spawn_bishop(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     bishop_handle.clone(),
    //     (0, 2),
    // );
    // spawn_queen(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     queen_handle.clone(),
    //     (0, 3),
    // );
    spawn_king(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        king_handle.clone(),
        king_cross_handle.clone(),
        (5, 5),
    );
    // spawn_bishop(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     bishop_handle.clone(),
    //     (0, 5),
    // );
    // spawn_knight(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     knight_1_handle.clone(),
    //     knight_2_handle.clone(),
    //     (0, 6),
    // );
    // spawn_rook(
    //     &mut commands,
    //     white_material.clone(),
    //     PieceColor::White,
    //     rook_handle.clone(),
    //     (0, 7),
    // );

    // for i in 0..8 {
    //     spawn_pawn(
    //         &mut commands,
    //         white_material.clone(),
    //         PieceColor::White,
    //         pawn_handle.clone(),
    //         (1, i),
    //     );
    // }

    // White pawns
    for x in 4..7 {
        for y in 4..7 {
            if x == 5 && y == 5 {
                continue;
            }
            spawn_pawn(
                &mut commands,
                white_material.clone(),
                PieceColor::White,
                pawn_handle.clone(),
                (x, y),
            );
        }
    }

    for location in vec![(5, 3), (5, 7), (7, 5), (3, 5)] {
        spawn_pawn(
            &mut commands,
            white_material.clone(),
            PieceColor::White,
            pawn_handle.clone(),
            location,
        );
    }

    // Black pawns
    for location in vec![(5, 9), (5, 1), (9, 5), (1, 5)] {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            location,
        );
    }

    for i in 3..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (0, i),
        );
    }

    for i in 3..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (10, i),
        );
    }

    for i in 3..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (i, 0),
        );
    }

    for i in 3..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (i, 10),
        );
    }
}

fn spawn_king(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    mesh_cross: Handle<Mesh>,
    position: (u8, u8),
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
            color: piece_color,
            piece_type: PieceType::King,
            x: position.0,
            y: position.1,
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

// fn spawn_knight(
//     commands: &mut Commands,
//     material: Handle<StandardMaterial>,
//     piece_color: PieceColor,
//     mesh_1: Handle<Mesh>,
//     mesh_2: Handle<Mesh>,
//     position: (u8, u8),
// ) {
//     commands
//         // Spawn parent entity
//         .spawn_bundle(PbrBundle {
//             transform: Transform::from_translation(Vec3::new(
//                 position.0 as f32,
//                 0.,
//                 position.1 as f32,
//             )),
//             ..Default::default()
//         })
//         .insert(Piece {
//             color: piece_color,
//             piece_type: PieceType::Knight,
//             x: position.0,
//             y: position.1,
//         })
//         // Add children to the parent
//         .with_children(|parent| {
//             parent.spawn_bundle(PbrBundle {
//                 mesh: mesh_1,
//                 material: material.clone(),
//                 transform: {
//                     let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
//                     transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
//                     transform
//                 },
//                 ..Default::default()
//             });
//             parent.spawn_bundle(PbrBundle {
//                 mesh: mesh_2,
//                 material,
//                 transform: {
//                     let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
//                     transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
//                     transform
//                 },
//                 ..Default::default()
//             });
//         });
// }

// fn spawn_queen(
//     commands: &mut Commands,
//     material: Handle<StandardMaterial>,
//     piece_color: PieceColor,
//     mesh: Handle<Mesh>,
//     position: (u8, u8),
// ) {
//     commands
//         // Spawn parent entity
//         .spawn_bundle(PbrBundle {
//             transform: Transform::from_translation(Vec3::new(
//                 position.0 as f32,
//                 0.,
//                 position.1 as f32,
//             )),
//             ..Default::default()
//         })
//         .insert(Piece {
//             color: piece_color,
//             piece_type: PieceType::Queen,
//             x: position.0,
//             y: position.1,
//         })
//         .with_children(|parent| {
//             parent.spawn_bundle(PbrBundle {
//                 mesh,
//                 material,
//                 transform: {
//                     let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -0.95));
//                     transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
//                     transform
//                 },
//                 ..Default::default()
//             });
//         });
// }

// fn spawn_bishop(
//     commands: &mut Commands,
//     material: Handle<StandardMaterial>,
//     piece_color: PieceColor,
//     mesh: Handle<Mesh>,
//     position: (u8, u8),
// ) {
//     commands
//         // Spawn parent entity
//         .spawn_bundle(PbrBundle {
//             transform: Transform::from_translation(Vec3::new(
//                 position.0 as f32,
//                 0.,
//                 position.1 as f32,
//             )),
//             ..Default::default()
//         })
//         .insert(Piece {
//             color: piece_color,
//             piece_type: PieceType::Bishop,
//             x: position.0,
//             y: position.1,
//         })
//         .with_children(|parent| {
//             parent.spawn_bundle(PbrBundle {
//                 mesh,
//                 material,
//                 transform: {
//                     let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 0.));
//                     transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
//                     transform
//                 },
//                 ..Default::default()
//             });
//         });
// }

// fn spawn_rook(
//     commands: &mut Commands,
//     material: Handle<StandardMaterial>,
//     piece_color: PieceColor,
//     mesh: Handle<Mesh>,
//     position: (u8, u8),
// ) {
//     commands
//         // Spawn parent entity
//         .spawn_bundle(PbrBundle {
//             transform: Transform::from_translation(Vec3::new(
//                 position.0 as f32,
//                 0.,
//                 position.1 as f32,
//             )),
//             ..Default::default()
//         })
//         .insert(Piece {
//             color: piece_color,
//             piece_type: PieceType::Rook,
//             x: position.0,
//             y: position.1,
//         })
//         .with_children(|parent| {
//             parent.spawn_bundle(PbrBundle {
//                 mesh,
//                 material,
//                 transform: {
//                     let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 1.8));
//                     transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
//                     transform
//                 },
//                 ..Default::default()
//             });
//         });
// }

fn spawn_pawn(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
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
            color: piece_color,
            piece_type: PieceType::Pawn,
            x: position.0,
            y: position.1,
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
            .add_system(move_pieces);
    }
}

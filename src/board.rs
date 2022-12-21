use crate::minimap::*;
use crate::pieces::*;
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

#[derive(Component)]
pub struct Square {
    pub x: i8,
    pub y: i8,
}
impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

pub const CORNERS: [(i8, i8); 4] = [(0, 0), (10, 10), (10, 0), (0, 10)];
pub const CORNERS_AND_CENTER: [(i8, i8); 5] = [(0, 0), (10, 10), (10, 0), (0, 10), (5, 5)];

fn create_board(
    mut commands: Commands,
    meshes: Option<ResMut<Assets<Mesh>>>,
    materials: Option<Res<SquareMaterials>>,
) {
    let mesh = meshes.map(|mut m| m.add(Mesh::from(shape::Plane { size: 1. })));

    // Spawn 64 squares
    for i in 0..11 {
        for j in 0..11 {
            let square = Square { x: i, y: j };

            if cfg!(test) {
                commands
                    .spawn(PickableBundle {
                        ..Default::default()
                    })
                    .insert(square);
            } else {
                let material = if CORNERS_AND_CENTER.contains(&(i, j)) {
                    materials.as_ref().unwrap().red_color.clone()
                } else if square.is_white() {
                    materials.as_ref().unwrap().white_color.clone()
                } else {
                    materials.as_ref().unwrap().black_color.clone()
                };

                commands
                    .spawn(PbrBundle {
                        mesh: mesh.as_ref().unwrap().clone(),
                        // Change material according to position to get alternating pattern
                        material,
                        transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                        ..Default::default()
                    })
                    .insert(PickableBundle {
                        ..Default::default()
                    })
                    .insert(Square { x: i, y: j });
            }
        }
    }
}

fn color_squares(
    selected_square: Res<SelectedSquare>,
    materials: Res<SquareMaterials>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>)>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Get entity under the cursor, if there is one
    let top_entity = match picking_camera_query.iter().last() {
        Some(picking_camera) => picking_camera
            .intersections()
            .last()
            .map(|(entity, _intersection)| entity),
        None => None,
    };

    for (entity, square, mut material) in query.iter_mut() {
        // Change the material
        *material = if Some(entity) == top_entity.copied() {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_square.entity {
            materials.selected_color.clone()
        } else if CORNERS_AND_CENTER.contains(&(square.x, square.y)) {
            materials.red_color.clone()
        } else if square.is_white() {
            materials.white_color.clone()
        } else {
            materials.black_color.clone()
        };
    }
}

#[derive(Resource)]
struct SquareMaterials {
    highlight_color: Handle<StandardMaterial>,
    selected_color: Handle<StandardMaterial>,
    black_color: Handle<StandardMaterial>,
    white_color: Handle<StandardMaterial>,
    red_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        SquareMaterials {
            highlight_color: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
            red_color: materials.add(Color::rgb(1., 0., 0.).into()),
        }
    }
}

#[derive(Default, Resource)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}
#[derive(Default, Resource)]
struct SelectedPiece {
    entity: Option<Entity>,
}

#[derive(Resource)]
pub struct PlayerTurn(pub Player);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(Player::Attacker)
    }
}
impl PlayerTurn {
    fn change(&mut self) {
        self.0 = match self.0 {
            Player::Defender => Player::Attacker,
            Player::Attacker => Player::Defender,
        }
    }
}

fn select_square(
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    squares_query: Query<&Square>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the square under the cursor and set it as the selected
    if let Some(picking_camera) = picking_camera_query.iter().last() {
        if let Some((square_entity, _intersection)) = picking_camera.intersections().iter().last() {
            if let Ok(_square) = squares_query.get(*square_entity) {
                // Mark it as selected
                selected_square.entity = Some(*square_entity);
            }
        } else {
            // Player clicked outside the board, deselect everything
            selected_square.entity = None;
            selected_piece.entity = None;
        }
    }
}

fn select_piece(
    selected_square: Res<SelectedSquare>,
    moving: Res<Moving>,
    mut selected_piece: ResMut<SelectedPiece>,
    turn: Res<PlayerTurn>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
) {
    if !selected_square.is_changed() || moving.0 {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    if selected_piece.entity.is_none() {
        // Select the piece in the currently selected square
        for (piece_entity, piece) in pieces_query.iter() {
            if piece.x == square.x && piece.y == square.y && turn.0 == piece.player {
                // piece_entity is now the entity in the same square
                selected_piece.entity = Some(piece_entity);
                break;
            }
        }
    }
}

#[derive(Default, Debug, Resource)]
pub struct LastDestination {
    pub x: i8,
    pub y: i8,
}

fn move_piece(
    selected_square: Res<SelectedSquare>,
    selected_piece: Res<SelectedPiece>,
    mut last_dest: ResMut<LastDestination>,
    mut turn: ResMut<PlayerTurn>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
    mut reset_selected_event: EventWriter<ResetSelectedEvent>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    if let Some(selected_piece_entity) = selected_piece.entity {
        let pieces_vec = pieces_query.iter_mut().map(|(_, piece)| *piece).collect();
        // Move the selected piece to the selected square
        let mut piece =
            if let Ok((_piece_entity, piece)) = pieces_query.get_mut(selected_piece_entity) {
                piece
            } else {
                return;
            };

        if piece.is_move_valid((square.x, square.y), pieces_vec) {
            // Move piece
            piece.x = square.x;
            piece.y = square.y;

            last_dest.x = piece.x;
            last_dest.y = piece.y;

            // Change turn
            turn.change();
        }

        reset_selected_event.send(ResetSelectedEvent);
    }
}

struct ResetSelectedEvent;

fn reset_selected(
    mut event_reader: EventReader<ResetSelectedEvent>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
) {
    for _event in event_reader.iter() {
        selected_square.entity = None;
        selected_piece.entity = None;
    }
}

fn check_killing(
    mut commands: Commands,
    last_dest: Res<LastDestination>,
    query: Query<(Entity, &Piece), Without<Taken>>,
) {
    if last_dest.is_changed() {
        let map = MiniMap::from_query(&query);

        // Get list of locations where pieces are killed
        let killings = map.detect_killings((last_dest.x, last_dest.y));

        // Then iter each piece, and see if it is killed
        for kill in &killings {
            for (entity, piece) in query.iter() {
                if kill.0 == piece.x && kill.1 == piece.y {
                    println!("Killing {:?}, not {:?}", kill, last_dest);
                    commands.entity(entity).insert(Taken);
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct GameOver(Option<Player>);

fn check_victory(
    query: Query<(Entity, &Piece), Without<Taken>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut gameover: ResMut<GameOver>,
    moving: Res<Moving>,
) {
    if gameover.0.is_some() || moving.0 {
        return;
    }
    for (_entity, piece) in query.iter() {
        if piece.is_king && CORNERS.contains(&(piece.x, piece.y)) {
            println!("Defender win!");
            app_exit_events.send(AppExit);
            gameover.0 = Some(Player::Defender);
            break;
        }
    }
}

#[derive(Component)]
pub struct Taken;

fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    mut gameover: ResMut<GameOver>,
    moving: Res<Moving>,
    query: Query<(Entity, &Piece), With<Taken>>,
) {
    if moving.0 {
        return;
    }
    for (entity, piece) in query.iter() {
        // If the king is taken, we should exit
        if piece.is_king {
            println!("Attackers win!");
            gameover.0 = Some(Player::Attacker);
            app_exit_events.send(AppExit);
        }

        // Despawn piece and children
        commands.entity(entity).despawn_recursive();
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<PlayerTurn>()
            .init_resource::<LastDestination>()
            .init_resource::<GameOver>()
            .add_event::<ResetSelectedEvent>()
            .add_startup_system(create_board);
        if !cfg!(test) {
            app.init_resource::<SquareMaterials>().add_system_set(
                SystemSet::new()
                    .with_run_criteria(is_moving)
                    .label("select_square")
                    .with_system(color_squares)
                    .with_system(select_square),
            );
        }
        app.add_system(
            // move_piece needs to run before select_piece
            move_piece.after("select_square").before("select_piece"),
        )
        .add_system(select_piece.after("select_square").label("select_piece"))
        .add_system(
            check_killing
                .after("select_piece")
                .before(despawn_taken_pieces),
        )
        .add_system(despawn_taken_pieces)
        .add_system(reset_selected.after("select_square"))
        .add_system(check_victory);
    }
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use bevy::app::App;

    pub fn force_move_piece(
        app: &mut App,
        player: Player,
        piece_loc: (i8, i8),
        target_loc: (i8, i8),
    ) {
        // Select the 'piece'
        let square_entity_old = app
            .world
            .query::<(Entity, &Square)>()
            .iter(&app.world)
            .find(|p| p.1.x == piece_loc.0 && p.1.y == piece_loc.1)
            .unwrap()
            .0;

        // Set the piece as the 'selected_square'
        let mut selected_square = app.world.resource_mut::<SelectedSquare>();
        selected_square.entity = Some(square_entity_old);

        // Update twice to make sure we are moving
        app.update();

        let selected_piece = app
            .world
            .resource::<SelectedPiece>()
            .entity
            .unwrap_or_else(|| {
                panic!(
                    "No piece is selected! Probably incorrect player!\n{:?}->{:?}\n\n{:?}",
                    piece_loc,
                    target_loc,
                    MiniMap::from_app(app)
                )
            });

        assert_eq!(
            app.world.get::<Piece>(selected_piece).unwrap().player,
            player,
            "Selected piece is not of the correct player! Should never happen!"
        );

        app.update();

        // Wait until the moving is done
        loop {
            let moving = app.world.resource::<Moving>();
            if moving.0 == false {
                break;
            }
            app.update();
        }
        // app.update(); remove in 2023

        // Select the destination square
        let square_entity_new = app
            .world
            .query::<(Entity, &Square)>()
            .iter(&app.world)
            .find(|p| p.1.x == target_loc.0 && p.1.y == target_loc.1)
            .unwrap()
            .0;

        // Set the square as selected_square
        let mut selected_square = app.world.resource_mut::<SelectedSquare>();
        selected_square.entity = Some(square_entity_new);

        // Update twice to make sure we are moving
        app.update();
        app.update();

        // Wait until the moving is done
        loop {
            let moving = app.world.resource::<Moving>();
            if moving.0 == false {
                break;
            }
            app.update();
        }
        app.update();
    }

    pub fn skip_turn(app: &mut App, player: Player) {
        let mut player_turn = app.world.get_resource_mut::<PlayerTurn>().unwrap();

        let active_player = player_turn.0;

        // First change the player turn, so we can free the 'app' var
        player_turn.change();

        // Check if the player was the one we expected
        assert_eq!(
            active_player,
            player,
            "\nError while skipping turns, incorrect player!\n{:?}",
            MiniMap::from_app(app)
        );
    }

    pub fn expect_game_over(app: &mut App, player: Player) {
        let gameover = app.world.get_resource::<GameOver>().unwrap();

        assert!(
            gameover.0.is_some(),
            "\nGame not over yet!\n{:?}",
            MiniMap::from_app(app)
        );

        let winning_player = gameover.0.unwrap();
        drop(gameover);

        // Check if the player was the one we expected
        assert_eq!(
            winning_player,
            player,
            "\nError while assuming game over, incorrect player!\n{:?}",
            MiniMap::from_app(app)
        );
    }
}

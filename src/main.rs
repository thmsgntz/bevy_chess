use bevy::prelude::*;
use bevy_mod_picking::*;

mod pieces;
use pieces::*;
mod board;
use board::*;
mod ui;
use ui::*;
mod minimap;

fn main() {
    App::new()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Chess!".to_string(),
            width: 600.,
            height: 600.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecesPlugin)
        .add_plugin(UIPlugin)
        .add_startup_system(setup)
        .add_system(keyboard_input_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        // Camera
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-7.0, 20.0, 4.0),
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default())
        // Light
        .commands()
        .spawn_bundle(DirectionalLightBundle {
            transform: Transform {
                translation: Vec3::new(4.0, 8.0, 4.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                ..default()
            },

            ..Default::default()
        });
}

/// This system prints 'A' key state
fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let mut camera_transform = camera_query.single_mut();

    let mut velocity = Vec3::ZERO;
    let mut rotate = 0.0;
    let mut home = false;
    let local_z = camera_transform.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);

    for key in keyboard_input.get_pressed() {
        match key {
            KeyCode::W => velocity += forward,
            KeyCode::S => velocity -= forward,
            KeyCode::A => velocity -= right,
            KeyCode::D => velocity += right,
            KeyCode::Q => rotate += 1.0,
            KeyCode::E => rotate -= 1.0,
            KeyCode::H => home = true,
            KeyCode::Space => velocity += Vec3::Y,
            KeyCode::LShift => velocity -= Vec3::Y,
            _ => (),
        }
    }

    if home {
        let home = Transform::from_matrix(Mat4::from_rotation_translation(
            Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
            Vec3::new(-7.0, 20.0, 4.0),
        ));
        *camera_transform = home;
    } else {
        velocity = velocity.normalize_or_zero();

        camera_transform.translation += velocity * time.delta_seconds() * 10.0;
        camera_transform.rotate_y(rotate * time.delta_seconds());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::test_helpers::*;
    use crate::minimap::MiniMap;
    use crate::Player::{Attacker, Defender};

    fn expect_n_pieces(app: &mut App, n: usize) {
        assert_eq!(
            app.world.query::<&Piece>().iter(&app.world).len(),
            n,
            "{:?}",
            MiniMap::from_app(app)
        );
    }

    #[test]
    fn spawn_board() {
        // Test if we spawn a decent board of 37 pieces
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);
    }

    #[test]
    fn skip_turn_happy() {
        // Test if the skip_turn function works as expected
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        skip_turn(&mut app, Defender);
        skip_turn(&mut app, Attacker);
        skip_turn(&mut app, Defender);
    }

    #[test]
    #[should_panic]
    fn skip_turn_illegal() {
        // Test if the skip_turn function fails if we accidentally skip the same players turn twice
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        skip_turn(&mut app, Defender);
        skip_turn(&mut app, Attacker);
        skip_turn(&mut app, Attacker);
    }

    #[test]
    fn simple_kill() {
        // Test if the simple kill mechanism works
        // A simple kill is done by surrounding a piece on 2 sides
        // Note we kill both a defender and an attacker
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        // First setup the board to kill an attacker
        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (7, 0), (7, 2));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));

        // Killed an attacker
        expect_n_pieces(&mut app, 36);

        // Then try to kill a defender
        force_move_piece(&mut app, Attacker, (7, 2), (6, 2));
        // Killed an defender
        expect_n_pieces(&mut app, 35);
    }

    #[test]
    fn simple_kill_but_not_the_mover() {
        // Test that a piece that lands between two enemies is not killed
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (7, 0), (7, 1));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));

        expect_n_pieces(&mut app, 36);
    }

    #[test]
    fn simple_kill_but_not_the_mover_in_the_turn_after() {
        // Test that a piece that lands between two enemies is not killed
        // Also not after an extra turn
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 2));
        force_move_piece(&mut app, Attacker, (3, 0), (3, 1));
        force_move_piece(&mut app, Defender, (4, 2), (4, 1));
        force_move_piece(&mut app, Attacker, (7, 0), (7, 1));

        expect_n_pieces(&mut app, 37);
    }

    #[test]
    fn simple_kill_but_not_from_a_bystander() {
        // Test that piece that lands between two enemies is not killed
        // Also not after a third enemy lands next to it
        // (The only legal kill is if an ememy lands opposite an existing enemy)
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 2));
        force_move_piece(&mut app, Attacker, (3, 0), (3, 1));
        force_move_piece(&mut app, Defender, (4, 2), (4, 1));
        force_move_piece(&mut app, Attacker, (4, 0), (3, 0));
        skip_turn(&mut app, Defender);
        force_move_piece(&mut app, Attacker, (3, 0), (4, 0));

        expect_n_pieces(&mut app, 37);
    }

    #[test]
    fn multi_kill1() {
        /*
        Test the multi kill mechanic
        Create a cluster of attackers at the bottom, defenders surrounds them
        */
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (3, 0), (0, 0));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));
        // First kill, collateral damage ;)
        expect_n_pieces(&mut app, 36);
        force_move_piece(&mut app, Attacker, (7, 0), (10, 0));
        force_move_piece(&mut app, Defender, (3, 5), (3, 0));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (7, 5), (7, 0));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (5, 3), (5, 1));

        expect_n_pieces(&mut app, 33);
    }

    #[test]
    fn multi_kill2() {
        /*
        Similar to multi_kill1, only cluster at the bottom is chopped in half.
        */

        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (3, 0), (0, 0));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));
        // First kill, collateral damage ;)
        expect_n_pieces(&mut app, 36);
        force_move_piece(&mut app, Attacker, (5, 0), (5, 1));
        force_move_piece(&mut app, Defender, (6, 1), (6, 2));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 2), (6, 1));
        // First second kill, more collateral damage ;)
        expect_n_pieces(&mut app, 35);

        force_move_piece(&mut app, Attacker, (7, 0), (10, 0));
        force_move_piece(&mut app, Defender, (3, 5), (3, 0));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (7, 5), (7, 0));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (5, 3), (5, 0));

        expect_n_pieces(&mut app, 33);
    }

    #[test]
    fn multi_kill3() {
        /*
        Similar to multi_kill2, only the 2 clusters are now size 2.
        */

        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));
        // First kill, collateral damage ;)
        expect_n_pieces(&mut app, 36);
        force_move_piece(&mut app, Attacker, (5, 0), (5, 1));
        force_move_piece(&mut app, Defender, (6, 1), (6, 2));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 2), (6, 1));
        // First second kill, more collateral damage ;)
        expect_n_pieces(&mut app, 35);

        // Maneuver left side
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (3, 5), (3, 1));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (4, 5), (2, 5));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (2, 5), (2, 0));
        skip_turn(&mut app, Attacker);

        // Maneuver right side
        force_move_piece(&mut app, Defender, (7, 5), (7, 1));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 5), (8, 5));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (8, 5), (8, 0));
        skip_turn(&mut app, Attacker);

        force_move_piece(&mut app, Defender, (5, 3), (5, 0));

        expect_n_pieces(&mut app, 31);
    }

    #[test]
    fn only_king_can_stand_in_the_centre() {
        /*
        Proof only the king can stand in the centre piece, other pieces can't.
        Move king out of the centre
        Move king back into the centre
        Try move defender into the centre (fail)
        Try move attacker into the centre (fail)
        */

        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));
        // First kill, collateral damage ;)
        expect_n_pieces(&mut app, 36);
        force_move_piece(&mut app, Attacker, (5, 0), (5, 1));
        force_move_piece(&mut app, Defender, (6, 1), (6, 2));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (6, 2), (6, 1));
        // First second kill, more collateral damage ;)
        expect_n_pieces(&mut app, 35);

        // Move the 2 Defenders below the king
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (5, 3), (5, 0));
        skip_turn(&mut app, Attacker);
        force_move_piece(&mut app, Defender, (5, 4), (5, 1));
        skip_turn(&mut app, Attacker);

        // Move the King out of the centre place
        force_move_piece(&mut app, Defender, (5, 5), (5, 2));

        // Move an Attacker in the direction of the centre piece
        force_move_piece(&mut app, Attacker, (0, 4), (5, 4));

        // Try move a Defender into the centre piece
        force_move_piece(&mut app, Defender, (5, 6), (5, 5));
        // Last step should have failed, so we skip the turn instead
        skip_turn(&mut app, Defender);

        // Try move an Attacker in the direction of the centre piece
        force_move_piece(&mut app, Attacker, (5, 4), (5, 5));
        // Last step should have failed, so move the attacker piece away instead
        force_move_piece(&mut app, Attacker, (5, 4), (0, 4));

        // Move the king back into the centre piece
        force_move_piece(&mut app, Defender, (5, 2), (5, 5));
        // Should be succesfull
        skip_turn(&mut app, Attacker);

        expect_n_pieces(&mut app, 35);
    }
}

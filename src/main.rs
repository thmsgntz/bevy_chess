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
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);
    }

    #[test]
    fn simple_kill() {
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (7, 0), (7, 2));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));

        expect_n_pieces(&mut app, 36);
    }

    #[test]
    fn simple_kill_but_not_the_mover() {
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
    #[ignore]
    fn multi_kill() {
        let mut app = App::new();

        app.add_plugin(BoardPlugin).add_plugin(PiecesPlugin);

        app.update();

        expect_n_pieces(&mut app, 37);

        force_move_piece(&mut app, Defender, (4, 4), (4, 1));
        force_move_piece(&mut app, Attacker, (3, 0), (2, 0));
        force_move_piece(&mut app, Defender, (6, 4), (6, 1));
        // First kill, collateral damage ;)
        expect_n_pieces(&mut app, 36);

        force_move_piece(&mut app, Attacker, (7, 0), (8, 0));
        force_move_piece(&mut app, Defender, (3, 5), (3, 0));
        force_move_piece(&mut app, Attacker, (0, 3), (1, 3)); // Filler
        force_move_piece(&mut app, Defender, (7, 5), (7, 0));

        force_move_piece(&mut app, Attacker, (1, 3), (0, 3)); // Filler
        force_move_piece(&mut app, Defender, (5, 3), (5, 1));

        expect_n_pieces(&mut app, 33);
    }
}

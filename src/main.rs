use bevy::prelude::*;
use bevy_mod_picking::*;

mod pieces;
use pieces::*;
mod board;
use board::*;
mod ui;
use ui::*;

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
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

/// This system prints 'A' key state
fn keyboard_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<PointLight>)>,
    mut light_query: Query<&mut Transform, With<PointLight>>,
) {
    let mut camera_transform = camera_query.single_mut();
    let mut light_transform = light_query.single_mut();

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

    if home == true {
        let home = Transform::from_matrix(Mat4::from_rotation_translation(
            Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
            Vec3::new(-7.0, 20.0, 4.0),
        ));
        *camera_transform = home.clone();
        *light_transform = home.clone();
    } else {
        velocity = velocity.normalize_or_zero();

        camera_transform.translation += velocity * time.delta_seconds() * 10.0;
        camera_transform.rotate_y(rotate * time.delta_seconds());

        // Nullify the Y factor
        velocity *= Vec3::X + Vec3::Z;

        light_transform.translation += velocity * time.delta_seconds() * 10.0;
    }
}

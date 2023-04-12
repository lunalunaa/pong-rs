//! Shows how to render simple primitive shapes with a single color.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::{collide_aabb::collide, MaterialMesh2dBundle},
};
use rand::{thread_rng, Rng};

const DEFAULT_SPEED: f32 = 50.;
const BALL_SIZE: Vec2 = Vec2::new(10., 10.);
const PLAYER_SIZE: Vec2 = Vec2::new(100., 10.);
const BALL_INITIAL: Vec3 = Vec3::new(0., -250., 0.);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<GameState>()
        .add_startup_system(setup)
        .add_system(move_ball)
        .add_system(bounce_ball)
        .add_system(out_of_bounds)
        .add_system(keyboard_input)
        .run();
}

#[derive(Resource, Default)]
struct GameState {
    score: (u32, u32),
}

#[derive(Component)]
struct Player {
    name: String,
}

#[derive(Component)]
struct Speed {
    dir: Vec3,
    speed_multiplier: f32,
}

impl Default for Speed {
    fn default() -> Self {
        Self {
            dir: Default::default(),
            speed_multiplier: DEFAULT_SPEED,
        }
    }
}

#[derive(Component, Default)]
struct Ball;

#[derive(Component)]
struct Wall;

// spawns ball and player
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = thread_rng();

    commands.spawn(Camera2dBundle::default());

    let mut spawn_wall = |dim_x: f32, dim_y: f32, translation: Vec3| {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Box::new(dim_x, dim_y, 0.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform: Transform::from_translation(translation),
                ..default()
            },
            Wall,
        ));
    };

    spawn_wall(600., 10., Vec3::new(0., 300., 0.));

    spawn_wall(10., 600., Vec3::new(300., 0., 0.));

    spawn_wall(10., 600., Vec3::new(-300., 0., 0.));

    // spawning ball
    commands.spawn((
        (MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(10.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(BALL_INITIAL),
            ..default()
        }),
        Ball,
        Speed {
            dir: Vec3::new(rng.gen_range(-10.0..10.0), rng.gen_range(0.0..10.0), 0.),
            speed_multiplier: DEFAULT_SPEED,
        },
    ));

    // spawning player
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Box::new(100., 10., 0.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            transform: Transform::from_translation(Vec3::new(0., -290., 0.)),
            ..default()
        },
        Player {
            name: "Player".to_owned(),
        },
    ));
}

fn move_ball(mut query: Query<(&mut Transform, &mut Speed), With<Ball>>, timer: Res<Time>) {
    for (mut transform, mut speed) in &mut query {
        transform.translation += speed.dir * timer.delta_seconds() * speed.speed_multiplier;
        speed.speed_multiplier = DEFAULT_SPEED;
    }
}

fn get_wall_size(wall_trans: &Transform) -> Vec2 {
    if wall_trans.translation.x == 0. {
        Vec2::new(600., 10.)
    } else {
        Vec2::new(10., 600.)
    }
}

fn bounce_ball(
    mut query_ball: Query<(&Transform, &mut Speed), With<Ball>>,
    query_walls: Query<&Transform, With<Wall>>,
    query_player: Query<&Transform, With<Player>>,
) {
    for (ball_trans, mut speed) in &mut query_ball {
        for wall_trans in &query_walls {
            let wall_size = get_wall_size(wall_trans);
            let collided = collide(
                wall_trans.translation,
                wall_size,
                ball_trans.translation,
                BALL_SIZE,
            );

            if collided.is_some() {
                let (x, y) = (wall_trans.translation.x, wall_trans.translation.y);

                let wall_normal = if x == 0. && y == 300. {
                    Vec3::NEG_Y
                } else if x == 0. && y == -300. {
                    Vec3::Y
                } else if x == 300. && y == 0. {
                    Vec3::NEG_X
                } else {
                    Vec3::X
                };

                speed.dir = speed.dir - (2. * speed.dir.dot(wall_normal)) * wall_normal;
                speed.speed_multiplier *= 2.;
                break;
            }
        }

        for player_trans in &query_player {
            let collided = collide(
                player_trans.translation,
                PLAYER_SIZE,
                ball_trans.translation,
                BALL_SIZE,
            );

            if collided.is_some() {
                let normal = Vec3::Y;
                speed.dir = speed.dir - (2. * speed.dir.dot(normal)) * normal;
                speed.speed_multiplier *= 2.;
            }
        }
    }
}

fn out_of_bounds(mut query: Query<&mut Transform, With<Ball>>, mut game_state: ResMut<GameState>) {
    for mut ball in &mut query {
        let collided = collide(
            ball.translation,
            BALL_SIZE,
            Vec3::new(0., -300., 0.),
            Vec2::new(600., 10.),
        );

        // place the ball back to the starting position
        if collided.is_some() {
            ball.translation = BALL_INITIAL;
            game_state.score.0 += 1
        }
    }
}

fn keyboard_input(
    mut query: Query<&mut Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::Left) {
        for mut transform in &mut query {
            if (transform.translation - Vec3::new(25., 0., 0.) - Vec3::new(50., 0., 0.)).x >= -325.
            {
                transform.translation -= Vec3::new(10., 0., 0.)
            }
        }
    }

    if keyboard_input.pressed(KeyCode::Right) {
        for mut transform in &mut query {
            if (transform.translation + Vec3::new(25., 0., 0.) + Vec3::new(50., 0., 0.)).x <= 325. {
                transform.translation += Vec3::new(10., 0., 0.)
            }
        }
    }
}

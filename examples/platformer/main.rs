use bevy::prelude::*;
use bevy_2delight_physics::prelude::*;

#[derive(std::hash::Hash, Debug, Clone)]
enum TriggerRxKind {
    Player,
}
impl TriggerKind for TriggerRxKind {}

#[derive(std::hash::Hash, Debug, Clone)]
enum TriggerTxKind {
    Spikes,
}
impl TriggerKind for TriggerTxKind {}

#[derive(Default, Debug)]
enum BulletTimeSpeeds {
    #[default]
    Normal,
    Slow,
}
impl BulletTimeClass for BulletTimeSpeeds {
    fn to_factor(&self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Slow => 0.1,
        }
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(PhysicsPlugin::<
        TriggerRxKind,
        TriggerTxKind,
        BulletTimeSpeeds,
    >::default());

    app.add_systems(Startup, startup);

    app.run();
}

#[derive(Component)]
struct UninterestingObject;

fn startup(mut commands: Commands) {
    commands.spawn((Name::new("camera"), Camera2d));
    commands.spawn((
        Name::new("SanitySprite"),
        Sprite {
            color: Color::linear_rgb(1.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("UninterestingObject"),
        UninterestingObject,
        Sprite {
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::Z),
        Pos::new(-100.0, 0.0),
        Dyno::new(10.0, 0.0),
    ));
}

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_2delight_physics::prelude::*;

#[derive(std::hash::Hash, Debug, Clone)]
enum TriggerRxKind {
    Player,
}
impl TriggerKind for TriggerRxKind {}

#[derive(std::hash::Hash, Debug, Clone, PartialEq, Eq)]
enum TriggerTxKind {
    Spikes,
}
impl TriggerKind for TriggerTxKind {}

#[derive(Default, Debug, Clone)]
enum BulletTimeSpeed {
    #[default]
    Normal,
    Slow,
}
impl BulletTimeClass for BulletTimeSpeed {
    fn to_factor(&self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Slow => 0.1,
        }
    }
}
impl BulletTimeSpeed {
    pub fn rotated(&self) -> Self {
        match self {
            Self::Normal => Self::Slow,
            Self::Slow => Self::Normal,
        }
    }
}

// I _highly_ recommend you create type aliases here to cut back on some verbosity
type TriggerRx = TriggerRxGeneric<TriggerRxKind>;
type TriggerTx = TriggerTxGeneric<TriggerTxKind>;
type TriggerColls = TriggerCollsGeneric<TriggerRxKind, TriggerTxKind>;
#[allow(dead_code)]
type TriggerCollRec = TriggerCollRecGeneric<TriggerRxKind, TriggerTxKind>;
type BulletTime = BulletTimeGeneric<BulletTimeSpeed>;
type PhysicsPlugin = PhysicsPluginGeneric<TriggerRxKind, TriggerTxKind, BulletTimeSpeed>;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin::default(),
    ));
    app.add_plugins(PhysicsPlugin::default());

    app.add_systems(Startup, startup);
    app.add_systems(Update, update.after(PhysicsSet));

    app.run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
#[require(Name(|| Name::new("Ground")))]
struct Ground;
#[derive(Bundle)]
struct GroundBundle {
    ground: Ground,
    pos: Pos,
    sprite: Sprite,
    static_tx: StaticTx,
}
impl GroundBundle {
    fn new(pos: Pos, size: UVec2) -> Self {
        Self {
            ground: Ground,
            pos,
            sprite: Sprite {
                custom_size: Some(size.as_vec2()),
                ..default()
            },
            static_tx: StaticTx::single(StaticTxKind::Solid, HBox::new(size.x, size.y)),
        }
    }
}

#[derive(Component)]
#[require(Name(|| Name::new("Spike")))]
struct Spike;
#[derive(Bundle)]
struct SpikeBundle {
    spike: Spike,
    pos: Pos,
    sprite: Sprite,
    trigger_tx: TriggerTx,
}
impl SpikeBundle {
    fn new(pos: Pos, size: UVec2) -> Self {
        Self {
            spike: Spike,
            pos,
            sprite: Sprite {
                custom_size: Some(size.as_vec2()),
                color: Color::linear_rgb(1.0, 0.0, 0.0),
                ..default()
            },
            trigger_tx: TriggerTx::single(TriggerTxKind::Spikes, HBox::new(size.x, size.y)),
        }
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((Name::new("camera"), Camera2d));

    let player_hbox = HBox::new(36, 36);
    commands.spawn((
        Name::new("Player"),
        Player,
        Sprite {
            custom_size: Some(player_hbox.get_size().as_vec2()),
            color: Color::linear_rgb(0.1, 1.0, 0.1),
            ..default()
        },
        Pos::new(0.0, -50.0),
        Dyno::new(0.0, 0.0),
        StaticRx::single(StaticRxKind::Default, player_hbox.clone()),
        TriggerRx::single(TriggerRxKind::Player, player_hbox.clone()),
    ));

    commands.spawn(GroundBundle::new(
        Pos::new(0.0, -300.0),
        UVec2::new(800, 72),
    ));
    commands.spawn(GroundBundle::new(
        Pos::new(-300.0, 0.0),
        UVec2::new(200, 72),
    ));
    commands.spawn(GroundBundle::new(Pos::new(300.0, 0.0), UVec2::new(200, 72)));

    commands.spawn(SpikeBundle::new(Pos::new(-300.0, 72.0), UVec2::new(36, 72)));
    commands.spawn(SpikeBundle::new(Pos::new(300.0, 72.0), UVec2::new(36, 72)));
}

fn update(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut bullet_time: ResMut<BulletTime>,
    mut player_q: Query<(&mut Pos, &mut Dyno, &mut Sprite, &StaticRx, &TriggerRx), With<Player>>,
    static_colls: Res<StaticColls>,
    trigger_colls: Res<TriggerColls>,
) {
    // Maybe toggle bullet time
    if keyboard.just_pressed(KeyCode::Space) {
        let new_speed = bullet_time.get_base().rotated();
        bullet_time.set_base(new_speed);
    }

    let (mut pos, mut dyno, mut sprite, srx, trx) = player_q.single_mut();

    // Horizontal movement
    let x_mag = 200.0;
    dyno.vel.x = 0.0;
    if keyboard.pressed(KeyCode::KeyA) {
        dyno.vel.x -= x_mag;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        dyno.vel.x += x_mag;
    }

    // Vertical movement
    let gravity_mag = 600.0;
    let jump_mag = 400.0;
    dyno.vel.y -= bullet_time.delta_secs() * gravity_mag;
    if keyboard.just_pressed(KeyCode::KeyW) {
        dyno.vel.y = jump_mag;
        // Commenting out because it feels bad but here's how to add a short-term bullet time effect
        // bullet_time.add_effect(BulletTimeSpeeds::Slow, 0.1);
    }

    // How to check for collisions
    if static_colls
        .get_refs(&srx.coll_keys)
        .iter()
        .any(|coll| coll.tx_kind == StaticTxKind::Solid)
    {
        sprite.color = Color::linear_rgb(0.1, 1.0, 1.0);
    } else {
        sprite.color = Color::linear_rgb(0.1, 1.0, 0.1);
    }
    if trigger_colls
        .get_refs(&trx.coll_keys)
        .iter()
        .any(|coll| coll.tx_kind == TriggerTxKind::Spikes)
    {
        *pos = Pos::default();
    }
}

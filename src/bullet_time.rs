use bevy::prelude::*;

pub trait BulletTimeClass:
    std::fmt::Debug + Default + std::marker::Send + std::marker::Sync + 'static
{
    fn to_factor(&self) -> f32;
}

#[derive(Debug, Default)]
pub enum BulletTimeClassDefault {
    #[default]
    Normal,
}
impl BulletTimeClass for BulletTimeClassDefault {
    fn to_factor(&self) -> f32 {
        match self {
            Self::Normal => 1.0,
        }
    }
}

#[derive(Debug)]
struct BulletTimeEffect<TimeClass: BulletTimeClass> {
    class: TimeClass,
    time_left: f32,
}

#[derive(Debug, Default)]
struct BulletTimeState<TimeClass: BulletTimeClass> {
    base: TimeClass,
    effects: Vec<BulletTimeEffect<TimeClass>>,
}

impl<TimeClass: BulletTimeClass> BulletTimeState<TimeClass> {
    /// Ticks down any active effects
    fn tick(&mut self, real_time: f32) {
        for effect in &mut self.effects {
            effect.time_left -= real_time;
        }
        self.effects.retain(|effect| effect.time_left > 0.0);
    }

    /// Gets the current time factor. This is the slowest active effect, or base if there are no active effects
    fn to_factor(&self) -> f32 {
        self.effects
            .iter()
            .map(|effect| effect.class.to_factor())
            .reduce(|a, b| a.min(b))
            .unwrap_or_else(|| self.base.to_factor())
    }
}

/// How much in-game time has happened. Basically time but accounts for slowdown.
#[derive(Resource, Debug, Default)]
pub struct BulletTime<TimeClass: BulletTimeClass> {
    state: BulletTimeState<TimeClass>,
    duration: std::time::Duration,
}
impl<TimeClass: BulletTimeClass> BulletTime<TimeClass> {
    pub fn delta(&self) -> std::time::Duration {
        self.duration
    }
    pub fn delta_secs(&self) -> f32 {
        self.duration.as_secs_f32()
    }
    pub fn set_base(&mut self, new_base: TimeClass) {
        self.state.base = new_base;
    }
    pub fn add_effect(&mut self, class: TimeClass, time: f32) {
        self.state.effects.push(BulletTimeEffect {
            class,
            time_left: time,
        });
    }
    pub fn clear_effects(&mut self) {
        self.state.effects.clear();
    }
}

fn update_bullet_time<TimeClass: BulletTimeClass>(
    mut bullet_time: ResMut<BulletTime<TimeClass>>,
    time: Res<Time>,
) {
    bullet_time.state.tick(time.delta_secs());
    bullet_time.duration = time.delta().mul_f32(bullet_time.state.to_factor());
}

#[derive(Default)]
pub(crate) struct BulletTimePlugin<TimeClass: BulletTimeClass> {
    _pd: std::marker::PhantomData<TimeClass>,
}
impl<TimeClass: BulletTimeClass> Plugin for BulletTimePlugin<TimeClass> {
    fn build(&self, app: &mut App) {
        app.insert_resource(BulletTime::<TimeClass>::default());
        app.add_systems(First, update_bullet_time::<TimeClass>);
    }
}

use bevy::prelude::*;

use crate::{
    bullet_time::{BulletTimeClassDefault, BulletTimePlugin},
    colls, logic, pos,
    prelude::BulletTimeClass,
    triggers::TriggerKind,
};

pub struct PhysicsPlugin<
    TriggerRxKind: TriggerKind,
    TriggerTxKind: TriggerKind,
    TimeClass: BulletTimeClass = BulletTimeClassDefault,
> {
    _pd: std::marker::PhantomData<(TriggerRxKind, TriggerTxKind, TimeClass)>,
}
impl<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind, TimeClass: BulletTimeClass> Default
    for PhysicsPlugin<TriggerRxKind, TriggerTxKind, TimeClass>
{
    fn default() -> Self {
        Self {
            _pd: std::marker::PhantomData,
        }
    }
}
impl<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind, TimeClass: BulletTimeClass> Plugin
    for PhysicsPlugin<TriggerRxKind, TriggerTxKind, TimeClass>
{
    fn build(&self, app: &mut App) {
        colls::register_colls::<TriggerRxKind, TriggerTxKind>(app);
        logic::register_logic::<TriggerRxKind, TriggerTxKind, TimeClass>(app);
        pos::register_pos(app);
        app.add_plugins(BulletTimePlugin::<TimeClass>::default());
    }
}

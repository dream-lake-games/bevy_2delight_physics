use bevy::prelude::*;

pub mod prelude {
    pub use super::bullet_time::{BulletTimeClass, BulletTimeGeneric};
    pub use super::colls::{
        ByHBox, StaticCollRec, StaticColls, TriggerCollRecGeneric, TriggerCollsGeneric,
    };
    pub use super::dyno::Dyno;
    pub use super::hbox::{HBox, HBoxMarker};
    pub use super::plugin::PhysicsPluginGeneric;
    pub use super::pos::{IPos, Pos};
    pub use super::statics::{StaticRx, StaticRxKind, StaticTx, StaticTxKind};
    pub use super::triggers::{TriggerKind, TriggerRxGeneric, TriggerTxGeneric};
    pub use super::PhysicsSet;
}

mod bullet_time;
mod colls;
mod dyno;
mod hbox;
mod logic;
mod plugin;
mod pos;
mod statics;
mod triggers;

/// The set that contains all physics related systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

/// The physics-internal set that resolves collisions
/// NOTE: Subset of PhysicsSet
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CollSet;

/// The physics-internal set that resolves collisions
/// NOTE: Subset of CollisionSet
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct PosSet;

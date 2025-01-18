use bevy::prelude::*;

use crate::{colls::CollKey, hbox::HBox, pos::Pos};

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, std::hash::Hash)]
pub enum StaticRxKind {
    /// Pushes the rx ctrl out of tx comps, sets vel to zero along plane of intersection
    Default,
    /// Observes collisions but does nothing to respond
    Observe,
}
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, std::hash::Hash)]
pub enum StaticTxKind {
    /// Standard solid thing. Stops stuff
    Solid,
}

pub(crate) struct StaticRxComp {
    pub(crate) kind: StaticRxKind,
    pub(crate) hbox: HBox,
}
#[derive(Component)]
pub struct StaticRx {
    pub(crate) comps: Vec<StaticRxComp>,
    pub coll_keys: Vec<CollKey>,
}
impl StaticRx {
    pub fn single(kind: StaticRxKind, hbox: HBox) -> Self {
        Self::new(vec![(kind, hbox)])
    }
    pub fn new<I: IntoIterator<Item = (StaticRxKind, HBox)>>(data: I) -> Self {
        Self {
            comps: data
                .into_iter()
                .map(|(kind, hbox)| StaticRxComp { kind, hbox })
                .collect(),
            coll_keys: vec![],
        }
    }
}

pub(crate) struct StaticTxComp {
    pub(crate) kind: StaticTxKind,
    pub(crate) hbox: HBox,
}
#[derive(Component)]
pub struct StaticTx {
    pub(crate) comps: Vec<StaticTxComp>,
    pub coll_keys: Vec<CollKey>,
}
impl StaticTx {
    pub fn single(kind: StaticTxKind, hbox: HBox) -> Self {
        Self::new(vec![(kind, hbox)])
    }
    pub fn new<I: IntoIterator<Item = (StaticTxKind, HBox)>>(data: I) -> Self {
        Self {
            comps: data
                .into_iter()
                .map(|(kind, hbox)| StaticTxComp { kind, hbox })
                .collect(),
            coll_keys: vec![],
        }
    }
    pub fn get_thboxes(&self, pos: Pos) -> Vec<HBox> {
        self.comps
            .iter()
            .map(|comp| comp.hbox.translated(pos.x, pos.y))
            .collect()
    }
}

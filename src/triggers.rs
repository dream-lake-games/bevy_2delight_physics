use bevy::prelude::*;

use crate::{colls::CollKey, hbox::HBox};

pub trait TriggerKind:
    Clone + std::fmt::Debug + std::hash::Hash + std::marker::Send + std::marker::Sync + 'static
{
}

struct TriggerRxComp<TriggerRxKind: TriggerKind> {
    kind: TriggerRxKind,
    hbox: HBox,
}
#[derive(Component)]
pub struct TriggerRx<TriggerRxKind: TriggerKind> {
    comps: Vec<TriggerRxComp<TriggerRxKind>>,
    pub coll_keys: Vec<CollKey>,
}
impl<TriggerRxKind: TriggerKind> TriggerRx<TriggerRxKind> {
    pub fn single(kind: TriggerRxKind, hbox: HBox) -> Self {
        Self::new(vec![(kind, hbox)])
    }
    pub fn new<I: IntoIterator<Item = (TriggerRxKind, HBox)>>(data: I) -> Self {
        Self {
            comps: data
                .into_iter()
                .map(|(kind, hbox)| TriggerRxComp { kind, hbox })
                .collect(),
            coll_keys: vec![],
        }
    }
}

struct TriggerTxComp<TriggerTxKind: TriggerKind> {
    kind: TriggerTxKind,
    hbox: HBox,
}
#[derive(Component)]
pub struct TriggerTx<TriggerTxKind: TriggerKind> {
    comps: Vec<TriggerTxComp<TriggerTxKind>>,
    pub coll_keys: Vec<CollKey>,
}
impl<TriggerTxKind: TriggerKind> TriggerTx<TriggerTxKind> {
    pub fn single(kind: TriggerTxKind, hbox: HBox) -> Self {
        Self::new(vec![(kind, hbox)])
    }
    pub fn new<I: IntoIterator<Item = (TriggerTxKind, HBox)>>(data: I) -> Self {
        Self {
            comps: data
                .into_iter()
                .map(|(kind, hbox)| TriggerTxComp { kind, hbox })
                .collect(),
            coll_keys: vec![],
        }
    }
}

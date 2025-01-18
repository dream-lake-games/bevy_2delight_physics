use bevy::{prelude::*, utils::HashMap};

use crate::{
    hbox::HBoxMarker,
    pos::Pos,
    statics::{StaticRx, StaticRxKind, StaticTx, StaticTxKind},
    triggers::{TriggerKind, TriggerRx, TriggerTx},
    PhysicsSet,
};

pub type CollKey = u32;

#[derive(Debug, Clone, Reflect)]
pub struct StaticCollRec {
    pub push: Vec2,
    /// Position of rx at time of collision
    pub rx_pos: Pos,
    /// Before collision, component of rx's velocity in collision normal direction
    pub rx_perp: Vec2,
    /// Before collision, component of rx's velocity perpendicular to normal direction
    /// Name is weird because it's "parallel" to original vel of rx
    pub rx_par: Vec2,
    /// Entity of the control associated with the rx
    pub rx_ctrl: Entity,
    /// The kind of the rx
    pub rx_kind: StaticRxKind,
    /// The marker of the hbox on the rx  triggering this collision
    pub rx_hbox: HBoxMarker,
    /// Position of tx at time of collision
    pub tx_pos: Pos,
    /// Entity of the control associated with the tx
    pub tx_ctrl: Entity,
    /// The kind of the tx
    pub tx_kind: StaticTxKind,
    /// The marker of the hbox on the tx  triggering this collision
    pub tx_hbox: HBoxMarker,
}
#[derive(Resource, Debug, Reflect)]
pub struct StaticColls {
    map: HashMap<CollKey, StaticCollRec>,
}
impl StaticColls {
    pub(crate) fn insert(&mut self, key: CollKey, rec: StaticCollRec) {
        self.map.insert(key, rec);
    }
    pub fn get(&self, key: &CollKey) -> Option<&StaticCollRec> {
        self.map.get(key)
    }
    pub fn get_refs(&self, coll_keys: &[CollKey]) -> Vec<&StaticCollRec> {
        coll_keys.iter().filter_map(|key| self.get(key)).collect()
    }
    pub fn all(&self) -> Vec<&StaticCollRec> {
        self.map.values().into_iter().collect()
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct TriggerCollRec<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind> {
    pub pos: Pos,
    pub rx_ctrl: Entity,
    pub rx_kind: TriggerRxKind,
    pub rx_hbox: HBoxMarker,
    pub tx_ctrl: Entity,
    pub tx_kind: TriggerTxKind,
    pub tx_hbox: HBoxMarker,
}
#[derive(Resource, Debug, Reflect)]
pub struct TriggerColls<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind> {
    map: HashMap<CollKey, TriggerCollRec<TriggerRxKind, TriggerTxKind>>,
}
impl<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>
    TriggerColls<TriggerRxKind, TriggerTxKind>
{
    pub fn insert(&mut self, key: CollKey, rec: TriggerCollRec<TriggerRxKind, TriggerTxKind>) {
        self.map.insert(key, rec);
    }
    pub fn get(&self, key: &CollKey) -> Option<&TriggerCollRec<TriggerRxKind, TriggerTxKind>> {
        self.map.get(key)
    }
    pub fn get_refs(
        &self,
        coll_keys: &[CollKey],
    ) -> Vec<&TriggerCollRec<TriggerRxKind, TriggerTxKind>> {
        coll_keys.iter().filter_map(|key| self.get(key)).collect()
    }
}

/// Helpful trait to categorize collisions by marked hitboxes
pub trait ByHBox<'a, Record> {
    fn by_rx_hbox(self) -> HashMap<HBoxMarker, Vec<&'a Record>>;
    fn by_tx_hbox(self) -> HashMap<HBoxMarker, Vec<&'a Record>>;
}
impl<'a> ByHBox<'a, StaticCollRec> for Vec<&'a StaticCollRec> {
    fn by_rx_hbox(self) -> HashMap<HBoxMarker, Vec<&'a StaticCollRec>> {
        let mut result = HashMap::<HBoxMarker, Vec<&'a StaticCollRec>>::new();
        for rec in self.into_iter() {
            if result.get_mut(&rec.rx_hbox).is_some() {
                result.get_mut(&rec.rx_hbox).unwrap().push(rec);
            } else {
                result.insert(rec.rx_hbox, vec![rec]);
            }
        }
        result
    }
    fn by_tx_hbox(self) -> HashMap<HBoxMarker, Vec<&'a StaticCollRec>> {
        let mut result = HashMap::<HBoxMarker, Vec<&'a StaticCollRec>>::new();
        for rec in self.into_iter() {
            if result.get_mut(&rec.tx_hbox).is_some() {
                result.get_mut(&rec.tx_hbox).unwrap().push(rec);
            } else {
                result.insert(rec.tx_hbox, vec![rec]);
            }
        }
        result
    }
}
impl<'a, TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>
    ByHBox<'a, TriggerCollRec<TriggerRxKind, TriggerTxKind>>
    for Vec<&'a TriggerCollRec<TriggerRxKind, TriggerTxKind>>
{
    fn by_rx_hbox(
        self,
    ) -> HashMap<HBoxMarker, Vec<&'a TriggerCollRec<TriggerRxKind, TriggerTxKind>>> {
        let mut result =
            HashMap::<HBoxMarker, Vec<&'a TriggerCollRec<TriggerRxKind, TriggerTxKind>>>::new();
        for rec in self.into_iter() {
            if result.get_mut(&rec.rx_hbox).is_some() {
                result.get_mut(&rec.rx_hbox).unwrap().push(rec);
            } else {
                result.insert(rec.rx_hbox, vec![rec]);
            }
        }
        result
    }
    fn by_tx_hbox(
        self,
    ) -> HashMap<HBoxMarker, Vec<&'a TriggerCollRec<TriggerRxKind, TriggerTxKind>>> {
        let mut result =
            HashMap::<HBoxMarker, Vec<&'a TriggerCollRec<TriggerRxKind, TriggerTxKind>>>::new();
        for rec in self.into_iter() {
            if result.get_mut(&rec.tx_hbox).is_some() {
                result.get_mut(&rec.tx_hbox).unwrap().push(rec);
            } else {
                result.insert(rec.tx_hbox, vec![rec]);
            }
        }
        result
    }
}

fn reset_colls_every_frame<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>(
    mut static_colls: ResMut<StaticColls>,
    mut trigger_colls: ResMut<TriggerColls<TriggerRxKind, TriggerTxKind>>,
    mut srx_ctrls: Query<&mut StaticRx>,
    mut stx_ctrls: Query<&mut StaticTx>,
    mut trx_ctrls: Query<&mut TriggerRx<TriggerRxKind>>,
    mut ttx_ctrls: Query<&mut TriggerTx<TriggerTxKind>>,
) {
    // Eh at some point we may want to shrink memory used, but this probably fine
    static_colls.map.clear();
    trigger_colls.map.clear();
    macro_rules! clear_coll_keys {
        ($thing:expr) => {
            for mut thing in &mut $thing {
                thing.coll_keys.clear();
            }
        };
    }
    clear_coll_keys!(srx_ctrls);
    clear_coll_keys!(stx_ctrls);
    clear_coll_keys!(trx_ctrls);
    clear_coll_keys!(ttx_ctrls);
}

pub(super) fn register_colls<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>(
    app: &mut App,
) {
    app.insert_resource(StaticColls { map: default() });
    app.insert_resource(TriggerColls::<TriggerRxKind, TriggerTxKind> { map: default() });

    app.add_systems(
        First,
        reset_colls_every_frame::<TriggerRxKind, TriggerTxKind>.in_set(PhysicsSet),
    );
}

use bevy::prelude::*;

use crate::{
    colls::{StaticCollRec, StaticColls, TriggerCollRecGeneric, TriggerCollsGeneric},
    dyno::Dyno,
    hbox::HBox,
    pos::Pos,
    prelude::{
        BulletTimeClass, BulletTimeGeneric, StaticRx, StaticRxKind, StaticTx, StaticTxKind,
        TriggerKind, TriggerRxGeneric, TriggerTxGeneric,
    },
    PhysicsSet,
};

/// A helpful function to make sure physics things exist as we expect them to
fn invariants(
    dyno_without_pos: Query<Entity, (With<Dyno>, Without<Pos>)>,
    static_rx_n_tx: Query<Entity, (With<StaticRx>, With<StaticTx>)>,
    moving_static_tx_vert_only: Query<&Dyno, With<StaticTx>>,
) {
    debug_assert!(dyno_without_pos.is_empty());
    debug_assert!(static_rx_n_tx.is_empty());
    for dyno in &moving_static_tx_vert_only {
        debug_assert!(dyno.vel.x.abs() == 0.0);
    }
}

/// Moves dynos that have no statics and no trigger receivers
fn move_uninteresting_dynos<TriggerRxKind: TriggerKind, TimeClass: BulletTimeClass>(
    bullet_time: Res<BulletTimeGeneric<TimeClass>>,
    mut ents: Query<
        (&Dyno, &mut Pos),
        (
            Without<StaticRx>,
            Without<StaticTx>,
            Without<TriggerRxGeneric<TriggerRxKind>>,
        ),
    >,
) {
    for (dyno, mut pos) in &mut ents {
        *pos += dyno.vel * bullet_time.delta_secs();
    }
}

/// Moves static txs
fn move_static_txs<TimeClass: BulletTimeClass>(
    bullet_time: Res<BulletTimeGeneric<TimeClass>>,
    mut ents: Query<(&Dyno, &mut Pos), (Without<StaticRx>, With<StaticTx>)>,
) {
    for (dyno, mut pos) in &mut ents {
        *pos += dyno.vel * bullet_time.delta_secs();
    }
}

/// Resolves collisions for a single entity.
/// If it has statics, it resolves static collisions and may update pos and vel
/// If it has triggers, it will trigger as needed (duh)
fn resolve_collisions<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>(
    my_eid: Entity,
    my_pos: &mut Pos,
    my_vel: &mut Vec2,
    my_srx: Option<(Entity, &StaticRx)>,
    my_trx: Option<(Entity, &TriggerRxGeneric<TriggerRxKind>)>,
    pos_q: &Query<&mut Pos>,
    dyno_q: &Query<&mut Dyno>,
    stx_q: &Query<(Entity, &mut StaticTx)>,
    ttx_q: &Query<(Entity, &mut TriggerTxGeneric<TriggerTxKind>)>,
    static_colls: &mut ResMut<StaticColls>,
    trigger_colls: &mut ResMut<TriggerCollsGeneric<TriggerRxKind, TriggerTxKind>>,
) {
    // Handle static collisions
    struct StaticCollCandidate {
        eid: Entity,
        pos: Pos,
        kind: StaticTxKind,
        thbox: HBox,
    }

    // Update all pos/dyno for static collisions, create records
    if let Some((_, my_srx)) = my_srx {
        for my_srx_comp in &my_srx.comps {
            let mut my_thbox = my_srx_comp.hbox.translated(my_pos.x, my_pos.y);
            // TODO: Performance engineer if needed
            // In order to avoid weird behavior when sliding along a straight edge, do this
            // First filter to only things it's colliding with
            let mut candidates = stx_q
                .iter()
                .flat_map(|(eid, stx)| {
                    let pos = pos_q.get(eid).expect("Missing pos on stx");
                    stx.comps.iter().map(move |comp| StaticCollCandidate {
                        eid,
                        pos: pos.clone(),
                        kind: comp.kind,
                        thbox: comp.hbox.translated(pos.x, pos.y),
                    })
                })
                .filter(|candidate| candidate.eid != my_eid)
                .filter(|candidate| my_thbox.overlaps_with(&candidate.thbox))
                .collect::<Vec<_>>();
            candidates.sort_by(|a, b| {
                //shutup rust
                let dist_a = a.thbox.area_overlapping_assuming_overlap(&my_thbox);
                let dist_b = b.thbox.area_overlapping_assuming_overlap(&my_thbox);
                dist_b.total_cmp(&dist_a)
            });
            for candidate in candidates {
                let Some(push) = my_thbox.get_push_out(&candidate.thbox) else {
                    // Likely means that resolving an earlier collision pushed us out of this box, do nothing
                    continue;
                };

                // COLLISION ACTUALLY HAPPENING
                let tx_dyno = dyno_q.get(candidate.eid).cloned().unwrap_or_default();
                let mut old_perp = my_vel.dot(push.normalize_or_zero()) * push.normalize_or_zero();
                let old_par = *my_vel - old_perp;
                if push.y.abs() > 0.0 {
                    old_perp.y -= tx_dyno.vel.y;
                }

                let coll_rec = StaticCollRec {
                    push,
                    rx_pos: my_pos.clone(),
                    rx_perp: old_perp,
                    rx_par: old_par,
                    rx_ctrl: my_eid,
                    rx_kind: my_srx_comp.kind,
                    rx_hbox: my_srx_comp.hbox.get_marker(),
                    tx_pos: candidate.pos,
                    tx_ctrl: candidate.eid,
                    tx_kind: candidate.kind,
                    tx_hbox: candidate.thbox.get_marker(),
                };

                let mut do_push = |grr: &mut HBox| {
                    *my_pos += push;
                    *grr = grr.translated(push.x, push.y);
                };

                match (my_srx_comp.kind, candidate.kind) {
                    (StaticRxKind::Default, StaticTxKind::Solid) => {
                        // Solid collision, no breaking
                        static_colls.insert(coll_rec);
                        do_push(&mut my_thbox);
                        *my_vel = old_par + Vec2::new(0.0, tx_dyno.vel.y);
                        if old_perp.dot(push) > 0.0 {
                            *my_vel += old_perp;
                        }
                    }
                    // TODO: Do I want this?
                    // (StaticRxKind::Default, StaticTxKind::PassUp) => {
                    //     // Any kind of passup
                    //     if push.y > 0.0
                    //         && old_perp.y < 0.0
                    //         && other_thbox.max_y() - 1.1 < my_thbox.min_y()
                    //     {
                    //         add_coll_rec();
                    //         do_push(&mut my_thbox);
                    //         *my_vel = old_par + Vec2::new(0.0, tx_dyno.vel.y);
                    //     }
                    // }
                    (StaticRxKind::Observe, _) => {
                        static_colls.insert(coll_rec);
                    }
                }
            }
        }
    }

    // Handle trigger collisions
    struct TriggerCollCandidate<InnerTriggerTxKind> {
        eid: Entity,
        pos: Pos,
        kind: InnerTriggerTxKind,
        thbox: HBox,
    }

    // Create trigger coll records
    if let Some((_, my_trx)) = my_trx {
        for my_trx_comp in &my_trx.comps {
            let my_thbox = my_trx_comp.hbox.translated(my_pos.x, my_pos.y);
            let candidates = ttx_q
                .iter()
                .flat_map(|(eid, ttx)| {
                    let pos = pos_q.get(eid).expect("Missing pos on ttx");
                    ttx.comps.iter().map(move |comp| TriggerCollCandidate {
                        eid,
                        pos: pos.clone(),
                        kind: comp.kind.clone(),
                        thbox: comp.hbox.translated(pos.x, pos.y),
                    })
                })
                .filter(|candidate| candidate.eid != my_eid)
                .filter(|candidate| my_thbox.overlaps_with(&candidate.thbox));
            for candidate in candidates {
                let coll_rec = TriggerCollRecGeneric {
                    rx_pos: my_pos.clone(),
                    rx_ctrl: my_eid,
                    rx_kind: my_trx_comp.kind.clone(),
                    rx_hbox: my_trx_comp.hbox.get_marker(),
                    tx_pos: candidate.pos,
                    tx_ctrl: candidate.eid,
                    tx_kind: candidate.kind,
                    tx_hbox: candidate.thbox.get_marker(),
                };
                trigger_colls.insert(coll_rec);
            }
        }
    }
}

/// As we resolve collisions, we create the collisions records but don't put the corresponding
/// keys in the needed vecs in the ctrls. This helper does that, assuming all colls have been resolved.
fn populate_ctrl_coll_keys<TriggerRxKind: TriggerKind, TriggerTxKind: TriggerKind>(
    srx_q: &mut Query<(Entity, &mut StaticRx)>,
    stx_q: &mut Query<(Entity, &mut StaticTx)>,
    trx_q: &mut Query<(Entity, &mut TriggerRxGeneric<TriggerRxKind>)>,
    ttx_q: &mut Query<(Entity, &mut TriggerTxGeneric<TriggerTxKind>)>,
    static_colls: &ResMut<StaticColls>,
    trigger_colls: &ResMut<TriggerCollsGeneric<TriggerRxKind, TriggerTxKind>>,
) {
    for (key, coll) in &static_colls.map {
        if let Ok((_, mut srx_ctrl)) = srx_q.get_mut(coll.rx_ctrl) {
            srx_ctrl.coll_keys.push(*key);
        }
        if let Ok((_, mut stx_ctrl)) = stx_q.get_mut(coll.tx_ctrl) {
            stx_ctrl.coll_keys.push(*key);
        }
    }
    for (key, coll) in &trigger_colls.map {
        if let Ok((_, mut trx_ctrl)) = trx_q.get_mut(coll.rx_ctrl) {
            trx_ctrl.coll_keys.push(*key);
        }
        if let Ok((_, mut ttx_ctrl)) = ttx_q.get_mut(coll.tx_ctrl) {
            ttx_ctrl.coll_keys.push(*key);
        }
    }
}

/// Moves the interesting stuff and handles collisions
fn move_interesting_dynos<
    TriggerRxKind: TriggerKind,
    TriggerTxKind: TriggerKind,
    TimeClass: BulletTimeClass,
>(
    bullet_time: Res<BulletTimeGeneric<TimeClass>>,
    mut pos_q: Query<&mut Pos>,
    mut dyno_q: Query<&mut Dyno>,
    mut srx_q: Query<(Entity, &mut StaticRx)>,
    mut stx_q: Query<(Entity, &mut StaticTx)>,
    mut trx_q: Query<(Entity, &mut TriggerRxGeneric<TriggerRxKind>)>,
    mut ttx_q: Query<(Entity, &mut TriggerTxGeneric<TriggerTxKind>)>,
    mut static_colls: ResMut<StaticColls>,
    mut trigger_colls: ResMut<TriggerCollsGeneric<TriggerRxKind, TriggerTxKind>>,
    // Objects that have a static rx. They may also have a trigger rx.
    // Basically all the stuff we should move in this system
    ents_q: Query<
        Entity,
        (
            With<Pos>,
            Without<StaticTx>,
            Or<(With<StaticRx>, With<TriggerRxGeneric<TriggerRxKind>>)>,
        ),
    >,
) {
    // First do the moving
    for eid in &ents_q {
        // Get the data
        let mut scratch_pos = pos_q.get(eid).expect("No pos on interesting ent").clone();
        let mut scratch_vel = dyno_q.get(eid).unwrap_or(&Dyno::default()).vel.clone();
        let srx = srx_q.get(eid).ok();
        let trx = trx_q.get(eid).ok();
        debug_assert!(srx.is_some() || trx.is_some());
        // Inch
        macro_rules! call_resolve_collisions {
            () => {{
                resolve_collisions(
                    eid,
                    &mut scratch_pos,
                    &mut scratch_vel,
                    srx,
                    trx,
                    &pos_q,
                    &dyno_q,
                    &mut stx_q,
                    &mut ttx_q,
                    &mut static_colls,
                    &mut trigger_colls,
                )
            }};
        }
        const DELTA_PER_INCH: f32 = 1.0;
        // Resolve collisions once always so stationary objects are still pushed out of each other
        call_resolve_collisions!();
        // Inch horizontally
        let mut amt_moved_hor: f32 = 0.0;
        let max_inch_hor = scratch_vel.x.abs() * bullet_time.delta_secs();
        while amt_moved_hor < max_inch_hor.min(scratch_vel.x.abs()) {
            let dont_overshoot = (max_inch_hor.min(scratch_vel.x.abs()) - amt_moved_hor).max(0.0);
            let moving_this_step = DELTA_PER_INCH.min(dont_overshoot);
            amt_moved_hor += moving_this_step;
            scratch_pos.x += scratch_vel.x.signum() * moving_this_step;
            call_resolve_collisions!();
        }
        // Then inch vertically
        let mut amt_moved_ver: f32 = 0.0;
        let max_inch_ver = scratch_vel.y.abs() * bullet_time.delta_secs();
        while amt_moved_ver < max_inch_ver.min(scratch_vel.y.abs()) {
            let dont_overshoot = (max_inch_ver.min(scratch_vel.y.abs()) - amt_moved_ver).max(0.0);
            let moving_this_step = DELTA_PER_INCH.min(dont_overshoot);
            amt_moved_ver += moving_this_step;
            scratch_pos.y += scratch_vel.y.signum() * moving_this_step;
            call_resolve_collisions!();
        }
        // NOTE: Why do this (inch horizontally then vertically)? Stops bugs going up and down against wall.
        // ^read: celeste does this
        // Set the data
        let mut set_pos = pos_q.get_mut(eid).expect("No pos on interesting ent");
        *set_pos = scratch_pos;
        if let Ok(mut set_dyno) = dyno_q.get_mut(eid) {
            set_dyno.vel = scratch_vel;
        }
    }
    // Then update the records in the controls once
    populate_ctrl_coll_keys(
        &mut srx_q,
        &mut stx_q,
        &mut trx_q,
        &mut ttx_q,
        &static_colls,
        &trigger_colls,
    );
}

pub(super) fn register_logic<
    TriggerRxKind: TriggerKind,
    TriggerTxKind: TriggerKind,
    TimeClass: BulletTimeClass,
>(
    app: &mut App,
) {
    app.add_systems(
        Update,
        (
            invariants,
            move_uninteresting_dynos::<TriggerRxKind, TimeClass>,
            move_static_txs::<TimeClass>,
            move_interesting_dynos::<TriggerRxKind, TriggerTxKind, TimeClass>,
        )
            .chain()
            .in_set(PhysicsSet)
            .in_set(super::CollSet)
            .before(super::PosSet),
    );
}

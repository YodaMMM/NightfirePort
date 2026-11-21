//! The driving tracks' dynamic scene objects: which are drawn, which are in the sim, and which
//! have been knocked loose as smackables (docs/formats/trackobj.md). Data comes from
//! `nf_formats::trackobj`; this is the state the trigger actions drive. No physics: the
//! `ASmackable` body (mass, inertia from `MOMENT`, the impulse `FUN_0015de48` applies) is not
//! ported.
//!
//! Rules [M, DRIVING.ELF]:
//! - Init World builds every flag-0x10 item with `ps+7 != 0xff` as a scene object, drawn from
//!   the start (`FUN_001c7098` sets `+0x18` bit 24, which the draw `FUN_001c8c70` tests). Items
//!   whose `ps` behaviour has bit 4 join the sim at once (`FUN_0022faf8` → `FUN_002015d8`).
//! - The triggers that hold `EStartMission` fire during Init World (`FUN_0022cd30`), so their
//!   hide/disable actions apply before the first frame. Other triggers fire on conditions this
//!   module does not model; [`TrackProps::apply`] runs one action when the caller decides.
//! - Hide (0x17) / show (0x18) flip the draw bit only; disable (0x95) also leaves the sim
//!   (`FUN_00201690`), enable (0x96) shows and rejoins (`FUN_002015d8`).
//! - Wake (0x23, `FUN_0015de48`): with behaviour bit 2 an `ASmackable` is spawned when the sim
//!   has room: `FUN_002012d0` retires the active smackable farthest from the new one (skipping
//!   those whose `+0x6e` is 2) when the sim's count `+8` is above 36, and allows the spawn only
//!   when the count is then below 36. A refused wake marks the trigger (`+0x12 |= 1`).
//! - The restart path (`FUN_00230730`) rebuilds every scene object from the saved records.

use nf_formats::trackobj::{behaviour, ObjectAction, SmackableAttrib, TrackObject, SMACKABLE_CAP};

/// One scene object's state.
#[derive(Clone, Debug, PartialEq)]
pub struct PropState {
    /// The `in#0` index.
    pub placement: usize,
    pub origin: [f32; 3],
    /// Drawn (`RSceneObj +0x18` bit 24).
    pub visible: bool,
    /// Registered with the sim.
    pub in_sim: bool,
    /// Knocked loose: an `ASmackable` with these hit points is running.
    pub smackable_hp: Option<f32>,
    behaviour: u32,
    damage: u8,
}

/// Every scene object of a track.
#[derive(Clone, Debug, Default)]
pub struct TrackProps {
    pub props: Vec<PropState>,
}

/// What a wake did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WakeResult {
    /// An `ASmackable` started; `retired` is the placement it pushed out of the pool, if any.
    Spawned { retired: Option<usize> },
    /// The sim had no room; the trigger gets `+0x12 |= 1` [M 0x15de48].
    Refused,
    /// Behaviour bit 0x40: the `FUN_002002e0` path, not ported.
    AltPath,
    /// Neither bit: nothing happens (and the trigger is marked like a refusal [M 0x15de48]).
    Nothing,
}

impl TrackProps {
    /// The state right after Init World, before any trigger [M 0x22faf8 / 0x1c7098].
    pub fn new(objects: &[TrackObject]) -> TrackProps {
        let props = objects
            .iter()
            .filter(|o| o.scene_object())
            .map(|o| PropState {
                placement: o.index,
                origin: o.origin,
                visible: true,
                in_sim: o.record.behaviour & behaviour::SIM_AT_INIT != 0,
                smackable_hp: None,
                behaviour: o.record.behaviour,
                damage: o.record.damage,
            })
            .collect();
        TrackProps { props }
    }

    /// The state at the first frame: [`TrackProps::new`] plus the start trigger's actions
    /// [M 0x22cd30 fires the `EStartMission` triggers at Init World].
    pub fn at_start(objects: &[TrackObject], start_actions: &[ObjectAction]) -> TrackProps {
        let mut t = TrackProps::new(objects);
        for a in start_actions {
            if !matches!(a, ObjectAction::Wake { .. }) {
                t.apply(a);
            }
        }
        t
    }

    pub fn get(&self, placement: usize) -> Option<&PropState> { self.props.iter().find(|p| p.placement == placement) }

    fn get_mut(&mut self, placement: usize) -> Option<&mut PropState> { self.props.iter_mut().find(|p| p.placement == placement) }

    /// Hide / show / disable / enable [M 0x14f218 / 0x159b38 / 0x14b630 / 0x14bb58]. A wake
    /// needs the sim state: use [`TrackProps::wake`]. Returns false when the placement has no
    /// scene object (the game then only flips the item's `+0x0c` bit 0).
    pub fn apply(&mut self, a: &ObjectAction) -> bool {
        let Some(p) = self.get_mut(a.placement()) else { return false };
        match *a {
            ObjectAction::Hide { .. } => p.visible = false,
            ObjectAction::Show { .. } => p.visible = true,
            ObjectAction::Disable { .. } => {
                p.visible = false;
                p.in_sim = false;
            }
            ObjectAction::Enable { .. } => {
                p.visible = true;
                p.in_sim = true;
            }
            ObjectAction::Wake { .. } => return false,
        }
        true
    }

    /// Active smackables.
    pub fn active_smackables(&self) -> usize { self.props.iter().filter(|p| p.smackable_hp.is_some()).count() }

    /// Wakes `placement` [M 0x15de48 → 0x1ffa90 → 0x2012d0 / 0x19e100]. `sim_count` is the sim's
    /// object count (`0x339190 +8`; what it counts besides smackables is not traced).
    pub fn wake(&mut self, placement: usize, attrib: &SmackableAttrib, sim_count: usize) -> WakeResult {
        let Some(p) = self.get(placement) else { return WakeResult::Nothing };
        if p.behaviour & behaviour::SMACKABLE == 0 {
            return if p.behaviour & behaviour::ALT_WAKE != 0 { WakeResult::AltPath } else { WakeResult::Nothing };
        }
        let at = p.origin;
        let damage = p.damage;
        let mut count = sim_count;
        let mut retired = None;
        if count > SMACKABLE_CAP {
            let far = self
                .props
                .iter()
                .filter(|q| q.smackable_hp.is_some())
                .map(|q| (q.placement, (0..3).map(|k| (q.origin[k] - at[k]).powi(2)).sum::<f32>()))
                .fold(None::<(usize, f32)>, |b, x| if b.map_or(true, |b| b.1 < x.1) { Some(x) } else { b });
            if let Some((r, _)) = far {
                self.get_mut(r).unwrap().smackable_hp = None;
                retired = Some(r);
                count -= 1;
            }
        }
        if count >= SMACKABLE_CAP {
            return WakeResult::Refused;
        }
        self.get_mut(placement).unwrap().smackable_hp = Some(attrib.start_hit_points(damage));
        WakeResult::Spawned { retired }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::trackobj::InstanceRecord;

    fn obj(index: usize, behaviour: u32, z: f32) -> TrackObject {
        TrackObject {
            index,
            article: None,
            flags: 0x10,
            ps: index as u16,
            record: InstanceRecord { kind: 1, damage: 0, behaviour, scene_part: 0 },
            origin: [0.0, 0.0, z],
            radius: 2.75,
        }
    }

    #[test]
    fn start_actions_hide_and_take_out_of_the_sim_before_the_first_frame() {
        let objs = [obj(1, 0x203, 0.0), obj(2, 0x4, 0.0), obj(3, 0x203, 0.0)];
        let t = TrackProps::new(&objs);
        assert!(t.props.iter().all(|p| p.visible));
        assert_eq!(t.props.iter().map(|p| p.in_sim).collect::<Vec<_>>(), [false, true, false]);
        let t = TrackProps::at_start(&objs, &[ObjectAction::Disable { placement: 2 }, ObjectAction::Wake { placement: 1, ps: 1 }]);
        let p2 = t.get(2).unwrap();
        assert!(!p2.visible && !p2.in_sim);
        assert!(t.get(1).unwrap().smackable_hp.is_none(), "a start wake needs the sim, not run here");
        let mut t = t;
        assert!(t.apply(&ObjectAction::Enable { placement: 2 }));
        assert!(t.get(2).unwrap().visible && t.get(2).unwrap().in_sim);
        assert!(t.apply(&ObjectAction::Hide { placement: 3 }));
        assert!(!t.get(3).unwrap().visible && !t.get(3).unwrap().in_sim);
        assert!(!t.apply(&ObjectAction::Hide { placement: 99 }));
    }

    #[test]
    fn wake_spawns_with_damage_scaled_hit_points_and_respects_the_sim_cap() {
        let mut objs: Vec<TrackObject> = (0..40).map(|i| obj(i, 0x203, i as f32)).collect();
        objs[5].record.damage = 64;
        objs.push(obj(100, 0x40, 0.0));
        objs.push(obj(101, 0x0, 0.0));
        let mut t = TrackProps::new(&objs);
        let a = SmackableAttrib::parse("[Default]\nMASS=10\nHITPOINTS=10\n");
        assert_eq!(t.wake(5, &a, 0), WakeResult::Spawned { retired: None });
        assert_eq!(t.get(5).unwrap().smackable_hp, Some(7.5));
        assert_eq!(t.wake(100, &a, 0), WakeResult::AltPath);
        assert_eq!(t.wake(101, &a, 0), WakeResult::Nothing);
        // 36 in the sim: no retire (only above 36), no room.
        assert_eq!(t.wake(6, &a, 36), WakeResult::Refused);
        // 37: the farthest active smackable from the new one goes, then 36 is still no room.
        assert_eq!(t.wake(39, &a, 1), WakeResult::Spawned { retired: None });
        assert_eq!(t.wake(0, &a, 37), WakeResult::Refused);
        assert_eq!(t.get(39).unwrap().smackable_hp, None, "39 was farthest from 0 and retired");
        assert_eq!(t.get(5).unwrap().smackable_hp, Some(7.5));
        // 35: room.
        assert_eq!(t.wake(7, &a, 35), WakeResult::Spawned { retired: None });
        assert_eq!(t.active_smackables(), 2);
    }
}

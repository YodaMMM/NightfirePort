//! The monitor consoles: placement 0x30 (`Monitor_Create` 0x1b08b0, `Monitor_Activate` 0x1b0058,
//! `Monitor_Update` 0x1afcc8).
//!
//! Semantics [M]:
//! - `Monitor_Create` 0x1b08b0: instance 0x14, class 0x38; `obj+0xF6` = param 0 (the monitor
//!   group id, matched against the sensors' param-6 byte); param 1 == 0 → health 1.0
//!   (shootable), else health 0 (immune).
//! - `Monitor_Activate` 0x1b0058 (the E path, `Player_Activate` case 0x38): requires state 0 and
//!   a class-3 activator; picks the FIRST sensor from `CameraList` (walked from the head,
//!   param 5 == 0, not destroyed) whose `+0x5E` (param 6) equals the monitor's group; hides it
//!   (`obj+0xF8 = 0`); state 2; `Camera_SetUpdator` (the view renders the sensor's pose — draw
//!   side), `Input_SetAction(0xE,1)` (the activating press is consumed); `Player_Disable`; HUD
//!   panes.
//! - `Monitor_Update` 0x1afcc8 while viewing: a watched sensor that died (obj+0xF4 == 3) is
//!   unhidden and the replacement walk runs FORWARD from it, falling back to the list head; no
//!   replacement → watched = 0 → the exit arm (state 9 + `Camera_SetUpdator(…,0x18f5d8)` +
//!   `Player_Enable` — the port derives the restore from the state). Each viewed tick writes
//!   `switch_channels[watched_sensor+0x5C] = 1` (the sensor's param-8 monitor-view channel,
//!   SKIPPED when the channel is 0 — `if (uVar2 != 0)`, :171-174), mirrors the sensor pose into
//!   the viewer (draw side) and sets the sensor's +0x60 unhide. The action-0xE press edge
//!   (mask 4, :118) restores the watched sensor's mask to 1 and walks FORWARD from it (its
//!   node's next) — NO wrap: the next live same-group camera, or watched = 0 → the exit.
//! - The exit cooldown counts DOWN one state per tick 9 → 8 → … → 5, and 4 → 0 (re-usable)
//!   [M 0x1afcc8:55-65].
//! - Shot (`Monitor_ApplyDamage`): health − damage < 0 → `Control_NextLOD`, state 3 and
//!   `Explode_Create(r·6, r, 0.5, …, script 0x6000027, 200, 200)`; the restore arm (:35-42)
//!   runs when a player was viewing (the port's derived freeze ends with the state).
//!
//! The camera-view rendering (the sensor's picture on screen) is the draw side's work; the sim
//! delivers the state, the watched-sensor selection and the channel writes.

use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::level::Level;

#[derive(Clone, Debug)]
pub struct Monitor {
    pub placement: usize,
    /// The object position (obj+0x80 stand-in: the placement translation).
    pub pos: V3,
    /// `obj+0xF6` = param 0 — the group id matched against the sensors' param-6 byte.
    pub group: u8,
    /// param 1 == 0 → 1.0 (shootable), else 0.0 (immune) [M 0x1b08b0:16-21].
    pub health: f32,
    /// obj+0xF4: 0 idle, 2 viewing, 3 destroyed, 9/4 the exit cooldown [M 0x1b0058/0x1afcc8].
    pub state: u8,
    /// The watched sensor's index into `Sensors::list` while state 2.
    pub watched: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Monitors {
    pub list: Vec<Monitor>,
}

impl Monitors {
    /// `Monitor_Create` over the main map's 0x30 placements (placement order, case 0x30
    /// [M 0x1d04f0]). The position is the placed object's glist-bounds centre — 07000001's
    /// monitor (#105 `room_communication_tv`) sits at the identity transform with world-space
    /// object verts [D placements], like the rooms; the game keys the placed object's own
    /// obj+0x80 (rebuilt from the matrix + glist centre by the 0x1336f8 pass 6).
    pub fn create(level: &Level) -> Monitors {
        let Some(m) = level.main_map() else { return Monitors::default() };
        let mut list = Vec::new();
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0x30 { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (lo, hi) = crate::props::bounds(level, m, p);
            let pos = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
            list.push(Monitor {
                placement: i,
                pos,
                group: (param(0) & 0xff) as u8,
                health: if param(1) == 0 { 1.0 } else { 0.0 },
                state: 0,
                watched: None,
            });
        }
        Monitors { list }
    }

    /// `Monitor_Activate` 0x1b0058: the state gate is `monitor->0xF4 == 0` [:12-13] — the
    /// ACTIVATE is the enter only: pick the group's FIRST `CameraList` sensor (`Sensor.listed`,
    /// not destroyed — the head walk :17-36), hide it (`obj+0xF8 = 0`, :26) → state 2. While
    /// viewing the action-0xE edge is `Monitor_Update`'s own consumer (0x1afcc8:117-168) — it
    /// rides `tick`'s `monitor_edge`, NOT this call (the decompile's activate never runs in
    /// state 2: `Player_Activate` is behind the player freeze). Returns whether the enter fired.
    pub fn activate(&mut self, i: usize, sensors: &mut crate::sensor::Sensors) -> bool {
        let Some(m) = self.list.get_mut(i) else { return false };
        if m.state == 3 { return false; }
        if m.state != 0 { return false; }
        let group = m.group;
        let first = Self::next_listed_from(sensors, group, 0);
        match first {
            Some(si) => {
                // [M 0x1b0058:26] the picked camera's display mask clears (the body hides).
                sensors.list[si].display_mask = 0;
                m.state = 2;
                m.watched = Some(si);
                true
            }
            None => false,
        }
    }

    /// The CameraList order is the sensors' creation order — the game tail-appends every listed
    /// sensor [M `LList_Add` 0x1cca28; `Monitor_Activate` 0x1b0058:17-36 walks the list from the
    /// head] — so the port's `Sensors::list` index order. A candidate is listed, the monitor's
    /// group, and not destroyed (`obj+0xF4 == 3` skips [M 0x1b0058:21, 0x1afcc8:80/98/138]).
    fn next_listed_from(sensors: &crate::sensor::Sensors, group: u8, from: usize) -> Option<usize> {
        (from..sensors.list.len()).find(|&si| {
            let s = &sensors.list[si];
            s.listed && s.monitor_id == group && s.state != 3
        })
    }

    /// `Monitor_Update` 0x1afcc8's per-tick body for the viewing monitors: a watched sensor that
    /// died hands the view to the next live same-group sensor (forward, then the head) or exits;
    /// the action-0xE press edge (`monitor_edge` — the game polls it INSIDE this function,
    /// `Input_Action(slot, 0xE, 4)` :117-118) cycles FORWARD with no wrap or exits; the viewed
    /// tick latches the sensor's monitor-view channel (channel 0 skips the write) and sets its
    /// +0x60 unhide; the cooldown states drain 9 → 8 → … → 5 → 4 → 0 — a SEPARATE switch arm
    /// from the state-2 exit, so the exit tick itself leaves state 9 and the drain starts the
    /// next tick [M :55-65 vs :159-168].
    pub fn tick(&mut self, sensors: &mut crate::sensor::Sensors, channels: &mut Channels, frame: u32,
                monitor_edge: bool) {
        for m in &mut self.list {
            match m.state {
                2 => {
                    // [M 0x1afcc8:68-115] the watched sensor's death (obj+0xF4 == 3): unhide it
                    // (:75), then the replacement walk forward from it (:76-93) and, when that
                    // runs out, from the CameraList head (:94-114); none → watched = 0 (:114).
                    if let Some(si) = m.watched {
                        if sensors.list.get(si).is_none_or(|s| s.state == 3) {
                            if let Some(s) = sensors.list.get_mut(si) { s.display_mask = 1; }
                            let group = m.group;
                            m.watched = Self::next_listed_from(sensors, group, si + 1)
                                .or_else(|| Self::next_listed_from(sensors, group, 0));
                            if let Some(ni) = m.watched { sensors.list[ni].display_mask = 0; }
                        }
                    }
                    // [M 0x1afcc8:117-158] the press edge: the watched camera's mask restores
                    // (:125) and the walk goes FORWARD from it (its node's next) — no wrap; the
                    // next live same-group camera hides (:143) or the exit arm runs (:159-168).
                    if monitor_edge {
                        if let Some(ci) = m.watched {
                            sensors.list[ci].display_mask = 1;
                            m.watched = Self::next_listed_from(sensors, m.group, ci + 1);
                        }
                        match m.watched {
                            Some(ni) => sensors.list[ni].display_mask = 0,
                            None => m.state = 9,
                        }
                    }
                    if m.watched.is_none() {
                        // [M 0x1afcc8:159-168] the exit arm (the watched camera died with no
                        // replacement, or the press walked past the last camera): state 9, the
                        // restore is the port's derived freeze-end.
                        m.state = 9;
                    } else {
                        let si = m.watched.unwrap();
                        let s = &sensors.list[si];
                        // [M 0x1afcc8:170-174] the monitor-view channel latches 1 — the game's
                        // own `if (uVar2 != 0)` skips the write for channel 0.
                        if s.monitor_ch != 0 { channels.set(s.monitor_ch as u32, 1, frame); }
                        // [M 0x1afcc8:176] the viewed sensor's +0x60 unhide sets every tick.
                        sensors.list[si].unhidden = true;
                    }
                }
                9 | 8 | 7 | 6 | 5 => m.state -= 1, // [M 0x1afcc8:61-63 the decrement arm]
                4 => m.state = 0,                  // [M 0x1afcc8:65]
                1 => m.state = 0,                  // [M 0x1afcc8:51-55 the <3 non-0 arm]
                _ => {}
            }
        }
    }

    /// The bullet arm (`Monitor_ApplyDamage`): health − damage < 0 → state 3 + the explode event.
    /// Returns true when this shot destroyed it. Immune monitors (param 1 != 0, health 0) never
    /// die [M 0x1b08b0:16-21].
    pub fn bullet_hit(&mut self, a: V3, b: V3, damage: f32) -> bool {
        for m in &mut self.list {
            if m.state == 3 || m.health <= 0.0 { continue; }
            // The use-sphere stand-in (crate::props::seg_box's rule): the segment's AABB against
            // the monitor's small box.
            let (lo, hi) = ([m.pos[0] - 0.6, m.pos[1] - 0.6, m.pos[2] - 0.6], [m.pos[0] + 0.6, m.pos[1] + 0.6, m.pos[2] + 0.6]);
            let hit = (0..3).all(|k| a[k].max(b[k]) >= lo[k] && a[k].min(b[k]) <= hi[k]);
            if hit {
                m.health -= damage;
                if m.health < 0.0 {
                    m.state = 3;
                    m.watched = None;
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor() -> Monitor {
        Monitor { placement: 0, pos: [0.0, 6.0, 0.0], group: 1, health: 1.0, state: 0, watched: None }
    }

    fn sensor_at(placement: usize, group: u8, monitor_ch: u16) -> crate::sensor::Sensor {
        crate::sensor::Sensor {
            placement, pos: [1.0; 3], matrix: Default::default(), quat: Default::default(),
            keys: Vec::new(), limit: 0, step: 0.0, t: 0.0, cos_half: 0.0, sin_half: 0.0,
            range: 6.0, out: 0, disable: 0, monitor_id: group, listed: true, lens: 0, mount: 0,
            monitor_ch, state: 0, detect: 0.0, beep: 0.0, seen: 0, last_hit: false, unhidden: false,
            beam_len: 0.0, display_mask: 0x1F,
        }
    }

    #[test]
    fn activating_picks_the_group_sensor_and_the_view_writes_its_monitor_channel() {
        // 07000001: the monitor's p0 = 1 and the camera sensor's p6 = 1 [D placements dump];
        // the sensor's p8 (monitor_ch) rides the same rule [M 0x1afcc8:170-174 —
        // `switch_channels[sensor_inst+0x5C] = 1` per viewed tick, gated `if (uVar2 != 0)`:
        // the TEST previously pinned a channel-0 write, which contradicts the decompile —
        // fixed to a nonzero channel + a channel-0 sister].
        let mut mons = Monitors { list: vec![monitor()] };
        let mut sensors = crate::sensor::Sensors { list: vec![sensor_at(1, 1, 5)] };
        assert!(mons.activate(0, &mut sensors), "state 0 + a group-1 listed sensor → state 2 [M 0x1b0058]");
        assert_eq!(mons.list[0].state, 2);
        assert_eq!(sensors.list[0].display_mask, 0, "the picked camera's display mask clears [M 0x1b0058:26]");
        let mut ch = Channels::default();
        mons.tick(&mut sensors, &mut ch, 10, false);
        assert!(ch.on(5), "the viewed tick writes the sensor's monitor-view channel [M 0x1afcc8:170-174]");
        assert!(sensors.list[0].unhidden, "the sensor's +0x60 unhide sets while viewed [M 0x1afcc8:176]");
        // The channel-0 sister: the game's own gate skips the write.
        let mut mons0 = Monitors { list: vec![monitor()] };
        let mut sensors0 = crate::sensor::Sensors { list: vec![sensor_at(1, 1, 0)] };
        mons0.activate(0, &mut sensors0);
        let mut ch0 = Channels::default();
        mons0.tick(&mut sensors0, &mut ch0, 10, false);
        assert!(!ch0.on(0), "channel 0 skips the write [M 0x1afcc8:171 `if (uVar2 != 0)`]");
    }

    #[test]
    fn the_view_cycles_forward_and_the_press_after_the_last_camera_exits_and_re_arms() {
        // [M 0x1afcc8:117-158] the press edge is Monitor_Update's own consumer: the cycle walks
        // FORWARD from the watched sensor (its node's next) — no wrap: from B the next is C —
        // and past the last camera the exit arm runs (:159-168: state 9). The exit and the
        // cooldown drain are SEPARATE switch arms, so the exit tick itself leaves state 9 and
        // the drain starts the next tick, one state per tick 9 → 8 → … → 5, 4 → 0 [M :55-65],
        // and the console re-arms.
        let mut mons = Monitors { list: vec![monitor()] };
        let mut sensors = crate::sensor::Sensors {
            list: vec![sensor_at(1, 1, 0), sensor_at(2, 1, 0), sensor_at(3, 1, 0)],
        };
        assert!(mons.activate(0, &mut sensors));
        assert_eq!(mons.list[0].watched, Some(0), "the enter picks the list head [M 0x1b0058:17-36]");
        mons.tick(&mut sensors, &mut Channels::default(), 0, true); // the E edge cycles
        assert_eq!(mons.list[0].watched, Some(1), "from A forward → B [M 0x1afcc8:124-153]");
        assert_eq!(sensors.list[0].display_mask, 1, "the cycled-away mask restores [M 0x1afcc8:125]");
        assert_eq!(sensors.list[1].display_mask, 0, "the next camera hides [M 0x1afcc8:143]");
        mons.tick(&mut sensors, &mut Channels::default(), 1, true);
        assert_eq!(mons.list[0].watched, Some(2));
        mons.tick(&mut sensors, &mut Channels::default(), 2, true); // past the last camera
        assert_eq!(mons.list[0].state, 9, "the exit runs [M 0x1afcc8:159-168]");
        assert_eq!(mons.list[0].watched, None);
        assert_eq!(sensors.list[2].display_mask, 1, "the last camera unhides before the walk [M 0x1afcc8:125]");
        // The activate is the state-0 enter only [M 0x1b0058:12-13] — it refuses while the
        // cooldown stands.
        assert!(!mons.activate(0, &mut sensors), "state 9 refuses the activate [M 0x1b0058:12-13]");
        for expected in [8, 7, 6, 5, 4] {
            mons.tick(&mut sensors, &mut Channels::default(), 0, false);
            assert_eq!(mons.list[0].state, expected, "the cooldown decrements [M 0x1afcc8:61-63]");
        }
        mons.tick(&mut sensors, &mut Channels::default(), 0, false);
        assert_eq!(mons.list[0].state, 0, "state 4 → 0 [M 0x1afcc8:65]");
        assert!(mons.activate(0, &mut sensors), "the cooled-down console re-arms");
        assert_eq!(mons.list[0].watched, Some(0));
    }

    #[test]
    fn a_dead_watched_camera_hands_the_view_to_the_next_or_exits() {
        // [M 0x1afcc8:68-115] the watched sensor's obj+0xF4 == 3: unhide it (:75), then the
        // replacement walk forward from it (:76-93), falling back to the CameraList head
        // (:94-114); no replacement → the exit arm (:159-168).
        let mut mons = Monitors { list: vec![monitor()] };
        let mut sensors = crate::sensor::Sensors { list: vec![sensor_at(1, 1, 0), sensor_at(2, 1, 0)] };
        mons.activate(0, &mut sensors);
        assert_eq!(mons.list[0].watched, Some(0));
        sensors.list[0].state = 3; // the disable channel put it down [M 0x1af618:107-117]
        mons.tick(&mut sensors, &mut Channels::default(), 0, false);
        assert_eq!(mons.list[0].watched, Some(1), "the replacement walks forward [M 0x1afcc8:76-93]");
        assert_eq!(sensors.list[0].display_mask, 1, "the dead camera unhides [M 0x1afcc8:75]");
        assert_eq!(sensors.list[1].display_mask, 0);
        sensors.list[1].state = 3;
        mons.tick(&mut sensors, &mut Channels::default(), 0, false);
        assert_eq!(mons.list[0].state, 9, "no replacement → the exit [M 0x1afcc8:159-168]");
        assert_eq!(mons.list[0].watched, None);
    }

    #[test]
    fn the_sister_idle_monitor_never_writes_and_a_shot_destroys_the_console() {
        // Sister: no activation → no channel, ever.
        let mut mons = Monitors { list: vec![monitor()] };
        let mut sensors = crate::sensor::Sensors { list: Vec::new() };
        let mut ch = Channels::default();
        mons.tick(&mut sensors, &mut ch, 10, false);
        assert!(!ch.on(0), "an idle monitor writes nothing");
        assert!(!mons.activate(0, &mut sensors), "no same-group sensor → the activate refuses");
        // A 1.0-health console dies to one 25-damage bullet [M Monitor_ApplyDamage].
        let mut mons = Monitors { list: vec![monitor()] };
        assert!(mons.bullet_hit([-1.0, 6.0, 0.0], [1.0, 6.0, 0.0], 25.0));
        assert_eq!(mons.list[0].state, 3, "state 3 [M Monitor_ApplyDamage]");
        // The sister miss.
        let mut mons = Monitors { list: vec![monitor()] };
        assert!(!mons.bullet_hit([-1.0, 60.0, 0.0], [1.0, 60.0, 0.0], 25.0));
        assert_eq!(mons.list[0].state, 0);
    }
}

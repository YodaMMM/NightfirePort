//! Which vehicle the player is in, per driving mission and per scripted phase, as DRIVING.ELF
//! picks it. Write-up: docs/systems/drive-player-vehicles.md.
//!
//! Evidence: [M] DRIVING.ELF decompile (EE addresses), [D] disc bytes, [S] suggested.
//!
//! - **The spawn vehicle** [M]: `FUN_0022faf8` hands the `MSet` record to `FUN_00205220`, which
//!   keeps its payload at mission+0x1c; `FUN_00204d18` copies that pointer (the payload opens
//!   with a C string) to mission+0x1b44, and only for the names `db5` (0x397498) / `z8`
//!   (0x3974a0) replaces it with the first set unlock flag's car: `DAT_0033a010` lotus,
//!   `DAT_0033a014` playervan, `DAT_0033a018` playerlimo, `DAT_0033a01c` playertaxi,
//!   `DAT_0033a020` playersmart (none set: the name stays). "Spawn player car" `FUN_00176658`
//!   takes the name from `FUN_002051f8`: mission+0x1b44, or `jungle_truck` when the low-memory
//!   byte 0x1009a1 has bit 0 set. The car ctor `FUN_001862c0` (:117-126) would prefer
//!   `DAT_0033196c` for the player, but nothing in DRIVING.ELF writes it.
//! - **The only swap** [M]: the one call of the player car's vtable slot +0x8c (`FUN_00185e50`,
//!   the in-place rebuild under a new pvehicle name: attrs, model, sound, body) in the whole
//!   decompile is `ECustomTransition` `FUN_0014b020`, and it runs only when the event's string
//!   is `paris` (0x36f2d0). The new name is slot +0x294 = `FUN_001932d0` = car+0x450, which the
//!   ctor fills from the `.atr` key `SECONDARY_TYPE` (`FUN_00169f18(attrs, 0x379a88, 0)` at
//!   :133). `FUN_00185e50` rebuilds only when the name differs from the current one
//!   (car+0x2cc). No other event touches the player's vehicle type: `EControlToCPU` /
//!   `EControlToPlayer` / `EControlToSpline` / `EEndControlToSpline` change who drives it.
//! - **Who drives** [M]: `EControlToSpline` (202, `FUN_0014ae60`, word 1 == 0 the player) hands
//!   the player car's driver (car+0x2e4, slot +0x5c `FUN_00192fe8`) the spline through its
//!   slot +0x24 `FUN_00131508`, which sets car+0x2c8 = 1 (slot +0x94 `FUN_00193000`), clears
//!   the inputs (slot +0xb4 `FUN_001872d8`), driver state 3 and builds the spline controller at
//!   driver+0x50 (`FUN_0013dfd8`). `EEndControlToSpline` (239, `FUN_0014c110`) runs slot +0x2c
//!   `FUN_00131590`: car+0x2c8 = 0, inputs cleared, driver state 1, controller freed
//!   (`FUN_0013dfd8(driver, 0)` zeroes +0x50).

use crate::drive_script::{MapScript, ScriptEvent};

/// `EControlToSpline` [M 0x14ae60].
pub const KIND_CONTROL_TO_SPLINE: u32 = 202;
/// `EEndControlToSpline` [M 0x14c110].
pub const KIND_END_CONTROL_TO_SPLINE: u32 = 239;
/// `ECustomTransition` [M 0x14b020].
pub const KIND_CUSTOM_TRANSITION: u32 = 240;

/// The one transition name `FUN_0014b020` acts on (0x36f2d0) [M].
pub const TRANSITION_PARIS: &str = "paris";

/// The unlock cars `FUN_00204d18` puts in place of `db5` / `z8`, in its test order [M].
pub const UNLOCK_CARS: [&str; 5] = ["lotus", "playervan", "playerlimo", "playertaxi", "playersmart"];

/// The run-time switches the spawn choice reads. All off is what this port knows of a normal
/// run: no DRIVING.ELF code writes `DAT_0033a010..020`, and the writer of 0x1009a1 bit 0 (a
/// byte the launching program leaves in low memory) is not traced [S].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpawnSwitches {
    /// `DAT_0033a010`, `DAT_0033a014`, `DAT_0033a018`, `DAT_0033a01c`, `DAT_0033a020`.
    pub unlock: [bool; 5],
    /// 0x1009a1 bit 0 (`FUN_002051f8`).
    pub jungle_truck: bool,
}

/// The pvehicle name an `MSet` payload opens with (a C string; `FUN_00204d18` stores the
/// payload pointer itself as the name) [M]. `None` for an empty name.
pub fn mset_vehicle(mset: &[u8]) -> Option<String> {
    let end = mset.iter().position(|&b| b == 0).unwrap_or(mset.len());
    (end > 0).then(|| String::from_utf8_lossy(&mset[..end]).into_owned())
}

/// `FUN_00204d18` then `FUN_002051f8`: the name the player car is built from [M].
pub fn spawn_vehicle(mset_name: &str, sw: SpawnSwitches) -> String {
    if sw.jungle_truck {
        return "jungle_truck".to_string();
    }
    if mset_name == "db5" || mset_name == "z8" {
        if let Some(i) = sw.unlock.iter().position(|&u| u) {
            return UNLOCK_CARS[i].to_string();
        }
    }
    mset_name.to_string()
}

/// A string value of a pvehicle `.atr` (`KEY.s=value`; the `.s` suffix optional), as the
/// attribute reader returns it. `None` when the key is absent (the reader's default 0).
pub fn atr_string(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some((k, v)) = line.split_once('=') else { continue };
        let k = k.trim();
        let k = k.split('.').next().unwrap_or(k).trim();
        if k == key {
            let v = v.trim();
            return (!v.is_empty()).then(|| v.to_string());
        }
    }
    None
}

/// `SECONDARY_TYPE` (0x379a88): the vehicle a `paris` transition rebuilds the car as [M+D].
pub fn secondary_type(atr: &str) -> Option<String> {
    atr_string(atr, "SECONDARY_TYPE")
}

/// The string an event's word `i` names, without the `CHAR::` prefix the `sr` strings carry
/// [D]. That the run-time pointer skips the prefix is [S] (the reference loader is not traced).
fn event_string<'a>(e: &ScriptEvent, i: usize, script: &'a MapScript) -> Option<&'a str> {
    let s = script.string(e.reference(i)?)?;
    Some(s.strip_prefix("CHAR::").unwrap_or(s))
}

/// What a scripted event does to the player's vehicle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VehicleEvent {
    /// `EControlToSpline` for the player: the car follows the spline the string names.
    ToSpline(String),
    /// `EEndControlToSpline` for the player: the spline controller is freed.
    EndSpline,
    /// `ECustomTransition` with its name (`paris` is the only one that acts).
    Transition(String),
}

/// Reads one event's effect on the player vehicle [M the handlers above]. Events aimed at an
/// AI element (word 1 != 0) and every other kind give `None`.
pub fn vehicle_event(e: &ScriptEvent, script: &MapScript) -> Option<VehicleEvent> {
    match e.kind {
        KIND_CONTROL_TO_SPLINE if e.word(1) == Some(0) => {
            Some(VehicleEvent::ToSpline(event_string(e, 2, script).unwrap_or("").to_string()))
        }
        KIND_END_CONTROL_TO_SPLINE if e.word(1) == Some(0) => Some(VehicleEvent::EndSpline),
        KIND_CUSTOM_TRANSITION => Some(VehicleEvent::Transition(event_string(e, 1, script).unwrap_or("").to_string())),
        _ => None,
    }
}

/// The player's vehicle and its driver at one point of a mission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerVehicle {
    /// The pvehicle name (car+0x2cc).
    pub name: String,
    /// The spline the car follows (driver+0x50 set), `None` = the player drives.
    pub spline: Option<String>,
}

impl PlayerVehicle {
    /// The vehicle at spawn [M `FUN_00176658`]: no spline yet.
    pub fn spawn(mset_name: &str, sw: SpawnSwitches) -> PlayerVehicle {
        PlayerVehicle { name: spawn_vehicle(mset_name, sw), spline: None }
    }

    /// Applies one event. `secondary` answers `SECONDARY_TYPE` for a pvehicle name (the
    /// mission archive's `.atr`). Returns true when the vehicle type changed.
    pub fn apply(&mut self, ev: &VehicleEvent, secondary: &dyn Fn(&str) -> Option<String>) -> bool {
        match ev {
            VehicleEvent::ToSpline(s) => { self.spline = Some(s.clone()); false }
            VehicleEvent::EndSpline => { self.spline = None; false }
            VehicleEvent::Transition(n) if n == TRANSITION_PARIS => match secondary(&self.name) {
                // FUN_00185e50 rebuilds only for a different name.
                Some(next) if next != self.name => { self.name = next; true }
                _ => false,
            },
            VehicleEvent::Transition(_) => false,
        }
    }
}

/// Every event list of a script that holds a player-vehicle event, with those events in list
/// order: the places a mission can change the player's vehicle or driver.
pub fn vehicle_lists(script: &MapScript) -> Vec<(u16, Vec<VehicleEvent>)> {
    script
        .lists
        .iter()
        .filter_map(|(&id, evs)| {
            let v: Vec<VehicleEvent> = evs.iter().filter_map(|e| vehicle_event(e, script)).collect();
            (!v.is_empty()).then_some((id, v))
        })
        .collect()
}

/// The vehicles a mission can put the player in: the spawn vehicle, then each distinct type a
/// `paris` transition in any of its lists would rebuild it as (in list order, each applied to
/// the vehicle the previous one left). A mission with no acting transition has one entry.
pub fn mission_vehicles(script: &MapScript, sw: SpawnSwitches, secondary: &dyn Fn(&str) -> Option<String>) -> Vec<String> {
    let Some(name) = script.chunks.get("MSet").and_then(|m| mset_vehicle(m)) else { return Vec::new() };
    let mut v = PlayerVehicle::spawn(&name, sw);
    let mut out = vec![v.name.clone()];
    for (_, evs) in vehicle_lists(script) {
        for e in &evs {
            if v.apply(e, secondary) && !out.contains(&v.name) {
                out.push(v.name.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mset_name_is_the_leading_c_string() {
        let mut p = b"helicopter".to_vec();
        p.extend_from_slice(&[0, 0x41, 0x42]);
        assert_eq!(mset_vehicle(&p).as_deref(), Some("helicopter"));
        assert_eq!(mset_vehicle(&[0, 1, 2]), None);
    }

    #[test]
    fn spawn_rules() {
        let off = SpawnSwitches::default();
        assert_eq!(spawn_vehicle("helicopter", off), "helicopter");
        assert_eq!(spawn_vehicle("db5", off), "db5", "no unlock flag: the name stays");
        let mut sw = off;
        sw.unlock[3] = true;
        assert_eq!(spawn_vehicle("db5", sw), "playertaxi");
        assert_eq!(spawn_vehicle("z8", sw), "playertaxi");
        assert_eq!(spawn_vehicle("vanquish", sw), "vanquish", "only db5 / z8 are swapped");
        sw.unlock[1] = true;
        assert_eq!(spawn_vehicle("z8", sw), "playervan", "the first set flag in FUN_00204d18's order wins");
        sw.jungle_truck = true;
        assert_eq!(spawn_vehicle("helicopter", sw), "jungle_truck", "FUN_002051f8 overrides everything");
    }

    #[test]
    fn secondary_type_reads_the_string_key() {
        let atr = "[helicopter]\r\nRENDER_FILENAME.s=helicopter.crp\r\nSECONDARY_TYPE.s=vanquish\r\nMASS.f=1660.0\r\n";
        assert_eq!(secondary_type(atr).as_deref(), Some("vanquish"));
        assert_eq!(secondary_type("[vanquish]\nMASS=1660\n"), None);
        assert_eq!(atr_string(atr, "RENDER_FILENAME").as_deref(), Some("helicopter.crp"));
    }

    #[test]
    fn events_for_an_ai_element_are_not_the_players() {
        let ev = |kind: u32, who: u32| {
            let mut params = kind.to_le_bytes().to_vec();
            params.extend_from_slice(&who.to_le_bytes());
            params.extend_from_slice(&0u32.to_le_bytes());
            ScriptEvent { kind, params }
        };
        let s = MapScript::default();
        assert_eq!(vehicle_event(&ev(KIND_END_CONTROL_TO_SPLINE, 0), &s), Some(VehicleEvent::EndSpline));
        assert_eq!(vehicle_event(&ev(KIND_END_CONTROL_TO_SPLINE, 3), &s), None);
        assert_eq!(vehicle_event(&ev(KIND_CONTROL_TO_SPLINE, 3), &s), None);
        assert_eq!(vehicle_event(&ev(KIND_CONTROL_TO_SPLINE, 0), &s), Some(VehicleEvent::ToSpline(String::new())));
        assert_eq!(vehicle_event(&ev(41, 0), &s), None);
    }

    #[test]
    fn only_paris_swaps_and_only_to_a_different_name() {
        let sec = |n: &str| match n { "helicopter" => Some("vanquish".to_string()), "loop" => Some("loop".to_string()), _ => None };
        let mut v = PlayerVehicle::spawn("helicopter", SpawnSwitches::default());
        assert!(!v.apply(&VehicleEvent::Transition("london".into()), &sec));
        assert_eq!(v.name, "helicopter");
        assert!(v.apply(&VehicleEvent::ToSpline("CARP::<<Map>>::{AISp}::0".into()), &sec) == false);
        assert!(v.spline.is_some());
        assert!(v.apply(&VehicleEvent::Transition("paris".into()), &sec));
        assert_eq!(v.name, "vanquish");
        assert!(!v.apply(&VehicleEvent::Transition("paris".into()), &sec), "the vanquish has no secondary");
        v.apply(&VehicleEvent::EndSpline, &sec);
        assert_eq!(v.spline, None);
        let mut l = PlayerVehicle { name: "loop".into(), spline: None };
        assert!(!l.apply(&VehicleEvent::Transition("paris".into()), &sec), "same name: no rebuild");
    }
}

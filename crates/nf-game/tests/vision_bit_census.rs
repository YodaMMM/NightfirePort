//! The vision-bit census, pinned: every drone state whose game ENTER writes `+0x4F8 |= 0x10000`
//! ([M] the 137-writer census of ACTION.ELF's `NDrone2_DSTATE_*` functions — each row carries its
//! decompile address and write line) must have the port's `vision_on = true` on that state's ENTER
//! arm. `DroneVision_ProcessDroneSight` 0x176a28 computes `+0x230` only for drones with the bit,
//! and the third loop refuses without it — a state that enters with the bit clear is blind for its
//! whole stay (the class of bug behind level-13-guards-start-alerted rounds 1-3).
//!
//! The state-id → function mapping is the game's own handler table `NDrone2_StateFuncs`
//! ([L] ee_snapshot.bin @0x29b300 + id·4 — 0x00 Global, 0x03 PlayScript, 0x63 GoToGoalPosition, …).
//! The test reads the port's source (no game bytes) and re-derives the arms the same way a
//! reviewer would: every `(STATE, msg::ENTER)` match arm, brace-matched to its body.
//!
//! States the port does not implement yet are pinned in `NOT_IMPLEMENTED`: the arm must stay
//! absent, so the moment one is written the test forces the author to add `vision_on` and move it
//! into `REQUIRED`.

use std::fs;

fn source() -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let drone = fs::read_to_string(format!("{manifest}/src/drone.rs")).expect("drone.rs");
    let kiko = fs::read_to_string(format!("{manifest}/src/kiko.rs")).expect("kiko.rs");
    format!("{drone}\n{kiko}")
}

/// One `(STATES…, msg::ENTER) =>` arm: the consts it covers and its body text.
fn enter_arms(src: &str) -> Vec<(Vec<String>, String)> {
    let lines: Vec<&str> = src.lines().collect();
    let mut arms = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let m = match l.find("msg::ENTER)") {
            Some(p) => p,
            None => continue,
        };
        let open = match l[..m].rfind('(') {
            Some(p) => p,
            None => continue,
        };
        let head = &l[open + 1..m];
        // only plain state lists: identifiers, `::`, `_`, `|`, `,`, spaces
        if !head.chars().all(|c| c.is_alphanumeric() || "::|_, ".contains(c)) {
            continue;
        }
        let states: Vec<String> = head
            .split('|')
            .map(|s| {
                s.trim()
                    .trim_matches(',')
                    .trim()
                    .trim_start_matches("state::")
                    .trim_start_matches("crate::kiko::")
                    .trim()
                    .to_string()
            })
            .filter(|s| !s.is_empty() && s.chars().next().unwrap().is_uppercase())
            .collect();
        if states.is_empty() {
            continue;
        }
        // brace-match the body
        let mut body = String::new();
        let mut depth = 0i32;
        let mut j = i;
        loop {
            let line = lines[j];
            body.push_str(line);
            body.push('\n');
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if depth <= 0 {
                break;
            }
            j += 1;
            if j >= lines.len() {
                break;
            }
        }
        arms.push((states, body));
    }
    arms
}

/// (port state const, decomp cite) — the game's ENTER writes `+0x4F8 |= 0x10000` at that address.
const REQUIRED: &[(&str, &str)] = &[
    // round 2 + earlier: the scripted and the abseil families
    ("PLAY_SCRIPT", "0x15ba68:24"),
    ("IDLE", "0x15bf18:30"),
    ("ALERT", "0x15c248:19"),
    ("PATROL", "0x15c498:19"),
    ("HOSTAGE_KILLER", "0x15c728:19"),
    ("HOSTAGE_KILLER_ATTACK", "0x15c940:22"),
    ("CIVILIAN", "0x15d118:26"),
    ("CIVILIAN_SCARED", "0x15d3a0:26"),
    ("CIVILIAN_HIDING", "0x15d520:17"),
    ("CIVILIAN_PATROL", "0x15d610:26"),
    ("CIVILIAN_MISSION", "0x15d860:22"),
    ("CIVILIAN_MISSION_WAIT", "0x15dbb0:21"),
    ("KIKO_MISSION", "0x15df28:19"),
    ("KIKO_MISSION_RUN", "0x15e0d8:15"),
    ("ENEMY_MISSION", "0x15e1e8:23"),
    // the ally/escort lead + follow chain (this round's additions)
    ("ALLY_LEAD_INIT", "0x15e4f0:24"),
    ("ALLY_LEAD", "0x15e698:21"),
    ("PLAYER_IN_WAY", "0x15ea28:20"),
    ("LEAD_HIDE", "0x15ec08:18"),
    ("ALLY_LEAD_WAIT", "0x15eda8:20"),
    ("LEAD_MISSION_WAIT", "0x15efc8:24"),
    ("LEAD_BOND_COMBAT", "0x15f120:19"),
    ("FOLLOW_INIT", "0x15f3c8:16"),
    ("FOLLOW", "0x15f4e0:21"),
    ("FOLLOW_WAIT", "0x15f6a0:21"),
    // snipers, civilians, truck, interrogation
    ("SNIPER_IDLE", "0x15fab0:16"),
    ("SNIPER_AIM", "0x15fc18:14"),
    ("SNIPER_FIRE", "0x15fd88:25"),
    ("SNIPER_RELOAD", "0x15ff80:16"),
    ("PARTY_GIRL", "0x160140:27"),
    ("CIVILIAN_GUARD", "0x1603a8:21"),
    ("CIVILIAN_DOOR_GUARD", "0x160648:22"),
    ("TRUCK_DRIVER_IDLE", "0x160ed0:19"),
    ("TRUCK_DRIVER_MISSION", "0x161030:22"),
    ("AMBUSH_WAIT", "0x161560:22"),
    ("INTEROGATE_ASSIST_WAIT", "0x161860:23"),
    ("INTEROGATE", "0x161a40:55"),
    ("CIVILIAN_CHALLENGE", "0x162558:16"),
    // surrender / stun / impact recoveries
    ("SURRENDER_ANIM", "0x1626a0:18"),
    ("SURRENDERED", "0x162950:20"),
    ("UNSURRENDER_ANIM", "0x162a70:15"),
    ("STUNNED_RECOVER", "0x1630b8:16"),
    ("STUN_GRENADE_RECOVER", "0x163350:16"),
    ("STUN_DART_RECOVER", "0x163700:20"),
    ("PUNCH_IMPACT", "0x172ee0:16"),
    ("EXPLOSIVE_IMPACT", "0x172fb0:16"),
    // combat / aim
    ("ATTACK", "0x164210:69"), // the game sets it on the no-attack-route arm (`+0xc5 != 2 && != 0x19`); the port's ENTER set is the documented superset
    ("COMBAT", "0x1731b8:16"),
    ("COMBAT_NO_MOVE", "0x1643e8:24"),
    ("COMBAT_OUT_OF_RANGE", "0x164690:22"),
    ("COMBAT_NEW_SIGHTING", "0x173280:22"),
    ("COMBAT_NO_SIGHT", "0x164838:20"),
    ("COMBAT_WAIT", "0x164960:19"),
    ("COMBAT_NO_ROUTE", "0x164c08:20"),
    ("ALERT_TO_POSITION", "0x165270:23"),
    ("GO_TO_GOAL", "0x165640:22"),
    ("SEARCH_AREA", "0x165bf0:20"),
    ("DRONE_STUCK", "0x165ef0:16"),
    ("DRAW_WEAPON", "0x166000:23"), // rides the draw path only (the bail to CivilianScared returns first) — the port places it after the same bail
    ("AIM_STAND", "0x166178:21"),
    ("AIM_STAND_FIRE", "0x166328:34"),
    ("AIM_STAND_RELOAD", "0x1666d8:16"),
    ("AIM_BACKOFF", "0x166b98:17"),
    ("CROUCH_COVER", "0x166d90:17"),
    ("AIM_CROUCH", "0x166f28:16"),
    ("AIM_CROUCH_FIRE", "0x167060:25"),
    ("AIM_CROUCH_RELOAD", "0x167370:15"),
    ("ALT_ATTACK", "0x167488:15"),
    ("STEP_AIM_LEFT", "0x167590:15"),
    ("STEP_AIM_RIGHT", "0x1676a0:15"),
    ("STRAFE_AIM_LEFT", "0x1677b0:19"),
    ("STRAFE_AIM_RIGHT", "0x167980:19"),
    ("STRAFE_DODGE_LEFT", "0x167b50:17"),
    ("STRAFE_DODGE_RIGHT", "0x167cf0:17"),
    ("ROLL_LEFT_CROUCH", "0x167e90:14"),
    ("ROLL_RIGHT_CROUCH", "0x167f80:14"),
    ("SMOKED_OUT_RECOVER", "0x173d30:16"),
    ("INVESTIGATE", "0x168070:19"),
    // route nodes and the scared runs
    ("OPEN_DOOR", "0x1681f8:19"),
    ("KICK_OBJECT", "0x1684c0:15"),
    ("ACTION_ANIM", "0x168628:16"),
    ("STAND_FIDDLE", "0x1687c0:17"),
    ("ENEMY_RUN_TO_POINT", "0x168910:18"),
    ("RUN_AWAY_FROM_OBJECT", "0x168aa8:19"),
    ("HIDE_FROM_SCARY_OBJECT", "0x168d18:19"),
    ("RUN_TO_ALARM", "0x1736f8:15"),
    ("PRESS_ALARM", "0x168ea8:22"),
    ("RUN_FOR_COVER", "0x169150:19"),
    // the rope family
    ("ELEVATOR_JUMPER", "0x169488:16"),
    ("ABSEIL_INIT", "0x169610:19"),
    ("ABSEIL_SLIDE", "0x169730:19"),
    ("ABSEIL_HANG", "0x169940:18"),
    ("ABSEIL_STEP_OFF", "0x1738e0:16"),
    // undercover
    ("UNDER_COVER_INIT", "0x169cc0:15"),
    ("UNDER_COVER_IDLE", "0x169e58:22"),
    ("UNDER_COVER_AIM", "0x16a2d0:15"),
    ("UNDER_COVER_FIRE", "0x16a3e0:31"), // the fire path only (the sniper bail returns first)
    ("UNDER_COVER_SNIPER_FIRE", "0x16a610:21"),
    ("UNDER_COVER_SNIPER_RELOAD", "0x16a8b0:15"),
    ("UNDER_COVER_LEAVE", "0x173ac0:12"),
    ("UNDER_COVER_LEAVE_NOW", "0x16ab38:15"),
    // the noise family (the game's writes are branch-conditional here; the port's arms match)
    ("HEARD_NOISE_AWARE", "0x16af20:21"),
    ("HEARD_NOISE_SUSPECT", "0x16b0e0:43"),
    ("HEARD_NOISE_ALERT", "0x16b540:29"),
    // ninja
    ("NINJA_ATTACK", "0x16bb50:26"),
    ("NINJA_GET_CLOSE", "0x16cb70:18"),
    ("NINJA_SOMERSAULT", "0x16d298:15"),
    ("NINJA_BACKFLIP", "0x16d430:16"),
    ("NINJA_SIDEFLIP", "0x173e08:21"),
    ("NINJA_SIDEFLIP_LEFT", "0x16cdd0:16"),
    ("NINJA_SIDEFLIP_RIGHT", "0x16cf68:16"),
    ("NINJA_STAND_FIRE", "0x16d5b8:21"),
    ("NINJA_NO_ROUTE", "0x16d878:24"),
    // astronaut / space
    ("ASTRONAUT_LAUNCH", "0x16dd28:17"),
    ("ASTRONAUT_COMBAT", "0x174458:16"),
    ("ASTRONAUT_COMBAT_MOVE", "0x174550:18"),
    ("SPACE_DRAKE", "0x16dee0:18"),
    // bullet impact is handled in the impact intercept, not a plain arm
    ("BULLET_IMPACT", "0x173098:18"),
    // grenade throw's ENTER lives in `handle_grenade_throw`
    ("GRENADE_THROW", "0x173480:15"),
    // AllyGoToGoalPosition sets the bit on UPDATE (case 3), not ENTER [M 0x15f938:20]
    ("GO_TO_GOAL_ALLY", "0x15f938:20"),
];

/// Writer states the port has no handler for yet ([M] each is a census writer). When one gets an
/// arm, this list fails — move it to REQUIRED with `vision_on = true` on the new ENTER.
const NOT_IMPLEMENTED: &[(&str, &str)] = &[
    ("CASTLE_CHAT_GUARD1", "0x1612e0:21"),
    ("INTEROGATE_WALK", "0x162338:20"),
    ("COMBAT_TOO_CLOSE", "0x173338:12"),
    ("OBSTRUCTED", "0x173390:14"),
    ("AIM_STAND_DISCARD", "0x166840:15"),
    ("PRONE", "0x166940:14"),
    ("PRONE_FIRE", "0x166a28:27"),
    ("HOLD_IT_RIGHT_THERE", "0x173570:14"),
    ("SEEN_DEAD_BODY", "0x16acd0:16"),
    ("SEEN_SURRENDERED_DRONE", "0x16ade0:17"),
    ("NINJA_STAND", "0x16b990:18"),
    ("NINJA_ATTACK_LONG_RANGE", "0x16bfc0:19"),
    ("NINJA_ATTACK_MID_RANGE", "0x16c248:27"),
    ("NINJA_ATTACK_SHORT_RANGE", "0x16c650:20"),
];

#[test]
fn every_game_vision_bit_writer_enter_sets_vision_on() {
    let src = source();
    let arms = enter_arms(&src);

    let mut failures = Vec::new();
    for (state, cite) in REQUIRED {
        match state_special_ok(&src, state) {
            Ok(true) => {}
            Ok(false) | Err(()) => {
                // fall through to the arm scan for plain states
                match arms
                    .iter()
                    .find(|(states, body)| states.iter().any(|s| s == *state) && body.contains("vision_on = true"))
                {
                    Some(_) => {}
                    None => failures.push(format!(
                        "{state}: the game's ENTER sets `+0x4F8 |= 0x10000` [M {cite}] but the port's ENTER arm does not set vision_on"
                    )),
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the vision-bit census failed for {} state(s):\n{}",
        failures.len(),
        failures.join("\n")
    );

    let mut ghosts = Vec::new();
    for (state, cite) in NOT_IMPLEMENTED {
        if arms.iter().any(|(states, _)| states.iter().any(|s| s == *state)) {
            ghosts.push(format!(
                "{state}: now has a port arm — add the ENTER `vision_on = true` ([M] {cite}) and move it to REQUIRED"
            ));
        }
    }
    assert!(
        ghosts.is_empty(),
        "previously-unimplemented census writers grew arms:\n{}",
        ghosts.join("\n")
    );
}

/// The three states whose ENTER bit does not live in a plain `(STATE, msg::ENTER)` arm.
fn state_special_ok(src: &str, state: &str) -> Result<bool, ()> {
    match state {
        "BULLET_IMPACT" => Ok(window_has(&src, "if s == BULLET_IMPACT && matches!(m, 0 | msg::ENTER | msg::UPDATE)", "vision_on = true", 4)),
        "GRENADE_THROW" => Ok(window_has(&src, "if s != GRENADE_THROW { return None; }", "vision_on = true", 8)),
        "GO_TO_GOAL_ALLY" => Ok(window_has(&src, "(GO_TO_GOAL_ALLY, msg::UPDATE)", "vision_on = true", 6)),
        _ => Err(()),
    }
}

fn window_has(src: &str, anchor: &str, needle: &str, lines: usize) -> bool {
    match src.find(anchor) {
        Some(p) => src[p..].lines().take(lines + 1).any(|l| l.contains(needle)),
        None => false,
    }
}

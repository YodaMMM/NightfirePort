# mp-bot-brain-rest — the MP bot decision helpers (BOTSTATE / BOT / BOTWEAP / MP object+sound)

The call-tree check (`tools/calltree/calltree.py`, master 20874c31) listed 25 reachable
functions the port never mentioned. This task ports or cites each one. Everything here is
`crates/nf-game/src/mp/bots.rs` (`BotBrain`/`BotVars`), `botweap.rs`, `mp/mod.rs` and one
doc-cite in `nav.rs` — data, state, flow and getters only; nothing draws.

## Ported into `BotBrain` (bots.rs section 9)

| Game function | Address | Port shape |
|---|---|---|
| `BOT_validateStateChange` | 0x124fb0 | `validate_state_change` — the full gate chain (impact refusals 0xEF/0xF0/0xF1, the 0xEE +0x76F latch, type 0xC/6 refusals, the recovery-flag 0xD0 block, the 0xEB seen/distraction arms, the type-4/7/0xC and type-10 arms, the 0xD0/0xD6 → `reallyWantACombatMove` replacement). Wired at the pending commit in `step` (`Drone_SM_SetState` 0x1758c0:47-52's gate). The +0x56C anim-band gate on 0xDE is taken false — the port's MP bot body runs no anim ids [S]. |
| `BOTSTATE_startRecovery` | 0x1299d0 | `start_recovery(now)` — deadline := now + recovery_b·2/3, flag bit1 gates a re-arm. `Drone+0xB9` is the stats' recovery_b (`BOT_setDroneStats` 0x126318:22) [M]. |
| `BOTSTATE_isDistracted` | 0x129c58 | `is_distracted()` — the +0x734 bit-2 clear, no-goal / non-0xEB / non-zero-sum / limit-reached chain [M]. |
| `BOTSTATE_increaseDistraction` | 0x129bb8 | `increase_distraction(amount)` — accumulates into +0x728 against the active goal's +0x20 limit; returns false below the limit (the game's own early returns), `(flag & 4) == 0` at it [M]. |
| `BOTSTATE_reallyCloseCombatRange` | 0x129b68 | `really_close_combat_range()` — [0.5, 1.5, 1.625] into Drone+0xEC/0xF0/0xFC [M]. |
| `BOTSTATE_defaultCombatRange` | 0x129b98 | `default_combat_range()` — restores from `BotVars.combat_range_default` (+0x71C/0x720/0x724) [M]. |
| `BOT_postLoadInit` | 0x1262b8 | folded into `bot_init`: the stats copy is `BOT_setDroneStats` 0x126318's (field map in the comment), and the combat ranges save into `combat_range_default` [M]. The Drone+0x4F8 \|= 0x80000000 bot mark has no port field (the brain is always a bot) [S]. |
| `BOTSTATE_chooseUnarmedAttackAnim` | 0x1292f0 | `choose_unarmed_attack_anim(rng)` — `Rand(4)`, the `NDrone2_CanAltAttack` 0x1549e0 1.5 reach gate, the 0x3D unarmed attack [M]; `CanDoAnimState` taken true [S — no anim pipeline]. |
| `BOTSTATE_reallyWantACombatMove` | 0x1267a0 | `really_want_a_combat_move(ctx, rng, mask)` — the five `Rand` orderings over 0xD4/0xD5/0xDB/0xDC gated by `can_side_move` (`checkAttackMove` 0x126a40's shape), the mask-bit-2 0xD7 arm [M]; the `NDrone2_EvasiveMove` roll codes (0x79/0x7a → 0xD9/0xDA) are not lifted [S]. |
| `BOT_handleOpponentHistory` | 0x1257c0 | `handle_opponent_history(ctx, candidate)` — the 16-ring (+0x6DC/+0x76C), later-duplicate scoring, dead-row pruning, the ×5 current-opponent weight, argmax → `SetOpponent` (+0x274 := 0, +0x270 := 1); a no-score tail passes the candidate through [M]. Not yet wired into `find_opponent` (the port's own scan stands, documented [*S* port choice] there) — the wiring is the report's next step. |
| `BOTSTATE_isPathWithinObjectRange` | 0x128f80 | `is_path_within_object_range(ctx, centre, radius, max_segs)` — the route-segment vs sphere walk (`segment_sphere_hit`, the `Vec_LineSphereIntersection` 0x1591d8 stand-in [S]); `pickGoal`'s call passes radius 4.0 / 4 segments around the opponent [M 0x126b88:525]. Not wired into `pick_goal_scan` yet (the pref-byte bit-3 verify arm is not lifted there). |
| `BOTSTATE_stopAnim` | 0x129918 | `stop_anim()` — a documented no-op: the port's MP bot body runs no anim scripts to tear down [S]. |
| `BOT_soundEffect` | 0x126618 | `bot_sound_effect(trigger)` — the bark trigger table (1 → 0x152+Rand(3), 0x1E → 0x152+Rand(2), 0x1F → 0x14D+Rand(5) behind the Playing_Ref gate) [M]; the port's MP bots have no bark channel (bot sound is presentation) [S]. |

## Ported into `botweap.rs`

| Game function | Address | Port shape |
|---|---|---|
| `BOTWEAP_getWeaponAnimSet` | 0x12b140 | `weapon_anim_set(w)` — the full switch [M]. |
| `BOTWEAP_getWeaponClipSize` | 0x12b010 | cited at `BotWeaponRow.clip` (+0x92) — the table lift already reads `weapon_data + w·0x10C + 0x92` [M]. |
| `BOTSTATE_hasWeapon` | 0x129658 | cited at `BotArsenal::has_weapon` — the thin forward to `BOTWEAP_hasWeapon` (+0x754) [M]. |

## Ported into `mp/mod.rs`

| Game function | Address | Port shape |
|---|---|---|
| `MPSound_Play` | 0x18bee0 | `mp_sound_play(sink, id)` — `Sound_Play(100.0, id, 0, 0)` [M]; the port's `Sounds::play` has no volume parameter (presentation) [S]. |
| `MP_sendTeamBotMessage` | 0x18bbc8 | `send_team_bot_message(...)` — the ~team mask (0 for sender 2) and the GameState+0x52 payload [M]; the cross-bot pump wiring is not lifted yet [S]. |
| `MP_resetPickupBotVisitTimes` | 0x18be50 | `reset_pickup_bot_visit_times(pickups, bot)` — the per-bot visit column zeroes across rows [M]. |
| `MP_UnregisterPickup` | 0x18a058 | `unregister_pickup(pickup, registered)` — the registered gate, the row memset, the count decrement [M]. |
| `MP_getObjExtObj` | 0x18bca8 | `get_obj_ext_obj(current, script_obj)` — the script-obj replacement decision (the msg-0x40 route rides `BotGlobalOutcome::ObjectiveSite`) [M]. |
| `MP_objectBeingDeleted` | 0x18ac90 | `object_being_deleted(objs, obj)` — the row-clear sweep + the msg-0x3D route note (the port's mode rows are the per-store obj ids) [M]. |
| `MP_Pickup_PostLoadInit` | 0x18b578 | cited at `nav::Network::emit` — the port of `AINetwork_EmitPath` 0x158cf8, which the game's pickup/ext post-load init calls [M]. |
| `MP_PostLoadInitObjExt` | 0x18b848 | same cite at `nav::Network::emit` [M]. |

## New BotVars fields (own lines, offsets cited)

`distraction` (+0x728), `combat_range_default` (+0x71C/0x720/0x724), `opp_history` (+0x6DC,
index+1 encoding), `opp_history_head` (+0x76C), `recovery_b` (+0xA9).

## Tests

`crates/nf-game/src/mp/bots.rs` `brain_tests`: `brain_rest_start_recovery_gate`,
`brain_rest_distraction_accumulates`, `brain_rest_combat_ranges`,
`brain_rest_unarmed_reach`, `brain_rest_weapon_anim_set`,
`brain_rest_opponent_history`, `brain_rest_validate_refuses`,
`brain_rest_stop_anim_and_barks`, `mp_rest_pickup_visit_reset`,
`mp_rest_team_msg_and_unregister` — each with a sister assertion that can fail. All 358
`nf-game` lib tests pass with the validation wired into the live commit path.

## Not wired yet (explicit, for the next task)

- `handle_opponent_history` at the `find_opponent` tail (the port's scan overwrite stands).
- `is_path_within_object_range` inside `pick_goal_scan`'s pref-byte bit-3 verify arm.
- The cross-bot message pump (msg 0x44 team route, msg 0x3D/0x40 object routes).

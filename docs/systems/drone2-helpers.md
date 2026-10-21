# NDrone2 helper helpers — the call-tree remainder (task ai-ndrone2-rest)

The call-tree check (`tools/calltree/calltree.py`, `--all-mentioned-dstates --list`) flagged 18
`NDrone2_*` helpers reachable from the ported guard states that the port never named. This note
records, for each: what the game does [M], who calls it, and where the port lands it — ported
into the caller, cited at the Rust that already does the work, or pinned as a stub. After the
task the check lists none of them.

Evidence tags per `docs/fields/README.md`: [M] decompile (`work/decomp/ACTION.ELF/`, `:NN` =
line), [L] live measurement (PCSX2 playtest), [S] suggested/port choice.

## Ported into the caller

### `NDrone2_NearArmedBond` 0x178678 — the driver's armed-Bond scare
The only caller is `TruckDriverIdle` case 3, which passes **6.0 and the arg is LIVE**:
`arg <= +0x1a0` (the opponent distance) → 0 [M 0x160ed0:27, 0x178678:12-14]. The rule:
prop 0x3c AND prop 0x51, `+0x1a0 < arg`, the seen bit `+0x228 & 4`; then levels
0x7000009/0xa → yes (the `+ 0xf8fffff7 < 2` unsigned fold), else
`DroneWeap_OpponentHasWeapon`, level 0x7000007 falling to `NDrone2_ObjectIsArmed`
0x178768 [M 0x178678:6-38]. The civilian states use the DIFFERENT predicate
`NDrone2_NearArmedPerson` 0x1787c8 (no distance gate, voice-0xD exempt) — the port had wired
the driver to that one; fixed.
- Port: `Drone::near_armed_bond` (drone.rs) + the `TRUCK_DRIVER_IDLE` UPDATE arm.
- **Prop 0x51 (corrected in the 2026-10-21 review of 844)**: the 07000006 driver's live behaviour
  block is the placement's normal block — placed drones take `DoModeSettingsNEW` 0x14c600, which
  fills both blocks from the placement words [M 0x14c600:61-70]; `DoModeSettingsOLD` and the
  `DroneModeSettings` init rows (`init_DMODE_CivilianGuard` 0x14e158 included) are unreachable for
  placements [D census; docs/port-map/unreached.md]. His normal block has 0x3c but **no 0x51**
  (only the alert block carries 0x50/0x51) [D `nf-extract drone-props … 07000006.bin`, (66.1, 39.2,
  28.9) k15 13]. So the game's NearArmedBond is 0 while he idles: armed Bond alone does not scare
  him. 844 had forced 0x51 for mode 0x17, citing the playtest observation "he does get scared" —
  that observation was of the port, not the game. No runtime writer of prop 0x51 exists outside
  the init rows [M grep setProperty(0x51)].
- Test: `nf-view/tests/fix_06_driver_live.rs::the_scared_cower_keeps_him_in_place_on_the_floor`
  — armed Bond 4 units in the idling driver's cone → he stays idle; the scared chain entered directly cowers in place (was: CivilianScared/Hiding through the REAL
  input path (`nf_view::load` + `Sim::step_bond`); sister `gunfire_never_arms_the_driver…`
  (the scare is not a shot: the gunfire arm still kills him through the shared death path and
  the guards still react). Both green.

## Cited at the Rust that does the same work (drone.rs)

| Game helper | Address | The port's site |
|---|---|---|
| `NDrone2_UpdatePatrolRoute` | 0x1514d8 | `Drone::update_patrol_route` (the doc comment now carries the address and the game's anim pick 0x35/0x21 by prop 0x25 + `+0x50c` ≥ 0.66; the port's `alerted` fold picks ALERT_WALK) |
| `NDrone2_SetAngleToAlert` | 0x154e90 | `HEARD_NOISE_SUSPECT` ENTER/TURNED — `+0x694` = atan2 to the alert pos, gated on the own-sight bit; the alert pos `+0xaf0` is the sound-alert position (`DroneFunc_HandleSoundAlerts` 0x148a30:77) = the port's `sound_pos` |
| `NDrone2_GetAngleDifferenceToAlert` | 0x154f00 | same ENTER — `|Δ| > 2.3561945` (3π/4) turns with anim 0x61/resume 0x23 [M 0x16b0e0:44-49] |
| `NDrone2_SetAngleToDest` | 0x154e20 | `mp/bots.rs` BotAttackRunChangePosition row (0x16fb20:37; phase 4) |
| `NDrone2_ChangeToAttackMode` | 0x14a648 | the `ATTACK` ENTER alt-state arm — the game's helper flushes the pending resume `+0x5a4` (promotes `+0xc6`→`+0xc5`, ninja 2→0x19, the 2nd-behaviour switch on `+0xc48`==100, returns the old `+0x5a4`) [M 0x14a648:7-36] |
| `NDrone2_DroneInSights` | 0x14a800 | the `ATTACK` ENTER AimStand pick — a size-8 stub returning 0 [M 0x164210:79-84], so prop 0x18 + in-range → AimStand 0x67 unconditionally, resume 0x68 |
| `NDrone2_HostageSituationActive` | 0x14aaf8 | the `HOSTAGE` doc — a size-8 empty body whose result the caller DISCARDS (`call; return 1` [M 0x15cb78:30-33]) |
| `NDrone2_HasOpponent` | 0x14ad58 | the `NO_OPPONENT` stand-in note — opponent `+0x170` != 0 OR the last-opponent tick `+0x238` (`DroneVision_HaveOpponentSight` 0x175ef0:66-67) within 30 ticks [M]; the port's `ever_seen` latch is the coarse fold [S] |
| `NDrone2_SetOpponentAimPos` | 0x141fe8 | `Drone::opponent_info` — the game's `+0x210` aim point (tracked bone, the 0.2/0.6 y-lift, the 2/7 smoothing, re-run from `DroneWeap_NextBulletTime` 0x179d40:130); the port stands in the player's aim-target snapshot [S fold] |
| `NDrone2_Player` | 0x149768 | the `Drones::tick` opponent pass — `glb_players` or the MP active-player global [M 0x149768]; the port has one Bond |
| `NDrone2_HeadTrackObj` | 0x14ada0 | the per-tick head-tracking block — the Bond-talk request behind the prop-0x4c gate: `AnimObjectSetHeadTracking(3.0, 0.13, …)` [M 0x14ada0:9-15]; the port's tracking covers it with the same 3.0 range |
| `NDrone2_DoTypeSettingsOLD` | 0x14d2e8 | the abseil dtypenew comment (full name now at the 0x14b300:676 site) |
| `NDrone2_CreateNinjaEyes` | 0x149600 | the `NinjaStand` 0xa6 ENTER doc — two gfx objects (hash 0x2000132, bones 0x54/0x60) spawned by `ControlDTYPE_Ninja` 0x14b0f8:6 / `PreDroneControl` 0x148db0:126 [M]; visual-only, the AI reads the eyes datum for the BLIND_DRONES cheat arm only — the main agent renders |

## Cited at the caller (mp/bots.rs — the MP data layer, phase 4 ports)

| Game helper | Address | Caller |
|---|---|---|
| `NDrone2_InShoutingRange` | 0x1788a0 | `BotGlobal` 0x16e060 → `NDrone2_ReactToDroneAlertMsg` 0x1437a0:181,227 — dist < 15 yes / 15..20 same cel / ≥ 20 no; level band 0x700000c..f: 40/100 [M 0x1788a0] |
| `NDrone2_SendAttackMessage` | 0x14a410 | `BotAttack` 0x16f198 UPDATE — team 1, `+0x4F8 & 0x200000` clear, opponent set → `+0x4FC \|= 0x10`, `DroneAlertToObject(10, 10, 1, msg 0x16, opp)` [M 0x14a410] |
| `NDrone2_SetAngleToDest` | 0x154e20 | also the BotAttackRunChangePosition row (above) |

## Never delivered on the ported paths

- **`NDrone2_ObstructedPatrolPath` 0x14ad50** — a size-8 function returning the state id 0x61
  (`NDrone2_DSTATE_Obstructed` 0x173390) [M]. `DSTATE_Patrol` 0x15c498:95-97 calls it for msg
  0x1b; no decompiled sender of msg 0x1b exists [M grep] — the port has no 0x1b arm, cited at
  the `PATROL` UPDATE.

## Tests run for this task

- `nf-view --test fix_06_driver_live` (both tests, the scare + the gunfire sister) — green
  after the fix, red on the scare before it (the driver stayed 0x34: prop 0x51 missing).
- `nf-game` lib + integration, `nf-formats`, `nf-audio`, `nf-view` — see
  `docs/reports/ai-ndrone2-rest.md` for the exact totals.

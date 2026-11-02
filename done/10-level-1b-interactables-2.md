## Your task: `level-1b-interactables-2`, 1b: every player-touchable kind behaves like the game

`interactables_0700001b_live.rs` has 7 tests. Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player triggers), run the census of 0700001b [D every placement kind], and for every kind a player can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_0700001b_live, walkreal_0700001b_live and nf-game; quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-02T01:30Z

---
## Report (done, 2026-11-02T03:40Z)


---
## Report from the worker (done, 2026-11-02T03:40Z)


---
## Report from the worker (done, 2026-11-02T03:41Z)

# level-1b-interactables-2 — report

## What's done

The 0700001b kind→test matrix is complete. `interactables_0700001b_live.rs` went from 7 tests to **9** (all passing), one per player-touchable placement kind on the space station:

| Kind (the §7 census) | Test | Status |
|---|---|---|
| SpaceMissile 0x39 + the console SP chain | `missile_console_shots_latch_and_the_launch_fails_on_ch_99` | existing, passing |
| AI drones (Drake, the hub guard) | `drake_defeat_by_real_fire_ends_the_mission`, `the_hub_guard_dies_to_fire`, `the_spawner_refuses_a_template_on_top_of_bond` | existing, passing |
| Destroyables 0xE1 | `the_hub_destroyable_shatters_under_fire` | existing, passing |
| Bond-bonus pickups (kind 6) | `the_bonus_pickup_collects_on_the_walk_over` | existing, passing |
| Hint marker 0x33 | `the_spawn_hint_fires_on_the_first_walk` | existing, passing |
| **Sound triggers 0xF9 (5 × AluWarning)** | `the_launch_order_and_the_disarm_speech_fire_the_pit_klaxons` | **NEW** |
| **Hurt volume 0xE4 (OBJECT_MissileFlame #17)** | `the_missile_flame_hurts_on_contact` | **NEW** |
| SpaceLaser 0x38 | `space_laser_1b_live.rs` (4 tests, from level-1b-space-laser) | covered in its own file |
| AND muxes (kind 0xec, 7) | walkreal's `the_first_wave_kill_raises_the_launch_order` (firing side) + the new test's objective-AND sister | covered |
| ScriptPlayers 0xD9 (the G-047 solar panels' 20-HP damage tail) | the console shots' mechanic (test 1) | same kind, covered |

Not player-touchable (no test owed): the Apocalypse 0x4A FX, the DynObject 0x3B, the sky 0x2A, the start 0x24. The station has no doors/locks/switches/monitors/fuseboxes/sensors/grapple/vents/SimpleScripts [D the §7 census].

## The two new tests, with their evidence

**`the_launch_order_and_the_disarm_speech_fire_the_pit_klaxons`** — the 0xF9 kind. I decoded the level's script streams from the disc (`census_0700001b::dump_0700001b_scripts`, cross-checked against `space_missile.rs`'s P2_WRITES) and found the five AluWarning klaxons' ON channels 0x4c..0x50 are **conditional event-11 writes** — `Script_EventHandler` 0x1d94c0 case 0x11 writes `[cond, ch_if_on, ch_if_off, val]`'s second channel only while the condition channel reads OFF [M; the port's nis.rs 0x11 arm] — behind a speech chain: the missiles' p2 disarm scripts write ch 0x53/0x54/0x55/0x59 [D P2_WRITES], whose rise starts SP#152–155 (mask 1 = SP_Hit 0x1dc470:11-13's own-out-channel bit [M]), whose speech scripts 0x060009a5–0x060009a8 end in the klaxon writes (t900/t1080/t1560/t3000 [D]); SP#201's 0x06000a31 (mode 1, gate p9 = 19 — the launch order) writes ch 0x4c at its t1500. The klaxon conditions are the pair muxes [6&8]→91 and [1&2&7]→92 plus the raw console channels — **the klaxons warn about the missiles still armed**. The test drives the launch-order mux's inputs (193/194 — the mux itself then writes ch 19, its own real write) and consoles 3–6 every tick (the documented stand-in; the real writer is the shot screen's t682 write, pinned for consoles 1/7 by the missile test): 0x4f and 0x50 rise, latch, and 0x4f's trigger plays; 0x4c/0x4d/0x4e stay silent as the armed-missile sisters; the objective's AND #139 out stays off with m1/m2/m7 armed; the m8 launch branch fails the mission mid-run and the chain co-exists with it.

**`the_missile_flame_hurts_on_contact`** — the 0xE4 kind. The ungated OBJECT_MissileFlame #17 (p0 = 2 the pain type, p1 = 0x32 the damage 50, gate 0, who 0 [D audit:444]) starts at once [M Hurt_Update 0x17e5d0:51-102]; the damage rides the player's object contact — `Player_DealWithObjHit` 0x194458's class-0x27 arm → `Player_HandlePain` [M], pain types 1..7 take no armour [M 0x1902c8:84-104]. The test floats the load's own zero-g substate 8 into the volume's own collision AABB (the placement's object collision [M Hurt_Create 0x17e870:44]; the west-shaft transit is the documented stand-in) and the health drops 50 × 0.6 = 30 [M damage_factor's 0x1b × Easy row]. Sister: the stand-off leaves the health and the volume's contact witness untouched.

## Port gaps found

**None.** Every law the new tests exercise (the SP own-out start law, the event-0x11 conditional write, the SoundTrigger create param map [M 0x1b2628:24-48] and update lifecycle [M 0x1b2790], the hurt contact delivery) already matched the decompiles. No port code changed.

## Verification

- `interactables_0700001b_live`: **9 passed / 0 failed** (28.6 s).
- `walkreal_0700001b_live`: **17 passed / 0 failed** (125.6 s).
- `nf-game`: **653 passed / 0 failed / 5 ignored** (1.05 s).
- `nf-audio`: **23 passed / 0 failed**.
- `nf-formats`: **105 passed / 3 failed** — the 3 are exactly the `disc_golden` digest tests the driver's rule allows on the slim ISO.
- The fast full-suite runner (`tools/test/suite.py`) could not complete in this sandbox: the sandbox disk is 9.9 GB total and the full nf-view test-binary build needs ~5 GB of `target/release/deps` on top of the rest of the workspace; it hit "No space left on device" twice even after pruning stale artifacts. The four crate runs above are the same builds the runner drives (its per-crate `cargo test` invocations), executed directly.

## What's left / live checks for the main agent

1. **The second klaxon's play in the real game**: headless, the audio driver has no output device, so nothing reports a one-shot's end (`Sound::batch`'s `take_ended` [M sfx.rs]) — the FIRST klaxon's GlobalVoiceHandle holds forever and the second klaxon (0x50, SP#155's t3000) exercises the busy-kill arm [M 0x1b2790:19-28]. In the game with a live IOP, the first klaxon ends after its bank length and the second should PLAY. Check: during the missile-disarm run with consoles 3–6 disarmed and m1/m2/m7/m8 armed, both AluWarning klaxons (ids 0x36f, 0x34d) should be audible at the pit bottom, ~2800 ticks apart.
2. The census's old "the 1b deck pick did not fire" note is superseded: the bonus-pickup take path is pinned by the existing test (the take-tail split fix, 2026-10-23).
3. The `space_laser_1b_live.rs` harness poke of the laser's ch-19 arm gate could now be replaced by driving the mux inputs 193/194 the way the new klaxon test does (the mux's own write holds the gate) — cosmetic, not a gap.

## Usage

not visible to me (no token/credit/context figures are exposed to this session).

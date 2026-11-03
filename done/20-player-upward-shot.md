# player-upward-shot — Bond's rounds die on upward shots at range

Branch from origin/master.

level-0c-yard-climb-real-health (blocked; its report is on origin/s/level-0c-yard-climb-real-health) found that on 0700000c Bond's aimed rounds at the deck snipers (weapon 0x1e drones ~33 u away and above him) never land: three clips dry, target hp unchanged, strict gun rays true, while level shots at the same range work.

Do:
1. Trace one upward shot through the port's bullet path (flight segment, the per-tick HITTEST, Collide_Pick straddle cels, the lip geometry) and log where the round stops.
2. Compare each step with the game [M Bullet_update 0x12e7b0, Collide_Pick 0x1eb320, the cel straddle 0x1eac98]: a round that the game lets through but the port stops (wrong cel scope, a wrong tri, a range cap) is the bug — fix it and cite it. If the game's geometry stops it too, say so with the tri/placement that does.
3. Add a test that fires up at a deck sniper from the yard and pins the outcome; run every bullet/decal/stairwell test, walkreal_0700000c_live, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets, and the code must check it) and you must run EVERY walkreal_*_live binary before finishing.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox1 at 2026-11-03T01:25Z

---
## Report (done, 2026-11-03T03:25Z)


---
## Report from the worker (done, 2026-11-03T03:25Z)

# player-upward-shot — report

Branch `s/player-upward-shot` (cut from origin/master 6f16146e).

## What's done

**The verdict: the port's upward bullet flight is NOT the bug — the harness's stands fired the
HIP act, and the sniper's hip cone IS the miss.** The aimed act lands and kills, and now ships
pinned through the real input path.

1. **The trace** (`crates/nf-view/tests/upward_shot_trace_live.rs`, kept — a documented
   measurement-spot teleport at the yard ladder base): the aimed weapon-0x1e round at the deck
   sentry #18 (36.4, 12.06, −208.79, d 34.9, ~10 u up) was walked tick by tick. The flight's cel
   scope stays cel 78/77 the whole lane (no `find_cel` → None, so the flat fallback never
   engages), the scoped and the flat picks agree, the lane's world stop sits 4.7 u BEHIND the
   chest box (the wall behind the deck), and the AIMED round impacts
   `Drone { drone: 18, part: 4 }` at (36.16, 12.27, −208.68) — hp 10 → −50. The port's pick law
   matches the game's on this lane [M Bullet_update 0x12e7b0 fills HITTEST+0x80 from the bullet's
   linked cel; Collide_Pick 0x1eb320:125 straddles via Collide_StraddleCels 0x1eac98;
   control_handle_cel_change 0x133288 keeps a class-5 bullet's cel in open air by clamping it
   back into the old cel's bbox — the scope never becomes the whole world; the port's per-tick
   `find_cel` found a cel every tick here so the divergence never fired].
2. **The dry clips explained** [L ee_start_05.bin @ 0x2c10b8, weapon_data rec 0x1e]: spread base
   +0x24 = **100.0** → the hip cone is 0.14 rad = **4.9 u of scatter radius at 35 u** against a
   ~0.5 u chest box; [M Bullet_init 0x12c5a0:139-170 — the player owner's mul is 0 ONLY with
   +0x68 & 0x400000 AND the aim bit; 0x1e carries 0x400000 [L flags68 0x408050]]. The shared
   `stand_and_fire` never set `input.aiming`, so every "aimed" stand fired hip — the game's own
   hip sniper misses the same way. The "level shots work" fights ran at d 7-10 where the same
   cone is ~1.1 u. Three dry clips ≈ 15 draws × (0.5/4.9)² ≈ 0.16 expected hits — zero observed.
3. **The fix is the aimed act, opt-in** (the shared-helper rule): `DamageLog::aimed_fights`
   (default false — zero behaviour change for every other walk), set only by the 0c test.
   `stand_and_fire` under the flag holds `Input { aiming: true }` every tick AND tightens the tap
   gate to 0.01/0.01 (the plain 0.03 rad / 0.04 norm = ±1.05 u / ±2.2 u at 35 u is wider than the
   chest box; the zero-spread round goes exactly along the view — the trace hit with an exact
   view, the loose gate dried clips on the same box).
4. **The real-input proof** (`crates/nf-view/tests/upward_shot_0700000c_live.rs`, both green):
   `walkreal0c.rs` legs 1-6 (the walkway start #1 [D] → the laser-watch burn → the vent drop →
   the staircase descent → the plates chain → the open secret door → the yard → the ladder base)
   were factored into `walk_spawn_to_the_yard_ladder_base` (the band walk calls it and continues
   at leg 7 — no behaviour change; the file is 0c-only, its two consumers both re-run green).
   - `the_hip_pull_dries_three_clips_on_the_deck_sentry` — THE SYMPTOM, pinned: the walk reaches
     the ladder base on real input, the number-key switch to the sniper 0x1e [M Player_WeaponChange
     mode 0; owned, the boot loadout], the hip stand dries its rounds, hp 10 → 10.
   - `the_aimed_upward_round_lands_and_drops_the_deck_sentry` — THE FIX, pinned: the same stand
     with `aimed_fights` drops the emplacement (the pair #18/#36 occupies one spot [L the
     census]; a round into the exposed box hits whichever twin the ray meets first [M
     Drone_CollisionHandler 0x140968 — the game's bullets hit any drone]) — states [(−50.0, true)].
5. Also pinned en route: `Sim::step_bond`'s 2nd arg is the TRIGGER (the probe_0d_bullet
   `!clip_empty` convention) — my first aimed runs passed `false` and never fired.

## How it was verified

- `upward_shot_0700000c_live`: 2 passed. `upward_shot_trace_live`: 1 passed.
- The bullet/decal/stairwell group (task requirement): `bullet_range_live` 3, `decal_order_live` 1,
  `decal_door_follow_07000011_live` 4, `stairwell_bullets_07000011_live` 2,
  `stairwell_decal_interp_07000011_live` 2, `stairwell_ghosts_07000011_live` 3,
  `stairwell_guards_07000011_live` 3, `stairwell_walls4_scope_07000011_live` (1 ign),
  `stairwell_walls_07000011_live` 2, `bullet_pass_probe` (2 ign) — all green.
- EVERY `walkreal_*_live` binary (the common-helper rule — damage_log.rs is shared): 77 binaries
  — the 25 `walkreal_0700*_live`, `walkreal_celprobe_live`, `walkreal_dump_live` and all 50
  `walkreal_probe*_live` — **all green** (incl. both 0c consumers of the split walk).
- `nf-game`: 1040 tests passed, 0 failed (101 binaries). `nf-formats` + `nf-audio`: 136 passed,
  3 failed = exactly the 3 allowed slim-ISO `disc_golden` tests.

## What's left

1. The rest of `nf-view`'s ~800 non-walkreal test binaries cannot fit this sandbox (the standing
   disk limit — same constraint the blocked 0c task documented). My changes touch no `src`, so
   those binaries carry master's own state; the walkreal family that exercises every
   `damage_log.rs` change ran green.
2. The blocked `level-0c-yard-climb-real-health` follow-up can now land: its `yard_sniper_stands`
   should set `w.dmg.aimed_fights = true` (the flag this task added) and un-ignore the
   real-health sister — the aimed stands should drop #18/#15 from the base and the climb runs on
   a clear field.
3. A code-level divergence NOT exercised by this repro, left named for the next session: the
   port's per-tick `find_cel(b.pos)` falls back to the FLAT whole-world `segment_bullet` when a
   round leaves every room-cel bbox (world.rs:1448), while the game keeps the bullet's LINKED cel
   and clamps it back into the old cel's bbox in open air [M control_handle_cel_change 0x133288:186-232
   — class 5 = the `default:` arm; Bullet_init never sets obj+0xf0, so flag 0x20 is clear]. On this
   repro's lane the scope never went None (the yard cel is tall enough to hold the deck lane), so
   nothing here is fixed on that account — but a shot that climbs OUT of every bbox (e.g. into the
   open sky above the yard) would test backdrop tris the game never tests. A faithful fix is the
   sticky cel (+ the clamp), not the flat fallback.
4. Live checks for the main agent:
   - PCSX2, 0700000c: from the yard floor near the ladder base, aim (scope) at the deck sentry
     (36.4, 12.1, −208.8) and fire — expect the kill in 1-2 rounds (the aimed act's zero spread).
   - Same spot WITHOUT aiming (hip): expect misses around the sentry at ~5 u scatter — confirm the
     game's hip cone misses too (the "three dry clips" was never a port gap).
   - Optional: fire a round into the open sky above the yard and watch the tracer — the game's
     clamp law predicts the round visually stalling at the last cel's bbox edge until it dies
     ([M 0x133288] the position write); the port draws it flying on (no clamp) — a divergence the
     renderer would show.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).

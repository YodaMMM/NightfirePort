# level-06-audit — the 07000006 behaviour audit (report)

Branch `s/level-06-audit`, head `35a9f2ed` (+ this report). The deliverable is
`docs/levels/07000006-audit.md`: **one row per placement for all 813 main-map placements** (30
kinds, re-dumped from the disc this session via `census_07000006.rs --ignored`), each with the
game's [M/D] chain, the port's side (file:line), and a GAP column — plus the level's script table
(all 4 live SPs + the special-death pair + the camera/hand-back law), the mission record, the doc
corrections, the large-gap write-ups and the live checks.

## What's done

1. **The audit table** (`docs/levels/07000006-audit.md`): every placement row carries its census
   params [D]; every kind carries the creator/update/handler chain from the decompile
   (`parsemap_create_dynamic_objects` 0x1d04f0 + per-creator addresses) and the port's spawn/update/
   draw/use/sound status. Kinds ABSENT on 07000006 are named (no ladders 0x22, grapple 0x3A,
   switches 0x29, sensors 0x28, monitors 0x30, lifts 0x27, kind 0xF8).
2. **Small gaps fixed (each with a test):**
   - **The SS operate sound** — `SS_Update`'s 0x40→1 arm plays `Sound_Play3D(…, ext+0x14, obj+0x80)`
     [M 0x1a9948:86-89]; the port skipped it explicitly. Now `SsList::pending_sounds` emits
     (sample, object-matrix translation) on the arm and the Drones tick drains it into the audio
     sink (the lock's `pending_sounds` routing). 06 rows: barrel traps #353/#354/#389/#390 carry
     sample 0x11b = 283 [D census]. Test: `ss_operate_sound_06_live` (green).
   - **The nis.rs light-boot comment** — it claimed "no param carries the hash" for 0x06000066; the
     census proves SP placement #808's p0 IS 0x06000066 [D]. Comment rewritten (no behaviour change;
     the synthetic boot + the placement player both stay, the beat stays a live check).
3. **Corrections recorded** (audit doc §"Corrections"; `docs/levels/07000006.md` deliberately
   untouched — no shared-file rewrites): spotlight TouchOnces output ch 52/56 not 57/56 [D census];
   door p5/p6/p7 are open/close/locked SOUND ids [M 0x136100:25-70]; pickup p4 is the take-sound id
   [M 0x18dda0:123-128]; #376 is the only 0x20 (a glass window), not a "sky-light"; hurt p5/p6 field
   map (decomp +0x15 one-shot = p6, +0x16 mask = p5 [M 0x17e870:25-58] — the port's hurt.rs swaps
   them; inert on 06, latent elsewhere); the radio is ambient-from-load, E-inert [M 0x1db9a8:81-86].
4. **Large gaps written up for tasking** (audit doc §"Large gaps", each with the [M] chain):
   0xD8 rigid bodies not simulated (well bucket #65 — RB_Create 0x1d6bf0/RB_Update 0x1d6e38);
   breakable panes not solid (#376 — drone.rs:16209-16224 withholds the capsule pass); the dressing
   tux skin swap (pre-existing); the 007-token gold-medal gate (D-004 vs [M 0x18dda0:~93]).
5. **Live checks listed** for the main agent (audit doc): SP #808's activation beat (gdbwatch its
   state across the ch-52 plates), the Destroy blast radius source (obj+0x8C vs the port's bounds
   stand-in), the well bucket's roll, and the SS trap sample 283 audible in a real boot.

## How it was verified

- **Evidence tags**: every game-side claim cites [M address] (the decompile read this session), [D]
  (the census probe dump re-run on the ISO), or a merged test; nothing rests on memory.
- **Tests run (all green, release, NF_ISO set)**: `census_07000006 --ignored` (the dump),
  `ss_operate_sound_06_live` (new), `simplescript_live` (2), `cutscene_06_live` (1),
  `alarm_panel_06_live`/`fusebox_06_live`/`interact_07000006_live` (9 across the three),
  `fix_06_driver_live` (2), `fix_06_alarm_live` (2), `fix_06_fire_live` (3),
  `searchlight_flip_07000006_live` (3), `searchlight_bullet_07000006_live` (2),
  `rook_sp_06_live` (1), `models_sweep_07000006_live` (2), `walk_07000006_live` (2). Total 30+ tests
  over 14 binaries, 0 failures.
- **Full suite**: `tools/test/suite.py` was started per the task, but its build phase
  (`cargo test --no-run`, feature-unified over all four crates) alone exceeded the session's
  remaining window on this 2-CPU sandbox (killed at 560 s inside the rebuild, no binary run) —
  **the total line was not obtained this session**. The four crates were fully built and every
  binary the task's changes touch ran green as listed above. The next session should run the
  fast runner on this tree first thing.

## What's left

- The full-suite total line on the final tree (see above).
- The four large gaps as their own tasks; the two open checks inside the scripts section
  (how ch 14/43's writers reach the barrel traps on the disc — no writer exists on the main map
  [D census]; the 0x06000071 stream-count unknowns for 0x0600004B/0x06000055/56/0x06000a28 are
  fillable with a stream dump probe).
- The hurt p5/p6 latent swap (no 06-visible effect; fix under the hurt-volumes spec).

## Usage

Context window: not visible to me (no token/cost readout is exposed in this environment's
interface). Input/output tokens, context used, cost: not visible to me.

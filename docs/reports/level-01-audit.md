# level-01-audit — the 07000001 behaviour audit

Branch `s/level-01-audit` off origin/master (aa94ce02), merged `origin/s/level-14-cutscene-door`
(5376c89b — the Script_CameraStart si+0x00 identity fix). Window 20:29–22:28 UTC. Single-agent (no
subagent failures this time — none were needed after the placement census went programmatic).

## What's done

1. **The per-placement table** — `docs/levels/07000001-audit.md`, 331 rows (every placement of main
   map 01000093 [D], re-parsed from the disc this session, ordinal numbering — the record's own u16
   field is the object index, not the placement number; the level doc's #N rows all verified,
   including the 0xD8 props #9-#12). Grouped by type; each type's header carries the game's creator
   in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, its update and use/damage/event
   handlers [M with addresses], the port's module (re-grepped this session), and the GAP verdict.
2. **The scripts** (§2 of the audit, full stream tables [D decode]):
   - The intro 0x06000087 (pack 07800001): camera=1, 15 streams, 2120 ticks; the camera's world-space
     keys (t0 hold (-6.8660, 6.7988, -9.1352) fov scale 0.305986/0.157729, dolly to (-10.2547,
     7.0901, -10.7304) at t2108); anims on skins 0x2b/0x17/0x4f/0x09; the world-space entity movers
     0x020006b1 (type 0x31) and 0x02000617 (type 0x19); the ch 0xB hand at ~t2119.
   - SP #276's scene 0x06000086 (level archive): 280 ticks, camera, guard anims + weapon events,
     the ch 0xE write.
   - SP #79: dead in the shipped game (scripts in no archive [D]) — no gap.
3. **The mission records** (§3): obj0's ch 0x47 has NO disc writer (re-verified); the fail chain
   ch 0x60 → switch_channels[0x63] → Mission_Update 0x182c60 [M] — both are the mission/dialogue
   layer (GAP-C).
4. **A gap fixed**: the cut-scene camera base (the task's "cut-scenes with broken cameras") — the
   `Script_CameraStart` 0x1d9140 si+0x00 identity fix merged from the level-14 branch, now pinned on
   THIS level by the new `cutscene_camera_07000001_live` (t0 hold + t2108 endpoint + the fov =
   key scale × 4/3 formula, all [D]/[M] cited). 1 passed.

## The gaps (§4 of the audit)

- **GAP-A (large, task seed)**: the rigid-body props (0xD8, placements #9-#12) are draw-only —
  `RB_Create` 0x1d6bf0 / `RB_Update` 0x1d6e38 (wind + Euler + bullet impulses [M]) have no port
  counterpart; nothing to pin without the [M] chain, so no test here.
- **GAP-B (render)**: the G-050 shoji (0xDB #14) drew black in a playtest — needs the main agent's
  render (live check 2).
- **GAP-C (large, known)**: the mission/dialogue layer owns obj0's ch 0x47 and the fail consumption.
- The driver's symptom list is mapped in the audit: walk-through-Bond and guest chatter are
  engine-level (drone machine / dialogue), the level's only E-console (the monitor #105) and its
  door locks are ported [M-cited rows].

## How it was verified

- The placement census cross-checked against the level doc's per-kind table (counts match exactly).
- `cutscene_camera_07000001_live` — 1 passed (it caught my own wrong t2108 fov expectation on the
  first run; fixed to key[1159]'s own scale — the test now pins the disc keys exactly).
- The touched-crate run: `cargo test --offline --release -p nf-view --test
  cutscene_camera_07000001_live -- --test-threads=1` (1 passed). The level-14 tests brought in by
  the merge were green in that task's runs; `cutscene_door_07000014_live` +
  `cutscene_camera_07000014_live` are on this branch and green there.
- The fast runner `tools/test/suite.py` from 20:31 UTC (803 binaries, build 1021 s): the failures
  seen so far are the disc-family ones — `disc_golden` ×3 + `identity_disc`, which are the TRIMMED
  onfoot ISO's environmental failures: the ISO measures 518,435 sectors [D] while
  `identity_disc` pins the full retail image's 1,191,072 (docs/build-identity.md) — pre-existing,
  not touched by this branch (nf-formats carries no delta from master). The final total line is in
  §7 below as the runner finishes.

## What's left

- GAP-A as its own task (the RB physics port), GAP-B as a render check, GAP-C as the mission-layer
  task.
- The suite's final total line (the runner completes ~22:05; §7 records it).

## Live checks for the main agent

1. The intro's render: `--bond -2.78,6.84,-12.40,-0.089,0` at load+30 — the hall from the t0 key eye
   (-6.87, 6.80, -9.14), the contact and the two movers visible; hand-back at +2119.
2. The G-050 shoji #14: `--bond 7.9,5.3,9.0,<yaw toward (7.916,5.335,11.139)>` — texture, not a
   black quad.
3. The second scene after the meet plate #160 — the 0x06000086 camera keys and the guard anims.

## Usage

not visible to me (no token/credit/context-window/cost figures are shown anywhere in this sandbox
environment; the agent transcript does not display usage meters).

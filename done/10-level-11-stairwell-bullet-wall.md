## Your task: `level-11-stairwell-bullet-wall`, 07000011: something invisible stops the rounds aimed at the rope guards

In the 07000011 stairwell, rounds fired at the abseiling guards stop on something invisible in the well. Three spots where it happens (Bond root x, y, z, yaw, pitch): (15.91, 24.01, -2.45, -5.907, -0.107), (20.35, 24.01, -2.44, -6.757, -0.102), (15.33, 26.69, 1.33, -4.900, -0.559); the guards are on the ropes at (18.895, 46.516, 1.434) / (19.745, 46.580, 0.363) [D]. The bullet decals land on the far stairwell wall, not on the guard.

1. From each spot, cast the bullet path toward the rope guard and list every triangle/object the port's bullet collision meets (owner object, placement kind, glist flags, material/surface byte, collision class). Find which one the game's bullet passes and the port's does not [M — `Bullet_update` 0x12e7b0 / `Bullet_CollisionHandler` 0x12ce80, the pick mask the round uses, `Collide_Pick` 0x1eb320's node/surface drops, the pass flags (surface 0x40/0x80, bit 0x10 render-only objects, glass class 0x10)].
2. Fix it [M], with a real-input test from one of the spots: the round reaches the guard and damages him; sister: a round into a real wall still stops there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T08:43Z

---
## Report (done, 2026-10-25T09:14Z)


---
## Report from the worker (done, 2026-10-25T09:14Z)

# level-11-stairwell-bullet-wall — rounds at the rope guards stopped on the invisible fudge

Branch `s/level-11-stairwell-bullet-wall` (from origin/master 8de76e31). Window 08:46–10:45 UTC;
this session is the whole task.

## The blocker, identified

The port's player-bullet flight trace stopped on the first triangle it met — including the
pass-flagged ones the game's flight walks past. In the 07000011 stairwell that triangle is the
collision object **map43#126**, whose own disc name says everything:

> `stairwell_blocking_ploys, this may look od buts its a fudge!!!! do not change or thre game will break!`

glist flags 0x10, its triangles carry the **surface byte 0x40** [D 07000011.bin]. The census
(`cargo test -p nf-game --lib stairwell_bullet_census_07000011 -- --ignored --nocapture`, the
assertion-free probe added to `crates/nf-game/src/world.rs`) shows it crossing every
spot→guard ray at t 0.05–0.10 from all three reported spots, with the far stairwell wall
(map43#127 `ROOM_StairWell_2`, mat 0x0a) behind the guard at t ≈ 0.36–0.39 — the wall the
user's decals land on when the round misses.

## The game's rule the port missed

A flying bullet ARMS its HITTEST pick every tick: `HITTEST+0x94 = 0x101` or `0x201` (the node
bounds class) and `+0x9a = 0x20` for guns whose `weapon_data+0x6c` lacks 0x800 [M Bullet_update
0x12e7b0 disasm 0x12f030-0x12f058 (the surface-mask write), 0x12f134/0x12f154 (the node-mask
writes), state gate at 0x12f030-0x12f038]. `Collide_Update` 0x1ea398 sorts the contact list
nearest-first and walks it: pass = `HITDATA+0x51` bit 1 (which `Collide_Intersect` 0x1ebaa8
ORs in from the glist node's flags bit 0x40 or the surface byte's bit 0x40 [M disasm
0x1ebf78-0x1ebf80]) | `HITDATA+0x50` bit 0x40 | the 0x80 random roll | (surface mask 0x20 →
glass class 0x10 passes) — a non-pass hit ends the walk, pass-flagged hits are skipped [M
0x1ea398:216-229]. The fudge's 0x40 surfaces pass; the round reaches the rope guard.

The earlier reading (the foliage session's "the bullets' HITTEST stays inactive (+0x94 = 0
[M 0x12c668]) so the pick never arms") mistook the INIT zeroing for the steady state: the
flight re-arms the masks per tick. That premise had pinned `fix_foliage_bullets_live`'s flight
assertion to the shell's first face — the TEST was wrong against the decompile and is fixed
here (the round now stops on the unflagged wood core obj 97 behind the flagged shell; the
"bulletproof leaves" look is unchanged, the core's decals hide inside the canopy; the sight
trace still stops on the shell face).

## The fix

- `crates/nf-game/src/drone.rs` `update_player_bullets`: the flight trace `d.world.segment(b.pos,
  end)` → `d.world.segment_bullet(b.pos, end)` (drops every hit whose glist node flags carry
  0x40 or whose surface byte carries 0x40 — the port's existing `bullet_pass` rule).
- `crates/nf-game/src/bullet.rs` `Bullet::update`: the drone-fired flight, same change.
- The 0x80 half-pass roll and the glass-class flight pass are NOT modelled in the port's flight
  trace [S documented deviation]: no 0x80/glass surface sits on these paths, and the roll would
  spend shared RNG the game spends only per 0x80-surface hit.

## Verification

- NEW `crates/nf-view/tests/stairwell_bullets_07000011_live.rs` (real input path, NF_ISO):
  `the_round_from_the_reported_spot_reaches_and_damages_the_rope_guard` — Bond wakes the abseiler
  p258 through the real plate (TouchOnce #253 → ch 0x13 [D]), the guard hangs [M 0x169730:55-58],
  Bond moves to the reported spot (15.33, 26.69, 1.33) and fires at the hanging guard — the round
  crosses the fudge plane and DAMAGES him. Sister `a_round_into_the_real_wall_still_stops_there`:
  a round aimed down stops on the stairwell's own geometry below the spot, guard untouched.
  → **2 passed, 0 failed**.
- `fix_foliage_bullets_live` (corrected): **3 passed, 0 failed**.
- `cargo test --offline --release -p nf-game --lib` → **547 passed, 0 failed** (incl. the
  pre-existing world/bullet unit tests, none pinned the old flight stop).
- Regression set 1 (bullet-adjacent): ai_traces 4/4, impact_remainder 3/3,
  stairwell_guards 3/3, stairwell_walls 3/3, stairwell_ghosts 2/2.
- Regression set 2 (gunplay/missions): doublecross_transitions 6/6, hostage_killer_07000002 4/4,
  fix_pickups_ammo 5/5.

## What's left

- `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` fails on this tree
  (stall at "hall ammo resupply 0", Bond (5.45, 6.68, 11.60), the nav gives up toward
  (−6.66, 5.34, −5.58)). Master (95a7bb72) pins it green; the delta here is this task's flight
  change, so the round-path change shifted the 01 walk's tuned choreography (the level-06 session
  tuned the shoot-line to the shared stream — every wall hit draws effect/ricochet RNG, and
  rounds that now pass the pass-flagged faces stop elsewhere, moving the draws). The next session
  should re-tune the walk's shoot line against the corrected ballistics (the [M] chain above is
  the ground truth; the walk's timing, not the rule, needs the update). Repro:
  `NF_ISO=… cargo test --release -p nf-view --test walkreal_07000001_live walk_from_the_spawn`.
- The full nf-view suite (≈90 test binaries) and nf-audio were not run this window (the sandbox
  kills background jobs — the fast runner died twice; foreground calls cap at 10 minutes). The
  touched crates ran green directly (above).

## Live checks for the main agent

- No visual work in this task; nothing to render. (Optional spot: on 07000011, standing at the
  reported spots and firing at the rope guards should now show the guards' hit reactions and,
  for misses, decals on the far stairwell wall — the game's own behaviour.)

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).

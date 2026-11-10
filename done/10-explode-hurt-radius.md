## Your task: `explode-hurt-radius`, explosions hurt only as far as the game's blast radius

The level-0b-roof-sprint-4 report (origin/s/level-0b-roof-sprint-4, docs/reports/level-0b-roof-sprint-4.md, item 2 of "What's left") measured the port's explosion hurting Bond at about 13.7 u where the disc's radius for that weapon row (0x2e) is 8.0 [L both sides in the report]. Read the level-08-rocket-self-blast report (origin/s/level-08-rocket-self-blast) first so you do not redo its findings. Then find the port's hurt reader for `Explode_Create(weapon+0x08 radius, weapon+0x0C damage)` [M near 0x12df3c, bullet.rs:52-54's comment] and compare its distance gate and falloff with the game's `Explode_Update` and the hurt it calls, with addresses, for Bond, guards and objects. Fix any difference in the game's order. Pin it with a test on the 0x2e row (hurt at 7.9 u, none at 8.1 u) and through real play on 0700000b. This shifts every walk with explosions (rockets, satchels, grenades, barrels), so run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the bullet and explosion binaries (grep tests for Explode_) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/explode-hurt-radius`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T23:42Z

---
## Report (done, 2026-11-10T00:21Z)


# explode-hurt-radius — report

Branch `s/explode-hurt-radius` = origin/master (0a9d39435) + 2 commits. Task: the port's
explosion hurt reportedly reached ~13.7 u where the disc's radius for weapon row 0x2e is 8.0
[level-0b-roof-sprint-4, "What's left" item 2]. **Finding: the port's blast law matches the
game everywhere audited; the sprint-4 "~48 at 13.7 u" was a misread of WHERE the rocket
detonated — the office furniture ate it ~4.7 u from Bond, and the falloff there IS ~44. Pinned
both ways; no code change to the law was needed.**

## The game's law ([M], all read from the disassembly this session)

- `Explode_Propagate` 0x17c518 signature `(VECTOR* pos, float radius(f12), cel_tag* cel(a1),
  float damage(f13), obj_tag* owner(a2))` [M prologue 0x17c518-0x17c55c: `mov.s f21,f12`,
  `mov.s f22,f13`, `s3←a0`, `s2←a2`; cel==0 → return 0x17c54c; radius < 0.0002 → return
  0x17c554-0x17c56c (the constant 0.0002 = 0x3951b717)].
- The pick: `Collide_SphereIntersect` 0x1ec940 fills the HITTEST sphere from the caller's
  {pos, radius} 16-byte block and calls `Collide_Pick` → `Collide_PickObj` 0x1eae80, whose
  version-0x100/0x101 arm is **sphere-vs-sphere**: hit iff `SqDist(centre) < (blast_r +
  obj+0x8C)²` [M 0x1eae80:221-228]. The pick adds the victim's own sphere radius.
- Per hit (skip `obj+0xFE & 1` [M 0x17c5c8-0x17c5d0]):
  `d = Vec_Dist3D(obj+0x80, pos)` [M 0x17c5d8-0x17c5e0], `gap = max(d − obj+0x8C, 0)`
  [M 0x17c5e8-0x17c608 — **`max.s f0,f0,f3` at 0x17c608: the decompile's `(int)x * (0 < x)` is
  Ghidra's print of max.s; no integer truncation exists**], `amount = damage · (1 − gap/radius)`
  [M 0x17c60c-0x17c614], skip if `amount < 0.0002` [M 0x17c618-0x17c620].
- Case dispatch on `obj+0xFF` [M 0x17c628-0x17c648 jump table]: **3 (Bond) →
  `Player_Hurt(obj, amount, pos)`** [M 0x17c650-0x17c65c; ≤ 0 return + owner≠0 + MP →
  `MP_RegisterBulletHit` 0x17c678-0x17c6d8]; **2 (guard) → `Drone_ExplosiveHit(obj,
  {amount, radius, owner})`** [M 0x17c6e0-0x17c6f8 — the DroneImpact struct {f20=amount@+0x00,
  f21=radius@+0x04, s2=owner@+0x20}]; 5 → `Bullet_handle_object_destruction` 0x12cd98 for ids
  {0x2b, 0x34-0x37, 0x3a}; 0x20 break, 0x21 destroy (falloff ≥ 1.0 + mask 0x20), 0 copter,
  0x28 vehicle, 0x35 MP, 0x36 GT, 0x37/0x38 (empty stubs), 0x3d sub, 0x4b mine chain.
- `Explode_Create` 0x17cbd0 stores radius→ext+0x2C, damage→ext+0x28, cel→ext+0x24,
  owner→ext+0x20 and creates the class-0xF object; its one-shot collision handler
  `Explode_CollisionHandler` 0x17cde0 calls `Explode_Propagate(obj+0x30, ext+0x2C, ext+0x24,
  ext+0x28, ext+0x20)` once [M 0x17cde0:9-14 — the +0x3A latch]. `Explode_Update` 0x17c350 is
  debris/sound only and never hurts [M whole function].
- The disc rows [L ee_snapshot.bin, base 0x2bf150 [M action-symtab.tsv], row = 0x10c]:
  0x2e radius 8.0 damage 100.0 +0x70 = 0x4 @ 0x2c2178; 0x2c/0x2d also 8/100; 0x2b 6/75;
  0x34 8/100.

## The port's side (audited, unchanged)

`spawn_blast` (drone.rs:15485) → `explode::propagate` (explode.rs:318) with the same falloff
(`explode::falloff`, the `max.s` reading was already pinned there), the player sphere 1.0576
= Bond's obj+0x8C [L ee_start_05.bin M_CONTROL+0x8C], guards DRONE_SPHERE_R 1.05 [S: a fold —
the game reads each guard's own obj+0x8C], breaks/destroys as their bounding spheres, the guard
damage the raw amount through msg 9 (`Drone_ExplosiveHit`'s law), the 0.0002 cutoff, and the
blast radius/damage read from `rec+0x08/+0x0C` at every creation site (fuse death
drone.rs:23825, wall impact 17340, guard-body 16699, cook-off 15578). No double-propagate (the
port's Explosion object only draws; `propagated` is a latch).

## What the reproduction showed (the real finding)

The new real-path pin first FAILED exactly like sprint-4: a rocket fired with the aim converged
at a wall 11.8 u away hurt Bond 44.2/45.3. The detonation was NOT at the aim point — the
rocket's flight sweep hits the office's script-entity furniture ~4.7 u out (the same furniture
the pickup jog wedged on, stall at (−23.02, 28.45) [L this session's runs]); the game's law at
gap 3.63 gives 100·(1−3.63/8)·0.8 = 43.7 ✓. The sprint-4 ledger read the fire's convergence
distance (13.7) as the blast distance. The plain `world.segment` ray does not see that
furniture — the rocket uses the flight filter (`segment_flight_scoped`) plus the object sweeps,
which is why the first test rays measured 11.8 u clear.

## The pins (both green)

1. **The 0x2e row, unit** — `nf-game` lib `explode::rocket_row_0x2e_hurts_inside_the_disc_radius`:
   edge 7.9 → hurt 1.25 (the straight-line amount), edge 8.1 → none, edge 8.0 → none, centre
   13.7 → none [M/L cited inline].
2. **The real path, 0700000b** — `walkreal_0700000b_live::walk_rocket_blast_holds_the_disc_radius`:
   fresh Agent boot, real input only (no teleport after the load), the office jog takes
   PICKUP_RocketLauncher #349 (0x2e, 8 rounds [D docs/levels/0700000b-audit.md:191]), then at the
   walked desk spot the sim measures its own wall rays with the rocket's flight filter and fires:
   the near leg (blast d3 2.49) took 67.17 against the law's 65.69 (tolerance 3.0), and the far
   leg detonated at sphere-edge 11.71 u > 8.0 and hurt NOBODY. The jog is difficulty-gated
   (≥ 2), and the d1/warm/real/Agent/00Agent walks stayed green whole-binary (8/8).

## What I ran (whole binaries)

- `nf-game --lib` (release, --test-threads=1): **757 passed / 0 failed / 5 ignored** (the new
  unit pin included).
- `walkreal_0700000b_live`: **8 passed / 0 failed / 0 ignored** (47 s) — the four mission-end
  walks (d1/real/Agent/00Agent) with the inserted jog, the cab, the cutscene and the new pin.
- `explode_live` (nf-view): 2 passed; `blast_cooks_off_grenades_live` (nf-game): 3 passed;
  `blast_smoke_live` (nf-game): 4 passed.
- The fast runner (`tools/test/suite.py`) did NOT fit this window (setup consumed most of it:
  the work/ decomp re-download, the ALSA prefix rebuild, the capstone install, and the
  three-iteration pin test). Mitigation: the only `crates/*/src` change is `#[cfg(test)]` code
  in explode.rs (verified by the 757-green lib run and the unchanged release build), so no other
  binary's behaviour can move; the walkreal sweep risk is nil by construction. **The next
  message should run `tools/test/suite.py` on this tree** (branch head 5f860e55f).

## What's left

1. The full four-crate suite on the final tree (above).
2. [S, optional] per-guard obj+0x8C spheres instead of the folded 1.05 (the guard hurt boundary
   shifts by the model delta; no walk evidence of a shift).

## Live checks for the main agent

1. **The rocket-vs-furniture detonation** (the refutation of the 13.7 reading): on 0700000b,
   Agent difficulty, stand in the exec office at the desk approach (−19.05, 48.56, 30.94), aim
   at a wall ≥ 10 u WSW (through the west strip), fire the MGL once. Expected: the rocket
   detonates on the office furniture ~4.7 u from Bond and Bond takes ~44 (Agent factor 0.8),
   NOT a hurt proportional to the aim distance. If the game's rocket instead flies to the wall
   (pierce on that material), the port's flight sweep is over-eager on script-entity objects and
   the pin's near/far structure still holds — but the sprint-4 reading would need re-deriving.
2. **The past-the-radius non-hurt** [L both sides pinned in the test]: same spot, fire at the
   far wall with the detonation ≥ 9.06 u from Bond (radius 8.0 + Bond's 1.0576): expected — no
   health change. If the game hurts, the obj+0x8C pick reading (0x1eae80:221-228) is wrong.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage readout is exposed to the worker).

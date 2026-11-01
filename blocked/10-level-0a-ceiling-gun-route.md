## Your task: `level-0a-ceiling-gun-route`, 0a: getting past the west-hall ceiling gun on real health

On 0700000a laser #607 in the west hall feeds OR #667 -> ch 0x0F, ceiling gun #640's enable (at -22.1, 2.3, 17.2); since the GT_Update case-3 fix [M 0x180498] the gun re-arms and two bursts (27 + 15 rounds x 2.4) outlast Bond's health; only explosions damage a turret (GT_ApplyDamage 0x1816d8 <- Explode_Propagate 0x17c518) and 0a's kit has none. `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` is ignored for this. Find how a player gets through: avoiding #607 (crouch under / step over — the laser row's height vs Player_Collision 0x194df0's 1.86/1.3 body), a route around it, the gun's cone/LOS, and whether its rounds spread (Bullet_init 0x12c5a0's spread term for owner -1 rounds — the port fires them dead-on). Fix any port gap with the address, make the real-damage walk take the player's route and un-ignore it. Run every *0700000a* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-ceiling-gun-route`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T10:10Z

---
## Report (blocked, 2026-11-01T11:45Z)


---
## Report from the worker (blocked, 2026-11-01T11:45Z)

# level-0a-ceiling-gun-route — report

Task: on 0700000a, laser #607 (out ch 0x1F, mask 8) feeds OR #667 → ch 0x0F, ceiling gun #640's
enable (param 4, at (−22.1, 2.3, 17.2) [D 0700000a-audit]); since the GT_Update case-3 fix
[M 0x180498] the gun re-arms after every crossing and two bursts outlast Bond's health; only
explosions damage a turret (GT_ApplyDamage 0x1816d8 ← Explode_Propagate 0x17c518 [M]) and 0a's
kit has none. Find how a player gets through, fix the port gaps with addresses, make
`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` take the player's
route and un-ignore it, run every *0700000a* binary and nf-game.

## What the game does (evidence)

- **Turret rounds spread.** `Bullet_init` 0x12c5a0 computes
  `spread = (weapon+0x24 [+ skill bloom]) × scale`, scale = 0 only for a player-owned (obj+0xff 2)
  round under flag 0x400000; every other owner keeps 1.0 [M 0x12c5a0:134-184; disasm 0x12c710-0x12c7f4
  — the suspicious `BL+0xC0` load is a delay-slot load, the decompile is not mangled]. The turret
  fires `Bullet_init(−1, owner, barrel, 0x2c55d0, muzzle, dir, 0xffff)` [M GT_Update 0x180498:347-348
  case 4, :519-521 case 7]; weapon record 96 = (0x2c55d0 − 0x2bf150)/0x10c, +0x24 = 4.0, +0x68 = 0
  (no 0x400000) [L ee_start_05.bin @0x2c55d0]. Then the cone draws: FRand_MVar2(2s)·0.0014,
  FRand(2π), FRand(π), the spherical offset added to the direction [M 0x12c5a0:166-184, disasm
  0x12c7f4-0x12c858] — 3 shared-stream draws per shot — and GT_Update draws one more Rand_FRand(2π)
  into the direction vector's w pad [M 0x180498:349/:522] — 4 draws per shot.
- **The crouch is a different pick arm.** `Player_Collision` 0x194df0's substate switch case 3
  (= substate 4) builds A = pos + 0.1 − (cc − 0.55), B = pos + 0.45 − cc, r 0.55 [M :228-248; the
  port's own mirror is player.rs `pick_arm`]; with the crouch rest cc 0.55 [M player.rs
  CROUCH_ROOT_HEIGHT] and the crouch park pos−floor 0.607 [L level_reload_live.rs:215] the crouched
  body tops ≈ feet+1.26, under beam #607's band y 1.49-1.76 (x ≈ −8.45, z 15-20.25 [D triggers dump,
  interactables_0700000a_live]) — the standing body 1.86 [M 0x194df0 default arm] crosses it.
- **The low beams cannot be crouched — they are dodged at their ends.** The laser web's other
  mask-8 rows on/near the route are LOW (ankle) lines [probe of the parsed trigger tris, this
  branch's `probe_laser_web_positions`]: #614 (out 0x22 → OR #667 → ch 0x0F → gun #640) z 13.03,
  x −24.7..−19.45, y 0.26-0.52; #621 (out 0x24 → OR #668 → ch 0x10 → gun #639) diagonal
  (−14.93,−32.34)→(−9.90,−33.87) y 0.26; the 0x25 fence #652-654 diagonal (22.93,−13.89)→
  (19.20,−10.76) 4 stacked rows; #625 (out 0x26) x 18.88, z 2.38-7.64. The capsule bottom reaches
  feet−0.04 [M 0x194df0 case 3], so no stance clears a low line; the pass is around the line's end
  (the game's own passage evidence: northward rays from z −34.4 clear at x −9 [D probe rays]).
- **GT burst/ammo**: GT_Create sets ext+0xF0 = 200 (facing 0) or 0xffff (wall guns) [M 0x17ffa0:83-96];
  the fire gate is 0.1 s/round [M 0x180498:317-323]. The "27 + 15 rounds" of the old ignore note is an
  observed artifact (re-arm cycles × LOS), not a spawn-site value.

## Port gaps fixed (both with addresses)

1. **Turret rounds flew dead-on with zero RNG draws** — `crates/nf-game/src/drone.rs`, both
   `GTEvent::Fire` consumers now run the round through `crate::bullet::spread(dir, weapon+0x24,
   flags68, rng)` (the existing bit-exact cone [M 0x12c5a0:166-184]) and consume the post-shot
   `frand(2π)` pad draw [M 0x180498:349 case 4 / :522 case 7]. Record 96: spread 4.0, flags 0 → the
   cone always applies [L ee_start_05.bin @0x2c55d0].
2. **The trigger toucher ignored the crouch** — `drone.rs`'s player toucher used the standing box
   (feet+0.55..feet+1.86, the measured reach stand-in) for every stance, so a crouched Bond still
   fired beam #607. Now the crouched stance uses the game's case-3 arm via the already case-correct
   `drone::Player::body` (built from bond.substate/crouched/cc at the scene site [M 0x194df0:228-248]);
   the standing stance keeps the calibrated box (the truck-chain/park-plate reach measurements
   documented at the site). New `drone::Player::crouched` field (own line, per merge lesson #4),
   fed from scene.rs.

## The route (the player's answer)

The real-damage walk now crouches every #607 crossing (the outbound hall leg, the walk-back's
crossing legs via `near_hall_beam`, the ring's west leg) and dodges the low beams at their ends
(#621 through the game's x −9 passage and the band dip; #614 east of its east end; the 0x25 fence
north of its north post; #625 south of its south end). A final pin asserts no mask-8 laser out
(0x1F/0x20/0x21/0x22/0x24/0x25/0x26) and no alert-OR out (0x0F/0x10/0x11/0x12/0x81) fired — gun
#640 never armed [D the OR wiring, M Trigger_MultiplexOrIn 0x1b2d38]. `#[ignore]` removed.

## Verification status (honest)

- **nf-game: 635 passed / 0 failed / 5 ignored** (both fixes in; run this session,
  `nf_game --test-threads=1`).
- `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage` (un-ignored): with
  the #607 crouch alone, Bond no longer dies under gun #640 — the death moved to gun #639 (42 × 2.4
  at (−20.96, 1.10, −9.62), armed by beam #621 through OR #668), which identified the low-beam
  crossings. With the full dodge set the walk gets to the Q-Worm install but the **test still
  fails**: the outbound `follow_nav_route` A* crosses beam #614 (z 13.03 at x ≈ −22.1 [walk log:
  node 0 (−21.10,17.15) → node 1 (−22.12,13.38) → node 2 (−22.19,8.62)]) which arms gun #640
  (0x22 → OR #667 → 0x0F), and the band approach legs need via-pinoning (the try_walk_to legs can
  silently strand Bond short of the stand, so the install shot never lands). The remaining work is
  mechanical: give the outbound nav a via east of #614's east end (as the walk-back already does
  for #621), and make the band legs fatal (`walk_to`) once the passage is proven. The diagnostic
  prints to finish this are on the branch (the `[ring]` per-leg channel/turret dump,
  `probe_laser_web_positions`).
- The other *0700000a* binaries (interactables, keyboard, walk, fix_0a_rain_indoors) built but
  **did not run** — the sandbox's disk filled twice (cleaned twice: the stale test binaries +
  4.4 GB), and the remaining window went to the route iteration above. The fast-runner suite
  (`tools/test/suite.py`) was started three times; the sandbox kills true background jobs between
  tool calls (the level-16 WORKLOG records the same), so no suite total exists this session.

## Live checks for the main agent

1. Fresh-boot 0700000a, walk Bond crouched through beam #607's plane (x −8.45, z 15-20.25):
   expect switch_channels[0x1F] stays 0 [M 0x194df0 case 3 vs the beam band 1.49-1.76 [D]].
2. Same boot, walk standing through #621's line ((−14.93,−32.34)→(−9.90,−33.87) at y 0.26 [D tris]):
   expect ch 0x24 fires; then crouched the same line: expect it still fires (the capsule bottom
   feet−0.04 [M]) — confirming the low beams are dodge-only.
3. Wall gun #640 firing: trace the 4 shared-stream draws per shot (3 in Bullet_init 0x12c5a0, 1 in
   GT_Update 0x180498:349) to pin the port's stream position.

## What's left

- The outbound nav via + the fatal band legs; then the real-damage walk should complete and the
  laser pin should hold.
- Run the remaining *0700000a* binaries and the fast-runner suite on the final tree; quote totals.
- The `probe_laser_web_positions` probe test can be trimmed or kept as the web census.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment).

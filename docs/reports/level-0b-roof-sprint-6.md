# level-0b-roof-sprint-6 — report

Branch `s/level-0b-roof-sprint-6` = `origin/s/level-0b-roof-sprint-5` (1d5bb40de) merged with
current `origin/master` (c7d5b9dfe; clean — master's effect-light pick + guard body sizes +
0d ladder leg). **Status: the 00 Agent walk is still red (the same tick-16517 death), but its
ledger now carries sprint-6's measurements: the sprint-5 vel-0 moving-accuracy divergence is
DISPROVEN with run proof, the port's blast radius is proven to obey the disc law exactly, and
the rocket-into-the-band lever is measured dead until the 0x2e flight law is probed.** The
whole `walkreal_0700000b_live` binary is 6 passed / 0 failed / 1 ignored on the final tree;
`interactables_0700000b_live` 14 passed / 0 failed. No `crates/*/src/` changes, no damage or
health lowered, no pins loosened; everything new is in the 0b test file, gated to difficulty >= 3
(the d1/d2 walks' streams byte-identical — the whole binary passes them unchanged).

## What's done

1. **The first lever (the moving-target accuracy) is resolved: it already covers this walk.**
   `origin/s/onfoot-moving-accuracy` is gone from origin, but its content IS on master
   (933065298, then 566f1d298 "Guards' moving term reads the player's move quad") and is an
   ancestor of sprint-5's base — it is in this tree. The code: the walk stores the lanes every
   tick (`Player_Move` 0x191dd0:166-170 → `Player::move_local`, player.rs:2000), the guards'
   targetting reads that quad per drone (`DroneWeap_DoOpponentTargetting` 0x179a48:34-41 —
   drone.rs:20595), latches it per guard (`observe_target_motion`, drone.rs:11831), and
   `DroneFiring_TargetMoving_Accuracy` (0x3f400000 = 0.75) applies on the fired shot
   (drone.rs:12869, [M 0x179740:71-73]). Run proof: the `[0b-stall]` trace now prints Bond's
   move quad AND each guard's own latch — every g4/g6/g7/g8/g9 line reads `moving true` on
   every 30-tick sample while Bond runs [L a00_run1 t15947-16517]. **The sprint-5 "vel (0,0,0)
   while running" was a mis-read: that print showed BL+0x50 (the collision velocity, zeroed
   while grounded), not the accuracy's move quad.** No fix was needed; none was made.
2. **The reproduction on the merged tree** (NF_DMG_TRACE, a00_run1): the same death as the
   sprint-5 ledger — tick 16517 at (7.13, 58.07, 41.22), 22 hits for 100.0 — with the full
   per-hit decomposition: #4 28.0 in 4 (14.0 at d 20-25 during P1, 14.0 at the close), #9 14.0,
   #6 14.0, #7 10.5, #5 10.5, #8 7.0, **the world 16.0 = FOUR 4.0 live-wire hurts** in the ring
   corridor (frames 13380-13423 at (−1.4, 47.8, z 2.9-6.2)). That world toll is the fusebox law
   — a group-1 wire death hurts while the group's +0x56 word is zero, unconditional on
   distance [M FuseBox_Update 0x1adfe8:99-101; fusebox.rs Cut::LiveWireHurt] — the wire deaths
   ride guard rounds/script during the exec-floor legs, and the walkthrough cannot dodge an
   unconditional hurt. Ledgered as the fusebox task's item.
3. **The P1/P2 long-range toll's intended answer — one carried rocket into the band — was
   built, run three times, measured, and NOT shipped.** The walk carries 0x2e clip 4 at control
   return [L a00_run1]; the disc placements hold #6/#7/#8/#9 within 6.1 u of #6 [D
   drone-props; L the stalls], so a hit on #6 kills all four on the disc falloff
   100·(1−gap/8) [L ee_snapshot weapon_data+0x2e·0x10c+0x08/0x0C; M Explode_Propagate
   0x17c518]. The attempts (all real input, all gated difficulty >= 3, each reverted):
   - a00_run3: a flat converged shot from control return (d 35.4) struck a SW deck structure at
     (−6.5, 58.6, 7.9) — 5.2 u from Bond, −47.37. The same run PROVED the port's blast law:
     the trace's explosion record reads radius 8.0, damage 100.0, and 100·(1−4.21/8) = 47.37 —
     **the port's blast radius obeys the disc exactly; the sprint-4 "hurt at 13.7 u" was a
     mis-attributed detonation distance, not a radius divergence** (the explode-hurt-radius
     task should re-measure the blast POSITION, not the radius).
   - a00_run4: from the P1 lane point (the ray east to #6 clears the structure by ~10 u) the
     shot hit the lower floor band at d ~14 — **the round ARCS**: impact y 57.6 vs the launch
     line's 58.93, 1.33 u of extra drop at 14 u [L]. The −35.01 self-hurt killed the run.
   - a00_run5/6: drop-compensated shots (quadratic then linear fits of that one point) — the
     quadratic flew ~8 u past the cluster (only #8's −4.2), the linear missed everything at d
     35. run5 also exposed a second trap: without a switch-back request the gun stays 0x2e
     ~100 ticks after the shot (w0x2e t16157 → t16277 [L]) and the sprint-5 close runs COLD —
     Bond died at d 1.0 of #4 under ×2 TooClose rounds [M 0x139fac-0x139fc8].
   The three attempt blocks are replaced in the test by a ledger comment; the ignore message
   carries the same ledger.
4. **The shipped changes** (all in `crates/nf-view/tests/walkreal_0700000b_live.rs`): the
   `[0b-stall]` diagnostics (the move quad per sample; guards #6/#9 added; the per-guard
   `moving` latch), the sprint-6 ledger comment where the window block was, the refreshed doc
   comment, and the sprint-6 ignore message (the resolved levers, the dead lever, the next
   levers). The d1/d2/warm walks are untouched (the whole binary passes them unchanged).

## How it was verified

- `walkreal_0700000b_live` (the whole binary, final tree): **6 passed; 0 failed; 1 ignored**
  (`--test-threads=1`, 37 s) — the same green set as sprint-5.
- `interactables_0700000b_live`: **14 passed; 0 failed** (25 s).
- `nf-formats` `effect_mesh_flags_census`: **1 passed; 0 failed** (0.7 s; the merge's new test).
- `nf-game` lib tests: **760 passed; 0 failed; 4 ignored** (1.1 s) — the merge's drone.rs /
  effect.rs changes exercised.
- The full `tools/test/suite.py` run did not fit this window: the background process is killed
  between tool calls in this sandbox (setsid + nohup included — it died twice, once on a lost
  PATH, once reaped), and the foreground full build + all binaries exceed the remaining time.
  The four runs above cover every binary the branch touches. The next window should start the
  suite early in FOREGROUND chunks (`--only bin1,bin2`) and read the total line.

## What's left

1. **The 00 Agent walk** (the ignore message's ledger): (a) probe the 0x2e flight law
   (Bullet_init 0x12c5a0's speed/fuse fields for the record — the drop measured 1.33 u at 14 u
   [L a00_run4] but its shape is unpinned), then re-arm the band shot at control return WITH
   the fire-tick 0x43 switch-back; (b) a bounded close+dart on #5 at the gap mouth (10.5 +
   run6's point-blank 21.0 at d 2.4 say the P2→P3 pass needs #4's treatment); (c) the
   live-wire 16.0 belongs to the fusebox task.
2. The explode-hurt-radius task: the radius is NOT divergent [L a00_run3] — re-point it at the
   blast position law (where the sprint-4 rocket actually detonated).

## Live checks for the main agent

1. **The rocket's arc.** In PCSX2 on 0700000b: fire one rocket (0x2e) level at a wall ~20 u
   away and one at ~35 u; expected — the round's impact sits measurably BELOW the muzzle line
   at the longer range (the port's 1.33 u @ 14 u drop [L a00_run4] should match the game's; if
   the game's round flies FLAT, the port's Bullet_init gravity bit for 0x2e is the divergence).
2. **The live-wire hurts.** In PCSX2 on 0700000b at the ring corridor (−1.4, 47.8, z 3-6):
   expected — the same 4.0-per-death Player_Hurt while the fusebox group is live [M
   0x1adfe8:99-101], and the walkthrough's 4 deaths (frames 13380-13423) should have disc
   counterparts (guard rounds or a script killing the wires).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage readout is exposed to the worker).

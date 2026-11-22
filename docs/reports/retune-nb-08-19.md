# retune-nb-08-19 — the 08 whole-mission 00 Agent real-damage walk, round 19

## What's done

- Branch `s/retune-nb-08-19` from `origin/master` (b5278f0d1) with `s/retune-nb-08-18`
  merged first (08-18 was not on master; master had not moved, so the merge fast-forwarded).
  Setup from scratch (the sandbox reset): clone, work-slim + the verified 369823203-byte
  ISO archive, rustc 1.100 (rustup), the extracted libasound2-dev AND runtime
  libasound2t64 debs into a user prefix (no root; the linker pointed at it via
  `~/.cargo/config.toml`, a sandbox file, never committed), `cargo fetch`, the nf-queue
  worktree.
- **THE DIVE-CADENCE READ (the round-18 ledger's item (3)) DONE AS [M] CODE READS — the
  round's key finding, no run needed:**
  - [M Copter_Move 0x17d628:63-113] at each route-path END the machine advances to the
    chain path whose +0x16 index equals `uVar4 = (uint)(route_path_count * damage/(0.9*hp)
    + 0.5)` while the damage fraction is below 0.5 — a RANDOM pick at or above it — and
    LOOPS the current path when no index matches (t resets to path+0x14). Kind-2 paths are
    skipped at switch time; 08 has none [D the 07000008-audit route rows — #45's flight
    script 0x06000050 carries ONE spline entity].
  - [M Copter_Update 0x17dbb8] case 1 opens the attack state every (FRAME_RATE*4)/DiffMod
    ticks (120 at d3) on the 1000-u + view-cone + sight check; case 2 fires until
    FRAME_RATE*5*DiffMod (600 at d3), the sight drops, or the machine's taken damage
    passes 100 (rec+0x6c, reset at every attack-state entry).
  - **Consequence [M]: with ZERO machine damage the machine can never advance** — the
    round-18 ride-free shape (zero launches, Hind damage 0.0/598) was unwinnable BY LAW.
    The arrival-dive launch itself is the lever: one landed rocket = fraction
    101/(0.9·598) = 0.188, which drives uVar4 off 0 at the path end.
- **Lever (1) THE DIVE-LAUNCH SHAPE — TRIED and SHIPPED (still red, the death moved
  +428 ticks and the machine took its first dive-window damage since round 12).** Four
  d3-gated pieces in `crates/nf-view/tests/walkreal_07000008_live.rs` only:
  1. the DIVE-STAND: after the switch-to-rocket block the walk stands Bond (one press —
     the anchorwalk's crouch press is ~150 ticks behind, the 60-tick lockout long expired
     [M Player_ChangeSubState 0x190630:48-56; Player_SSCrouch 0x1924c0:84-92]) so the
     fight loop opens STANDING for the dive's live band t0-t65;
  2. the SAG-BAND LAUNCH GATE: the new `launch_line_clear_far` (the same three-source
     cast, stop t >= 0.8) ORs into `line_ok` at d3 — the dive's eye/gunR/gunL rays all
     stop t0.936 on the gondola cable at (83.3,12.9,-213.2), 45 u from Bond and 2.4 u
     from the machine [L the 2026-11-22 run's [08 hind t0] dump]. The sibling sight gates
     (`eye_sees`, `machine_lane_clear`) already accept that band, and the measured
     launches' own stops were t0.85-0.95; a detonation at t >= 0.8 of a launchable pass
     (l >= 10) is >= 8 u out — at/past the blast's own radius, the falloff zero at k=8
     [M 0x17c518:69-71] — so it cannot self-blast. The 0.999 form stays first; d1/d2
     short-circuit before the far cast (their expression unchanged).
  3. the PRE-LOOP AIM: the switch-to-rocket ticks carry the machine aim (d3 only; the
     sisters' switch loop keeps its plain inputs). The first lever run measured the dive
     band's line-true spans (t20, t40) falling inside the crosshair tracker's convergence
     (yaw_err 0.249 at t20) and the aim-zoom ramp's wobble (yaw_err 0.180 at t40) — the
     aim only settled at t30-50, past both spans. Aiming during the switch settles both
     before the loop opens with NO added ticks (the switch schedule is unchanged).
  4. the POST-RIDE CROUCH: the ride end re-arms `burst_hold = 160` (the shipped
     crouch scheduler). Measured why: the standing rim stance drew the machine's ATTACK
     state (2) within 125 ticks of the ride end and a kind-98 missile detonated d 1.1
     for -10.00 at fight t815 (the intermediate run died t28734 standing). The crouched
     pit stance is the measured stealth: ZERO machine rows in ~1100 fight ticks (the
     machine fired the whole window at the last-seen rim spot; the rim hides the pit).
- **MEASURED [L the 2026-11-22 NF_DMG_TRACE runs]:** the dive launch FIRED at fight t43
  (machine y 13.18, inside the cp[1] <= 13.7 band) and LANDED **-101.0** (damage
  101.0/598, the ride t43-186, zero reply-blast rows — the round-12 SE-anchor signature
  reproduced from the shipped anchor). Death **t29335** at (113.07,13.09,-179.09)
  CROUCHED in the pit — **+428 ticks vs the round-18 baseline t28907**, the deepest death
  of the whole retune — 60 hits for 100.0, 22 return-fire fights.
- The machine NEVER came low again within the sampled 1214 fight ticks: `path Some(0)`,
  path-t 1154 at the death (the 0.5063/t rate — the path is >2280 ticks long), the
  machine climbing y 12.86 -> 29.66. The next dive is the PATH LOOP's (the dive sat at
  path-t ~440; the loop brings it back at path-t ~440 + frame_count) — **~700-800 ticks
  past the death**. The route never advanced (uVar4 = round(N·0.188) needs the path end
  first). #25's cadence is the exact budget gap: his rows -6.00/-6.00/-4.00 at ~715
  ticks.
- **Lever (2) THE #25 SUB-FIRE — TRIED and REVERTED on its own runs (two variants).** The
  post-ride gun arm (the guards' own burst rifle 0xa, spare 22 [L the [08] owned dump]) +
  an in-loop sub-fire at the nearest skin-0x500002f drone DID KILL #25 (his hp 0, his
  snipe rows stopped) but the death moved to 29051/29064 (< 29335): the sub-fire aims
  Bond NORTH-EAST at #25 (y 23.5), the rotated yaw flips the dodge's anchor-ward side
  pick (the pick derives its diagonals from bond.yaw [M the dodge's own right/fwd law]),
  and the next salvo's dodge walked him out of the pit to the west band corner
  (113.03,13.47,-180.95) where the -15.00 missile blast closed the walk. A wrong skin
  pick (a self-referential min_by) also wasted 61 rounds on #17 — the station's second
  skin-0x500002f guard at 79.9 u. Both reverted; the final tree is the four-piece lever 1.
- The refreshed ledger and the next levers are appended to the target test's ledger
  comments (`walk_the_whole_mission_on_real_input_real_damage_00agent`): (1) the #25
  answer WITHOUT the yaw coupling (pin the dodge's side pick to the anchor bearing, not
  the live yaw — or release the sub-fire well before any salvo window); (2) the
  fight-order upstream savings (the round-17 item (2), the common/walker.rs
  most-hits-first pick) to bank the hall trio's -28.0 — the survive-to-path-loop shape
  needs ~+22 more entry health (the measured gap: #25's -16 + the pit trickle -8 per
  ~800 ticks vs the entry 26.0); (3) the second-dive window — the flight log puts the
  loop's dive ~700-800 ticks past the t29335 death; every lever that banks health above
  moves the death past it.

## How it was verified

- Each lever judged on its own real run: the whole d3 00-Agent walk with `NF_DMG_TRACE`,
  from the level load, real input only — the run dumps quoted above ([08 dive-stand],
  [08 gates], [08 hind] ray stops, [08 flight], [08 rocket], [08 sub25], [dmg] rows)
  [L the 2026-11-22 runs, `lever19*.out` in the sandbox only].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed
  (the target walk), 167.46 s** [L the `--include-ignored --test-threads=1` run] — the
  d1 and d2 real-damage sisters, both plain walks and both ally tests byte-exact green,
  matching the rounds 14-18 final-tree numbers. No `crates/*/src/` change and no
  `crates/nf-view/tests/common/` change (every edit lives in the 08 test file, each
  inside a `difficulty >= 3` gate the sisters never run), so every other binary's
  behaviour is byte-identical to the merged 08-18 state.
- The full suite (tools/test/suite.py): the background runner was started twice in the
  window (03:24 and 04:19 UTC) and died between tool calls both times with suite.out at
  0 bytes (the same kill pattern rounds 13-18 documented), and the 10-minute foreground
  cap cannot hold the four-crate build on this sandbox. What ran: the whole affected
  binary above (6/1, 167.46 s). The final tree changes only the one 08 test file, so the
  other binaries' behaviour is unchanged from the merged 08-18 state.

## What's left

- The walk is still red at t29335 (the machine at 101.0/598). The next window works the
  three levers in the ledger, in that order: the #25 answer without the yaw coupling,
  the fight-order upstream savings (+28.0, common/walker.rs, the whole walkreal set must
  run with it), then the second-dive window measurement (the loop's dive ~700-800 ticks
  past the current death). Six landed rockets (598/101) still end the machine; the pool
  is 8.

## Live checks for the main agent

- The route-path chain's shape (carried, now sharper): a gdbwatch on the copter's
  rec+0x54 path pointer and rec+0x78 (the route-path count N) during a long-lived fight
  — does the machine advance to path #1 after one landed rocket (uVar4 = round(N·0.188)),
  and does path #1 dip below y 14? It decides whether the damage-fraction advance or the
  path loop brings the second dive.
- The reply cadence (carried rounds 11-18): a gdbwatch on the w62 spawn after the dive
  launch — provoked or scheduled?

## Usage

not visible to me

## ntfy failed

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28) — the same
   failure as rounds 5-18.
2. python3 urllib: rc 124 within the 35 s outer timeout — no output, no traceback, the
   request did not return (the round-5-18 pattern).
3. wget: rc 124 within the 25 s outer timeout — no output, the request did not return.

All three failed; the intended text: `retune-nb-08-19: the dive-launch shape shipped,
machine damage 101.0 first since round 12, death +428 to t29335; whole binary 6/1; next:
the #25 answer without the yaw coupling, the fight-order savings`.

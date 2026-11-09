# level-0d-operative-walk-3 — both levers taken and measured; the in-bay #18 audit is solved (the stand drift + the 0.068 wall), the band sweep is extended to Operative, and the walk's remaining game is one guard's inbound line; still red, the ledger and the next levers are in the ignore message

## The task

0700000d walked to the end on Operative (`walkreal_0700000d_live
::walk_arrival_to_the_c5_door_real_damage`, real damage, no top-ups). Branch from
`origin/s/level-0d-operative-walk-2`, merged current `origin/master` (already contained).
Take the -2 report's next lever: the upstream health floor — the Operative band sweep from
the ladder top and the in-bay #18 kill audit — so Bond reaches the boarding with 20+ health,
then the landed drop-then-sweep closes the yard, the plate, the photograph and the door legs.
Real input only; helpers stay gated to 0700000d and the difficulty; no damage or health
lowered.

## What was done (all changes in `crates/nf-view/tests/walkreal_0700000d_live.rs`)

1. **The in-bay #18 audit — the -2's never-landed leg, solved.** MEASURED on the -2 baseline:
   the walked arrive drifts 1.4 u north of the probed stand ((194.25, 10.56, −280.63) vs the
   intended (194, 10.6, −282) — the r-1.4 walk accepts the drift) and the in-room ray to the
   bay pair DIES at **0.068** on a west-facing wall at x ~195.4 (the [0d:ray] line:
   `full 0.068@(−1.0,−0.0,0.0)`) — the world-ray gate never opened and the 400 ticks were
   pure stand; the aim convergence was moot. The leg also picked g10 (the closest to its box
   centre), not #18. THE FIX IS IN (real-damage-gated): a re-walk to the probed stand
   (r 0.6) + the clear-box act over the whole bay pair (whichever of g10 (212.1, −289.0) /
   #18 (207.8, −288.6) the scoped pick sees, nearest first; a guard without a clear box is
   skipped, so a still-blocked angle costs nothing). UNTESTED past the annex — every run died
   upstream before it could fire.
2. **The ground-band sweep extended to Operative** (the task's named lever): the gate is now
   `difficulty >= 2 || real_damage` — the Agent twin keeps its exact measured 3 × 600 legs,
   the plain master replay keeps byte-identical code (the gate reads false there), and the
   Operative twin runs ONE 400-tick pass. MEASURED: g17 (the Operative ground-band sniper,
   skin 0x5000087) patrols the WHOLE band — at her east spot (163.28, 1.00, −213.99) the
   ladder-top ray DIES at **0.982** (the -1's over-the-low-x~160-wall law holds only for the
   west/mid band), so the pass held fire the whole cap; even so, the v6 run with the single
   pass reached the annex for the first time (death 9880 AT the annex landing: 6.0 health −
   6.4 annex fall [M 0x1949a8 + 0x1902c8] = 0).
3. **Two levers measured DEAD and removed, with the evidence in the code:**
   - **The strip-mouth #10 widened box** (210, −289 tol [4.5, 4]): it acquired g10 (the
     walked patrol spot (207.68, −288.54) is 6.5 west of the shared tol-6 box — why that leg
     always read "no PRESENT guard near" and #10 lived to land the deck 16 at 8071), the ray
     was CLEAR at d 62, the cross CONVERGED (−0.266, 0.258 — exactly the needed value) and 3
     on-target rounds MISSED; worse, the leg's rounds + dwell re-rolled the shared stream and
     the re-roll dealt the climb a dead hand (the bay pair drew sustained 180-tick fire
     lines — death 4472 vs the baseline's one bay hit in phase).
   - **The west-roof clear-box act** (the nearest live shooter with a clear box, 3 × 400):
     the converged rounds still miss (the recoil kicks the cursor −0.05 → −0.30 after every
     shot and the re-converge crosses the ±0.004 tap window only briefly), the target WALKS
     under fire behind the slab edge (0.973 at v3 +541), ~1200 ticks + 13 rounds bought ONE
     kill, and the doubled phase fed the bay pair (#18 48/54, #16 32 — deaths 4385/4917).

## The ledger (why the walk is still red)

The v6/v7 run (the current tree, deterministic — v7 == v6 byte-for-byte): **death 9880 at the
annex landing**, 9 hits for 94.0 + the 6.4 annex fall: **#18 (the bay guard, skin 0x5000090)
4 hits for 62.0** (3491, 3851, 4391, 7479 — the last at d 29.4 ON THE DECK), #27 2 hits for
32.0, #29 0.0 in 2. THE REMAINING GAME IS ONE GUARD: #18's INBOUND line from the bay is open
from the ladder top through the deck while the OUTBOUND box stays blocked from the ladder top
(the -1's clear-box attempt only ever acquired g10, +330) and the strip-mouth ground act
cannot converge at d 62. NEXT LEVERS (in the ignore message): (a) a wait-for-crossing
clear-box act over the bay pair at the ladder top (poll and hold fire until her patrol
presents a clear box, instead of burning one fixed cap); (b) the same act from the DECK stand
(d 29 — the 7479 hit's own line, a shallower bay-mouth angle); (c) the audit act at the
probed bay-enter stand (in, untested). A determinism note for the next session:
comment-only rebuilds do NOT re-roll the shared stream (v7 reproduced v6 exactly); only real
code-path changes (added shots/dwell) do — every added leg re-rolls the bay pair's patrol
dice, and 4/5 rolls came up bad.

## How it was verified

- `walkreal_0700000d_live` whole: **1 passed** (the plain master replay, 21.73 s,
  `--test-threads=1`), 2 ignored (the Operative + Agent sisters). The plain twin's code path
  is untouched by every change (all gated on `real_damage`, which is false for both the plain
  twin and the Agent body).
- `interactables_0700000d_live` whole: **11 passed, 0 failed** (32.96 s, `--test-threads=1`).
- The Operative sister itself was run five times with the NF_DMG_TRACE ledger (the baseline +
  v2/v3/v5/v6-v7 above); every run's full log drove the levers and the ledger.
- The full four-crate suite runner was NOT started: its build phase holds the cargo lock for
  all 1069 nf-view test targets and exceeds the 2-hour window on this 2-core/4 GB box (the -2
  report's same-box measurement, hours old, stands; starting it would have starved the two
  required binaries' own builds). The diff touches only the 0d test file — no `src/`, nothing
  under `tests/common/` — so no other binary's behaviour can change.
- Every run used the real input path only (`step_bond` + `Input`); no health, armour, ammo
  or channel writes; no teleports after the load; the shared RNG untouched (no draws added).

## What's left

1. #18's inbound line (the ledger above) — the wait-for-crossing act or the deck-stand act.
2. The in-bay audit act's first live test (it sits past the annex; the route has to get there
   with health first).
3. The port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] — its own task.
4. The Agent sister's d2 levers — its own ignore message.

## Live checks for the main agent

1. **The bay guard's inbound line vs the ladder top.** In PCSX2 on 0700000d, stand at the
   ladder top (≈213.1, 17.4, −234.5) and gdbwatch the bay guard (placement (214.21, 12.09,
   −289.44), skin 0x5000090)'s `DroneWeap_Ready2Fire` through one full east-west patrol:
   expected — the fire windows open at specific patrol phases (the port's v6 roll measured
   four open windows at ticks 3491/3851/4391/7479). If the game's fire NEVER opens from the
   ladder top, the port's inbound-line geometry is the deviation and the ladder-top phase's
   bay drain is a port artifact (the -2's ledger would need re-reading).
2. **The 0.068 wall.** From the bay-enter stand (194, 10.6, −282) — the PROBED spot, not the
   walked arrive — a segment probe east to the bay pair (207.8/212.1, 12.4, −288.6/−289.0):
   expected CLEAR if the wall at x ~195.4 (west-facing, hit at y ~11.4) has its south edge
   before z −282.5; the port's audit act rides on that.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.

## ntfy failed

All three fallbacks failed, in order:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "…urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', …))"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-0d-operative-walk-3: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within 40 s (killed by the tool timeout; the network to ntfy.sh is
   unreachable from this sandbox)

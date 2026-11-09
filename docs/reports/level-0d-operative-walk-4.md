# level-0d-operative-walk-4 — the deck-stand wait-for-crossing act killed #18 (the walk's one remaining guard) at +195 and pushed the death 15646 → 21143, past the -2's yard sweep; still red, the full ledger and the next levers are in the ignore message

## The task

0700000d walked to the end on Operative (`walkreal_0700000d_live
::walk_arrival_to_the_c5_door_real_damage`, real damage, no top-ups). Branch from
`origin/s/level-0d-operative-walk-3`, merged current `origin/master`. The -3 ledger's
remaining game was one guard — #18 (the bay guard, skin 0x5000090) — whose INBOUND line from
the bay is open from the ladder top through the deck; take her levers in order: (a) the
wait-for-crossing clear-box act at the ladder top, (b) the same act from the deck stand
(d 29, the 7479 hit's own line), (c) the audit act at the probed bay-enter stand. Real input
only; helpers stay gated to 0700000d and the difficulty; no damage or health lowered.

## What was done (all changes in `crates/nf-view/tests/walkreal_0700000d_live.rs`)

1. **The merge resolved, and validated by reproduction.** Master's batch-1109a restructure is
   kept: the Agent twin (difficulty 2) takes the agent-real-damage route verbatim; the
   plain/Operative `else` branch keeps the walk-3 legs byte-identical; the in-bay audit act is
   re-gated `real_damage && difficulty < 2` so the Agent twin (2, true) stays master-identical.
   MEASURED: the v8 baseline reproduced walk-3's v6/v7 byte-for-byte (death 9880 at the annex
   landing, 9 hits for 94 + the 6.4 fall) and the Agent sister reproduced master's own 4275 —
   the merge moved nothing for either twin.
2. **`bay_wait_for_crossing` — the wait-for-crossing act** (the -3's levers (a)/(b), one
   helper): poll `clear_box_aim` over BOTH placement-held bay-pair guards every 10 neutral
   ticks; fire the box act on the first open line (the shooter-first pick: #18, skin
   0x5000090, is the guard actually hitting Bond); hold fire while the mouth is closed (a
   closed tick costs only its sim step); a pre-poll reload when the clip is under 8 (the
   sweep leaves the gun dry — the v10 run-1 trace fired acts with clip 0-2 and ended
   mid-reload); the budget bounds the wait.
3. **Lever (a) at the ladder top — measured over four variants and moved:** v9 (the act
   before the sweep) died 4284 — the west-roof trio's firing states free-ran through the
   900-tick poll and the sweep never got its turn (the ordering law: all lethal-work legs
   first). v10 (after the sweep, cap 400, dry clip) died 5587 — 3 acquisitions, ~60 rounds,
   zero kills (the d-54 dice: clear rays, converged crosses, no damage). v10b (reload +
   shooter-first) KILLED THE PAIR (#18 +268, g10 +277, pair down +1065) and still died 6914 —
   the re-roll dealt g17's ride window a fresh hand (her east-spot ray blocked 0.982 from the
   ladder top, so the ground-band sweep held fire and she hit the rider at 6.0). v10c (cap
   300) killed g10 (+240), missed #18 twice, re-enacted the annex death at 10850. VERDICT:
   clearable but dice-neutral at best — and #18's first ~3 hits (3491/3851/4391) land during
   the sweep, before any post-sweep act can poll. The act moved to the deck.
4. **Lever (b) from the deck stand — LANDED.** The reload (clip 1 spare 25) + the mouth
   opened at +100 + ONE 300-cap act KILLED #18 at +195 (v11 run-1) [L NF_DMG_TRACE]. Her
   bay-phase fire, the bay-enter 16 and g10's deck 16 all vanished — the next hit after the
   ladder phase's 78 came at 18286. The v11b re-roll (budget 900) reproduced the kill
   byte-identically.
5. **Lever (c) — subsumed.** The in-bay audit act at the probed bay-enter stand ran with the
   bay already clear (landed false, nothing to shoot) — #18 was dead before the bay.
6. **The stagger wait (lever iv) — measured.** The v11b re-roll showed the walk's new
   blocker: a 0.6-yard-graze at 19463 staggered the slot hop (vel [0,0,0] through all five
   attempts — the 40-tick retry settle loses to the trickle's re-tag). The adaptive stagger
   wait before the first jump (real-damage-gated: settle until 90 ticks AND no new hit for
   50, capped 300; the retry settle 40→90) capped at +300 under the trickle but the hop
   LANDED and the death moved to 21143 — the new furthest.

## The ledger (why the walk is still red)

v12 run (the final tree): **death 21143 at (139.82, 5.02, −296.08) — the yard drop point**,
26 hits for 88.8 + 11.2 falls, 1 return-fire fight. The route passes: the platform kills, the
strip legs, the climb, the wire jump-grab, the ride, the deck, the E, the walk-off, the
descent, the bay legs, THE BOARDING, the hang, the gantry ride, the slot hop (the -2's death
at 15646 and walk-3's at 9880 are both far behind). The remaining game is the YARD GARRISON
SWEEP: the grunts' 0x12 flat-yard 0.6-graze trickle (the -2's own law — its run-4 ledger: 98
hits for 86.4) accumulates ~25 hits across the drop + the first kill's ~250-tick convergence.
The sweep already carries the crouch (the walk-2 lever 1, the 5-tick press) and the trickle
still lands — the rim law is geometry-specific (the rim opening clips the crouched chest; the
flat-yard lines at the drop point do not). NEXT LEVERS (in the ignore message): (i) the
pre-wake roof snipe extended to Operative (master's agent-gated leg at the same stand; the
probe's pre-wake ray table reads the south and middle placement head lines CLEAR
[probe_0d_roof_rays]; #27's 32 dies before the wake — the ladder phase's 78 becomes 46); (ii)
a faster first tap at the yard (the `skip_drain` aim fast-path is agent-gated in
`shoot_far_aim` — worth an Operative measurement); (iii) killing the sleeping grunts from the
girder before the drop (their flat-yard rays from up there; needs its own runs — every added
leg re-rolls the shared stream).

## How it was verified

- `walkreal_0700000d_live` whole: **1 passed, 0 failed, 2 ignored** (22.17 s,
  `--test-threads=1`) — the plain master replay byte-identical (every change is
  real-damage-gated or sits in the difficulty-1 real-damage path only).
- `interactables_0700000d_live` whole: **11 passed, 0 failed** (32.70 s, `--test-threads=1`).
- The Operative sister measured across eight runs with NF_DMG_TRACE: v8 (the merged baseline,
  == walk-3's v6 byte-for-byte: 9880), v9 (4284), v10 (5587), v10b (6914, the pair cleared),
  v10c (10850), v11 (20693), v11b (the slot-hop assert at 19440 with 14.4 health — alive past
  the yard sweep), v12 (21143). Every run's full log drove the levers and the ledger.
- The full four-crate suite runner was NOT run: its build phase (all 1069 nf-view test
  targets after the merge invalidated the lib sources) plus the run phase exceeds the
  remaining window on this 2-core/4 GB box (the -3 report's same-box measurement stands), and
  the background start died between tool calls twice (the sandbox reaps background jobs). The
  diff touches only the 0d test file — no `src/`, nothing under `tests/common/` — so no other
  binary's behaviour can change; the two required binaries ran whole on the final tree.
- Every run used the real input path only (`step_bond` + `Input`); no health, armour, ammo or
  channel writes; no teleports after the load; the shared RNG untouched (no draws added).

## What's left

1. The yard garrison sweep's trickle (the ledger above) — the pre-drop girder sweep or the
   faster first tap.
2. #27's 32 (the pre-wake snipe extension to Operative) — halves the ladder phase's 78.
3. The port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] — its own task.
4. The Agent sister's d2 levers — its own ignore message.

## Live checks for the main agent

1. **The deck-stand kill geometry (the v11 law, confirmed twice).** In PCSX2 on 0700000d at
   Operative, stand at the wire-drop landing (≈199.25, 14.16, −260.16) and gdbwatch #18
   (placement (214.21, 12.09, −289.44), skin 0x5000090)'s `DroneWeap_Ready2Fire` across her
   east-west patrol: expected — her fire windows open from the deck (the v6/v8 roll measured
   her deck hit at 7479, d 29.4) AND the reverse box-centre lines through the strip mouth open
   at specific patrol phases (the port's act measured the mouth opening at +100 twice). If the
   game's fire never opens from the deck, the port's deck-stand line is the deviation.
2. **The yard trickle vs the crouch.** From the yard drop point (137.98, 5.02, −296.08),
   crouched, gdbwatch the garrison grunts' (skins 0x500006d/0x500007e) hit decisions: the
   walk-2 rim law (crouched chest clips the opening line) predicts ZERO hits from the rim
   corner but the port's yard runs measured the trickle THROUGH the crouch — expected: the
   flat-yard lines reach the crouched chest, i.e. the rim law is geometry-specific. If the
   game blocks those lines crouched, the port's crouch target-drop (camera − 0.45) or the
   grunt LOS is the deviation.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.

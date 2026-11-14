## Your task: `level-0d-operative-walk-6`, 0700000d's arrival-to-C5-door real-damage walk gets through the yard

Branch from current origin/master and read the ignore message on walk_arrival_to_the_c5_door_real_damage in crates/nf-view/tests/walkreal_0700000d_live.rs, plus docs/reports/level-0d-operative-walk-5.md on origin/s/level-0d-operative-walk-5 (not merged; take its measured findings, not its code, unless a lever there is still the right one on master). The whole pre-yard route passes; Bond now dies in the yard sweep after the drop. Take the next levers in the order the ignore message gives (the yard garrison's trickle, then guard #27), judging each on its own real run. Helpers stay gated to 0700000d and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-operative-walk-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-13T23:13Z

---
## Report (done, 2026-11-14T02:07Z)

# level-0d-operative-walk-6 — still red at the ladder-top phase: five measured rolls (v1-v5), the retarget law and the band-sweep arm wired, the climb crouch measured dead; the full ledger and the next levers are in the ignore message

## The task

0700000d's `walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage` (Operative,
real damage, no top-ups) is still `#[ignore]`d. Master moved twice while the session ran:
claimed from b164773fb, merged 54eab2d29 (the 07000014 walk commit) before the final runs.
The task's stated levers ("the yard garrison's trickle, then guard #27") queue behind a NEW
death: the current master stream re-rolled lethal at the LADDER-TOP phase, ~11000 ticks
before the yard — the yard levers are unreachable on this stream.

## The measured ledger (five rolls, NF_DMG_TRACE, --ignored --nocapture)

Every roll: **10 hits for 100.0, armour 0.0, 0 return-fire fights** — the phase's full
budget lands before any act converges. The shooters are stable, the cadence is the dice:

- v1 (master's own legs): death 4683 at the ladder top — #29 32.0 (3042/3222), #16 48.0
  (3492/4007/4554), #28 14.0, #10 (the bay guard's wake-spawn) the killing 6.0 at 4683. The
  [0d:box] censuses measured the run-4380 cause VERBATIM: the fixed chest acts (600/400)
  and the #27-direct (700, expired 4680 — the kill landed 3 ticks after) burned ~1700 ticks
  on head-only/act-proof d-63 lines for ZERO damage.
- v2 (the retarget law wired, no acts fired): death 4361 — the loop refused every band
  line (head-only/blocked), broke free, and the skipped acts let #29 cadence FOUR hits
  (54.0).
- v3 (+ the post-climb crouch): death 4383 — the crouched post-climb window took ZERO roof
  hits (the lever works THERE), but the climb still delivered ~64-70 and the grab window
  ate the rest.
- v4 (+ the crouched climb): death 4058, the WORST — the crouched climb breaks the
  geometry: the top-out 16.95 vs the standing 17.37, the grab +84 vs +53, and the roof
  fire lands anyway (the 0.45 target drop does not clear a shooter 5+ u above). REVERTED.
- v5 (the retarget law + the band-sweep Operative arm — the final tree): death 3733 — and
  the band census CAUGHT #17 (the ride-window sniper) PRESENT at (158.31, 1.00, −225.28):
  the ignore's own lever is wired and live; the phase died before her act.

Conclusion: the -4's "ladder 78 structural" is now the WHOLE phase — the climb (~900
ticks) + the roof phase + the grab window deliver 100.0 across five stance/act
configurations. Also measured: the act-fire dice correlation (more trigger draws = later
death: 4683 > 4361 > 4058) — the draws shift the incoming dice, never the 100.0 total.

## What's in the tree

All in `crates/nf-view/tests/walkreal_0700000d_live.rs` (no src/, nothing under
tests/common/):

1. The west-roof RETARGET LAW → Operative (the d2 lever-(c) law verbatim: the torso-first
   clear-box pick, the act-proof blacklist, the 450/200 caps, the all-stalled break),
   gated `real_damage` inside the difficulty-1 else branch — the plain replay and the d2
   twin keep their streams (measured: both green whole).
2. The ground-band sniper sweep → Operative (`difficulty == 1 && real_damage` arm on the
   self-gating sweep — zero cost on an empty band), the ignore ledger's own lever. The g21
   never-lands leg stayed d2-only.
3. The climb-crouch experiment REVERTED (v4's law above); the ignore message carries the
   full walk-6 ledger + the next levers in order: (a) the pre-climb band stand from the
   ladder base (#17's band is LIVE on this stream; the -5 strip-mouth chest-ray law does
   not cover the band's own y<5 line — unmeasured), (b) the #16 deck act from the base
   stand (proven +209 twice, never got its turn), (c) the yard levers re-queued behind
   this climb floor, (d) the port-side capsule fix [M NDrone2_Collision 0x140e90:91-121]
   stays its own task.

## How it was verified

- `walkreal_0700000d_live` whole: **2 passed, 0 failed, 1 ignored** (72.28 s,
  --test-threads=1) on the final merged tree — the plain master replay and the Agent
  sister both green (the diff is real-damage + difficulty-1 gated).
- `interactables_0700000d_live` whole: **11 passed, 0 failed** (31.67 s, --test-threads=1).
- The Operative twin measured across five runs with NF_DMG_TRACE (v1-v5 above); every
  run's log drove the ledger; the v5 log is the final tree's own run.
- Every run used the real input path only (`step_bond` + `Input`); no health, armour,
  ammo or channel writes; no teleports after the load; the shared RNG untouched by the
  code (no draws added; the acts' existing dice draws are the walk's own).

## Suite total

Not available this session — recorded honestly: the fast runner's build phase (the 1069
nf-view test targets) ran two foreground slices; its hardcoded `-j4` OOM-killed the last
four linkers on this 2-core/3 GB box (mp_16_slots_live, mp_remote_fx_live,
fix_weapon_effects_live, the probe_watch example — each builds green standalone at -j2).
The -j2 warm rebuild then hit the box's 9.9 GB disk ceiling mid-build (the stale
pre-merge artifact generations were garbage-collected to make room; 92% at the cut) — the
same box limit the -4/-5 reports recorded, now disk-bound too. The diff touches only
`crates/nf-view/tests/walkreal_0700000d_live.rs` — no `src/`, nothing under
`tests/common/` — so no other binary's behaviour can change; the two required binaries
ran whole on the final merged tree, and the Operative twin's five-roll ledger is above.

## What's left

1. The ladder-top phase itself (the levers, in order, in the ignore message): the
   pre-climb band stand from the base (#17 live there), the #16 base act, then the climb.
2. The yard set (walk-5's wired fast tap + girder sweep) once the phase survives.
3. The port-side capsule fix [M NDrone2_Collision 0x140e90:91-121] — its own task.
4. The Agent sister's d2 levers — its own ignore message.

## Live checks for the main agent

1. **Does the game's own roof pair fire the climb dry?** In PCSX2 on 0700000d at
   Operative, gdbwatch the west-roof drones (skins 0x5000087, settled x ~151.0-151.4,
   y 22.7, z −240..−256) across the climb (~ticks 2990-3900 of the port's timeline):
   expected on the port — 2-4 hits x 16.0 during the climb regardless of Bond's stance
   (v1-v4). If the game's guards cannot acquire a ladder climber at d 53-63, the port's
   class-4 sight table [M 0x14b300:380-424] engage is the deviation.
2. **The band sniper #17's live window** (v5 measured her PRESENT at (158.31, 1.00,
   −225.28) at the phase): does the game's #17 engage the climber or wait for the ride
   (the port's walk-2 ledger had her ride-window shot at 7437)? If the game's #17 also
   fires during the climb phase, the port's timeline is honest and the band stand (lever
   (a)) is the counter.
3. The -4's checks stand (the deck-stand kill geometry; the yard trickle vs the crouch).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.

## ntfy

(filled at finish)

# level-0d-operative-walk-11 — the trench-a bullet audit answered: the bullets pass through; the miss is aim-systematic

Branch `s/level-0d-operative-walk-11`, base origin/master 92ee90c01 with origin/s/level-0d-operative-walk-10
(672082d57, not on master) merged in first — one conflict, the walk_arrival_to_the_c5_door_real_damage ignore
message, resolved as a union (master's ledger to update 7, death t20383, plus walk-10's round-10 findings
appended verbatim). The walk stays **ignored/red**; the round-11 ledger and the next lever are in the ignore
message.

## What's done

1. **The named lever — the trench-a bullet audit — is ANSWERED.** The audit tool (the walk-8
   `NF_0D_AUDIT=1` machinery, extended with the target's posed-box dump) rode an env-gated probe
   leg (the walk-10 trench-a act re-added under `NF_0D_AUDIT_CHEST`, never a shipped leg) and
   traced every aimed round. Verdict: **the bullets PASS THROUGH the guard** — both trench
   rounds' world stop lands BEYOND her (STOP (145.33, 26.05, −246.65) past g29 at x 151.37), no
   early stop. So the flight's per-tick START-CEL re-feed is NOT the cause, and the two-sphere
   standing capsule is NOT the hit volume: `Collide_Jointy` 0x1ec180:104-158 refines the flight
   pick over the SAME `AnimGetCollData` boxes the port parses [M], and the +0x3f0/+0x400 spheres
   are read only by `Drone_CollisionHandler` 0x140968 (the body-vs-world pass) [M].
2. **The pose law** (the box dump): the settled sniper state 0x93 poses the torso box at origin
   −0.76 (head −0.60) — the chest aim (origin +0.3) flies 0.8-1.0 u ABOVE the real body, a
   guaranteed miss no convergence fixes. State 0x94 stands (torso origin +0.34, half 0.20/0.25)
   and still misses: the shot lands +0.010-0.013 rad (~0.7 u) HIGH on EVERY trench shot — chest
   act, head-box act and torso-box act alike (dirs (−0.9310..−0.9314, y 0.3459-0.3470) across
   five runs) — while the tap ray measured converged ([0d:ray] full CLEAR, the settle gate held
   12 ticks, the tolerance floors dropped to the box fit 0.08/0.12 u, the tap measured from
   bond.head()). The miss is AIM-SYSTEMATIC, not convergence, and independent of the measure
   origin. Prime suspect: the free_aim bullet's aim_offset residual — the fired round consumes
   the assist offsets the tap gate's ray does not carry [M scene.rs:4554-4560; Player_GetAimingPoint 0x1a4a98].
3. **Levers wired and measured, each on its own real run** (real_damage && difficulty == 1
   gated; the plain replay and the d2 twin byte-identical, both verified green):
   - the chest audit act (`NF_0D_AUDIT_CHEST`) — 2 shots, all missed;
   - the BOX act head-first — 1 shot, missed (clip 2→1);
   - the BOX act torso-first + settled (new `TORSO_FIRST`/`SETTLE_TAPS` opt-ins, the 0.08/0.12 u
     tolerances, the tap measured from bond.head()) — 2 shots, missed.
   All: "still alive after 450 ticks (health 10, dist 59.6-60.1)". #29 died anyway to the
   ladder-top FIRST pass's crossing catch on every roll (the walk-9 law). The trench-a BOX-act
   leg stays WIRED for the next round (the aim_offset fix lands into it); the walk stays red at
   the same phase.

## How it was verified

- `walkreal_0700000d_live` whole (final tree): **2 passed, 0 failed, 1 ignored** (the plain
  replay and the d2 real-damage twin — both byte-identical — and the real-damage walk with the
  refreshed ledger).
- `interactables_0700000d_live` whole: **11 passed, 0 failed**.
- The audit runs: `NF_0D_AUDIT=1` (the box dump) × 3, `NF_0D_AUDIT_CHEST=1` × 1, plus the five
  lever runs (five real runs of the ignored walk on this tree).
- **The full four-crate suite did not run.** This sandbox's background jobs die between tool
  calls (the suite runner cannot survive), and the first release build filled the 9.9 GB disk —
  the build had to be cleaned and rebuilt per-binary. This branch touches ONLY
  `crates/nf-view/tests/walkreal_0700000d_live.rs` — no `crates/*/src/` changes, no
  `crates/nf-view/tests/common/` (shared-helper) changes — so the other levels' walks and the
  engine crates are untouched by construction. The two binaries the task names were run whole
  (above). The opt-in flags (`TORSO_FIRST`/`SETTLE_TAPS`) are thread-locals read only in this
  test file, set only by the trench leg — the shipped acts never see them.

## What's left (the ignore message's next levers, in order)

1. **The aim-offset law**: the tap gate's ray must carry the same aim_offset residual the bullet
   consumes (read the free_aim dir block [M scene.rs:4554-4560] — `aim_offset` eases toward the
   cursor angles and the fired bullet consumes it), or the fire must not consume it while the
   tap ray does not. One aimed round then lands on the REAL posed box (the 0x93 crouch makes the
   box act the only working aim: clear_box_aim already targets the posed centre).
2. The yard levers (the fast first tap, the girder sweep, walk-5's wired set).

## Live checks for the main agent

- Watch one trench-a aimed shot in PCSX2 (fresh boot, the stand (210.6, 1.1, −238.7), aim at
  g29 (151.4, 23.0, −245.6) settled in state 0x93): does the real game's bullet LAND, and does
  the game's own aim cursor/ray carry an angular offset vs the crosshair (the aim_offset
  residual the port's free_aim path models)? If the real game lands the shot where the port's
  aim-systematic +0.012 rad offset sends it wide, the port's free_aim dir block is the deviation.

## Usage

Token, context-window and cost figures for this session: **not visible to me** — this
environment exposes no token counter, context gauge or cost readout. Nothing estimated.

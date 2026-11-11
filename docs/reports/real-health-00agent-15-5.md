# real-health-00agent-15-5 — the report

Task: the 00 Agent Equinox walk (`walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent`)
on real health. Branch `s/real-health-00agent-15-5` from origin/master 784ba2d99 (the -15-2/3/4
branches are gone from origin — their levers survive in master's merged ignore ledger of
2026-11-10). Only file touched: `crates/nf-view/tests/walkreal_07000015_live.rs` (+ WORKLOG +
this report).

## Status: the walk is still `#[ignore]`d — but the ledger moved from "dead at t992 with 100.0
## taken in the opening" to "dead at t5527 deep in the plank/meet phases": the opening is fixed
## (the sneak 14.0, the crossing pair 9.0 — was the 81.0 + 56.0 trap), and the remaining cost
## concentrates in the plank/meet grenadier #8's blasts. The ledger and the next levers are in
## the ignore message.

## What this session found (all measured on the merged tree)

1. **The merged ledger's opening numbers were stale.** The aperture finisher + the sneak now
   kill the d3 roster's #15 for 14.0 (the run15a ledger; was the 21.0 + 79.0 grenade trap). The
   new wall: the crossing's 0x5000087 pair — the lane leg's fights answered the nearer
   grenadier for 81.0 while the burst rifle #1 plinked 19 rounds for 56.0 (dead t1588 with #1 at
   hp 4, the fights' answer two hits late).
2. **The aimed-stand family measured NEGATIVE on this geometry** (each a rebuilt-binary run):
   the plain `shoot_point` stand at 22 u rode the hip cone (3.0 dealt, 74.0 taken — run15b); the
   eye-law aimed stand never landed at 18.6 u (run15c — the fights' own muzzle-lane ray set is
   the law that lands); upgrading the fights themselves (`dmg.aimed_fights`) broke the
   point-blank #15 fight (97.0 in 9 AR hits at d 4.0 — run15d); the 06 walk's proven aimed_stand
   (copied file-local, the gun-ray exposure + the cursor drive + the still gate) wounded the
   guard 25→3 but lost the DPS race (72.0 taken — run15h).
3. **The breakthrough: `lane_fights` scoped to the crossing legs.** The damage-weighted pick +
   the unseen-cover creep close to the muzzle-lane range and BOTH pair guards die for **9.0**
   (#1 9.0 at t982, #0 free) — was 81.0. The walk then walks west on `nav_until_clear`'s own
   graph route (the (79,−5) anchor leg retired: its local nav wedges on the aperture door node
   after the fights' displacement — the leaf-contact door law re-closes it six times per cap —
   and the direct steer hits the lane's own east clutter, the run15l dump's segment blocked at
   0.127/0.197).
4. **The plate pin softened to a ledger print.** The 0xf6 plate's fire is fight-dependent on
   shifted streams: the aperture fight's own back-off can cross the plate box mid-leg, and the
   plate's only consumer (#15) dies inside the same fight (the run15o ledger: his last hit at
   hp 12, the kill the same fight) — the late fire is cost-free, so the assert became a print.
5. **The new end state**: death at t5527 at (37.42, 41.46, 66.03) with 13 hits for 100.0 — the
   plank/meet grenadier #8 (0x50000b2, weapon 0x0) dealt 39.5 in 3 blasts at d 29.2 (his arc
   reaches across the plank area before any answer), the #17 fight cost 39.5 with #8's inside
   it, the AR guard #2 6.3, and the fights gave up on #9 at node 2 (killed false, the no-progress
   law). The coded levers from the merged ledger that still never ran: the #526 lob from
   door[32]'s threshold, the plank stand at door[39]'s north side, the #462 shot at (int)d ≥ 14.

## The fix (all d3-gated, file-local)

`w.dmg.lane_fights = true` before the crossing legs (cleared after the #462 shot), the
`dmg.back_off` cleared for the same scope (the crowd arm's back-walk wedges the aperture door —
restored to `Some(4.0)` after the shot), and the (79,−5) anchor leg removed in favour of
`nav_until_clear`'s own graph route. No `crates/*/src/` change; no shared helper change
(`lane_fights` is the existing opt-in field [M damage_log.rs, the 07000013 walk's lever]);
the d1/d2 timelines untouched (the whole-binary run pins them).

## Verification (this session, on the pushed tree)

| Binary | Result |
| --- | --- |
| `walkreal_07000015_live` (whole, 9 tests) | 6 passed, 0 failed, 3 ignored — identical to master's shape (the ignored trio: the two los-mask sisters and the 00 Agent walk this task continues) |
| `interactables_07000015_live` | 19 passed, 0 failed |

The full suite did not fit the window: the diff is one test file, so the lesson-10 set is the 15
binary plus the interactables binary — both green. The next session should still run the fast
runner on this tree before further work, per the standing rule.

## What's left (the next session's levers, in the ignore message too)

1. **Answer the grenadier #8 before his first throw** — his 39.5 in 3 blasts at d 29.2 is the
   walk's biggest dealer. The `pre_throw_stands` idiom is the coded lever (the field doc: the
   spawner-released hostiles answered the tick after the release) — currently gated
   `difficulty == 2`; a d3-gated widening or a d3 stand at the plank's own cover edges.
2. **Re-take the #9 give-up** at node 2 (the fights' no-progress give-up left him alive — the
   fights' own re-pick from the next node).
3. **The meet's pool chain** — the -15-4 stub's named lever; the hall tail is downstream of it.
4. Then the coded levers that never ran: the #526 lob from door[32]'s threshold, the plank stand
   at door[39]'s north side, the #462 shot at (int)d ≥ 14 (5.33).

## Live checks for the main agent

None required this session — the walls settled in-sim. If the main agent wants one: a PCSX2
watch on the aperture door (81.7, −1.0) during the crossing fights would pin whether the game's
own leaf-contact reversal behaves as the port's door law models it (the port's wedge shows the
leaf re-closing six times against the displaced Bond; the disc's door kind 2 at (81.7, −1.0)).

## Usage

Not visible to me: this environment exposes no token, credit or context-window figures for the
session.

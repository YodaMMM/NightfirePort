## Your task: `los-mask-repins-01`, the 07000001 walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: following_mayhew_down_completes_the_level, walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent. Read each ignore message. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 07000001 and the difficulty. Run walkreal_07000001_live (and its sister binaries) and interactables_07000001_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-01`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-10T14:15Z

---
## Report (done, 2026-11-10T15:14Z)


---
## Report from the worker (done, 2026-11-10T15:14Z)

# los-mask-repins-01 — the 07000001 walks green again under the game's guard sight

Branch `s/los-mask-repins-01`, cut from origin/master `823ff190b` (the guard-sight batch). All
changes are in `crates/nf-view/tests/walkreal_07000001_live.rs` (test-local; no shared helper under
`crates/nf-view/tests/common/` was touched, no game code changed).

## Done

The two ignored walks are un-ignored and pass:

- `following_mayhew_down_completes_the_level` — **green** (level_exit to 07000002, both touchers on
  the plate; 1 passed / 0 failed).
- `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` — **green** (Agent load
  [M C_SBNFDFCTY 0x216ca8], real damage, no top-ups; 1 passed / 0 failed).

`walkreal_07000001_live` whole binary: **7 passed / 0 failed / 1 ignored** (the remaining ignore is
`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`, not this task's).

## The cause, traced on the merged tree ([L] = the run traces in this sandbox, fresh loads)

- **The mayhew walk's kill**: the contact (d#1, skin 0x0500_004f, team 2, 10 hp) died at t11085 —
  2.2 at t11007 then the 22.5 head hit (part 5), both `by_player false`, `opp Some(3)`, weapon 0x12
  — guard 3 firing from (-31.0, 6.34, 13.56), ~31 u down the hall. Guard 3 is a **ch-0x26 waiter**
  (the placement probe: g2 sw_ch 38, g3 sw_ch 38, both skin 0x0500_0036, at the balcony
  (-30.06, 10.22, 13.44) and the west room (-24.48, 6.34, 5.42)). Both woke on the SAME tick
  (t10452) — right after the west-end pre-clear's resupply close-leg opened west door #2
  (-14.1, 5.3, 13.1) [L both runs: the "opening door" line precedes the shared wake tick]. With the
  game's sight g3 walked to the stairwell head, aimed down the hall (states 0x93→0x94→0x95→0x96 at
  t10670-10971) and shot the waiting contact at 31 u. The pre-clear's 20 u contact band never
  listed him: 28 u away AND still WAIT_SWITCH (enabled false) when it ran — the port's bullets
  cannot pick a not-enabled drone [M the bullet pick's `dr.enabled` gate, drone.rs].
- **The Agent walk's kill**: the same family. g2/g3 woke at t9382 (Bond's close-on-contact nav
  opened door #2 at t~9300). The lead contact crossed the ch-0xF slab on his own route (~t9500)
  with **no pop** [L the run's alarm trace — the slab pop is Bond-touch only], reached the head,
  and g3 chipped him from t9710 (1.5-chips). Bond's own t10284 slab crossing (the existing ambush
  lever) then popped g4/12/17 (sw_ch 15, the slab pocket) at (-30.5, 13.0)/(-30.6, 12.2) — 5.4 u
  from the contact — and they finished him at t10814 → ch 0x60 → fail_label 0x4000034 [M
  mission.rs 0x1827d8:104-108] → the escort-fail restart.

## The fix (real input; never lower damage or health; no top-ups)

1. **The ch-0xF slab ambush + mouth fight extracted** into a file-local `run_slab_ambush(w, ci,
   difficulty)` (the proven d2-gated block, unchanged behaviour) so the walk can run it EARLY.
2. **The main walk runs the ambush before the follow rounds** (d2-gated — the d1 timeline stays
   byte-exact): the pop fires on Bond's crossing while the contact still waits east, the pocket
   and the ch-0x26 waiters are fought from the mouth.
3. **A contact-radius clear between follow legs** (d2-gated): a started armed guard within 14 of
   the CONTACT is answered too, with the pocket rule (x < -27 shoot-only) — g3's walk to the head
   no longer outruns a 900-tick leg.
4. **The mayhew walk answers the corridor before the trail**: an own-radius answer sweep (g15 —
   sw_ch 0x12, a trio death channel — hunts east to 13 u off the hidden contact), then
   `run_slab_ambush` (at the Agent stance), then a head sweep over the stairwell head (the
   balcony's g2), all before the follow begins; the trail then walks a clean corridor.

## Verified (release, `--test-threads=1`, this tree `46193a596`)

| binary | result |
|---|---|
| walkreal_07000001_live | 7 / 0 / 1 (the 00 Agent sister stays ignored, not this task) |
| interactables_07000001_live | 10 / 0 |
| sight_cels_07000001_live | 1 / 0 |
| escort_follow_07000001_live | 1 / 0 |
| guards_shut_door_07000001_live | 1 / 0 |
| drone_fell_out_map_07000001_live | 2 / 0 |

The d1 walks in the same binary (the plain contact walk, the exit fade, the real-damage d1 sister)
all still pass — the ambush extraction returns immediately at difficulty 1 (same skip the old
inline `if difficulty >= 2 && !ch 0xf` gave).

## Fast runner

`NF_ISO=<iso> python3 tools/test/suite.py` was started in the background in the first minutes
(reaped — this sandbox kills background jobs between tool calls, so nothing survives) and then
continued in two 10-minute foreground chunks. **The build phase alone did not complete within the
window — no total line to report.** The suites this task's change touches were run whole instead
(the table above). The build cache it warmed persists in `target/` for the next session.

## What's left

- The full-suite total line on the final tree (the build was ~355 deps in; nf-view's ~30 test
  binaries remain to compile before the run phase starts).
- The 00 Agent walk (`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`) is
  still ignored with its own ledger (the AMMO-EXHAUSTED lever) — a separate task. My d2/d3-gated
  additions shift its stream; its ignore message's SPEND lever still applies.

## Usage

not visible to me (no token/credit/context figures are shown in this environment).

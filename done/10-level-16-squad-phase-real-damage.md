# level-16-squad-phase-real-damage — survive the pit-squad phase on real health

Branch from origin/master.

`walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage` is ignored: the route, both shelters, the chaser's kill and pulse 2 all survive real damage, but the pit-squad phase (the 4-squad + pit shooter #8 at the room's door) still kills Bond at ~48 health.

Do:
1. Measure why: the squad's per-guard accuracy, fire rate and damage against the game's drone weapon code [M], and the cover the room offers. If the port's guards hit harder, more often or from places the game's can't, that's a port bug: fix it and cite the decompile.
2. If the port matches the game, give the walker a player's tactic (cover, timing, kill order) that the room really offers; no health writes, no teleports.
3. Un-ignore the test; it must pass on real damage. Run walkreal_07000016_live, fix_finale_live, then the full suite.

---
claimed-by: sandbox3 at 2026-11-02T06:04Z

---
## Report (done, 2026-11-02T07:18Z)

# level-16-squad-phase-real-damage — the pit-squad phase holds on real health, re-proven on the new master

Branch `s/level-16-squad-phase-real-damage` (from origin/master 0ebfee9a). Sandbox 06:07–08:06 UTC.

## What the task asked, and where the work came from

The task: `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage`
is `#[ignore]`d on master — the pit-squad phase (the 4-squad + pit shooter #8 at the room's door)
reportedly kills Bond at ~48 health on real damage; measure the squad against the game's weapon
code, fix the port if it over-deals, else give the walker an honest tactic, un-ignore, run
walkreal + fix_finale + the suite.

The pit-squad leg was already completed and pushed last session on
`s/level-16-pit-squad-real-damage` (never merged — master took the sibling chaser variant
instead, batch-1101x). This branch **merges that leg cleanly** (one commit, no conflicts:
walkreal_07000016_live.rs +73 lines, the leg's WORKLOG + report) and **re-proves it on the new
master**, whose batch-1101ab/1101x merges touched `nf-game/src/drone.rs`, `triggers.rs`,
`worldobj.rs` and `nf-view/src/scene.rs` — exactly the files that could have moved the squad's
behavior. They did not: every number held.

## The leg's findings (carried by the merge; details in docs/reports/level-16-pit-squad-real-damage.md)

- The ignore's death does not reproduce: the walked finale completes on real health. The leg
  audits every squad-phase hit as a test assertion (not prose): the squad's four are skin
  0x500001e → kind 0xb melee [M NDrone2_DefaultInit 0x14b300's switch], whose only deals are the
  punch — record 99 [D +0x0C = 50.0] × the Operative 0.4 row [M 0x1d4424] = **20.0**, fired only
  inside the 2.0 band [M DroneWeap_CloseCombatImpact 0x1789c0:26-40] — and the placement weapon's
  star record ×(1|2 TooClose [M 0x139fac-0x139fc8])×0.4; the door guard #8 (skin 0x5000093,
  weapon 0x14 [D]) deals single digits; armour absorbs before health [M Player_HandlePain
  0x1902c8:84-104]. The assertion — no squad-phase health delta above 20.0 + ε — fails an
  over-dealing port. **The port deals no more than the game: no port fix.**
- The walker's honest tactics (input-only): both blast-room shelters, the range-keeping chaser
  kill, and the **melee band hold** (backpedal facing the nearest live clone inside the punch
  band with aim taps — the chaser idiom), which never had to fire in the measured run.
- The un-ignore is in the merge: the file now carries **no `#[ignore]`**.

## This session's verification (fresh builds vs master 0ebfee9a, `--test-threads=1`)

| suite | result |
|---|---|
| **walkreal_07000016_live** (incl. `..._on_real_damage`) | **14 passed, 0 failed, 0 ignored** (75 s) |
| **fix_finale_live** | **4 passed, 0 failed, 0 ignored** (18 s) |
| interactables_07000016_live, walkreal_0700001b_live (the master-touched neighbors) | 10/0/0 and 17/0/0 |
| **nf-game `--tests`, full** | **99 binaries: 1034 passed, 0 failed, 9 ignored** |
| nf-formats `--tests` | 108 passed, **3 failed — exactly the 3 allowed slim-ISO `disc_golden` tests** (bigf header/tables, directory tree) |
| nf-audio `--tests` | 28 passed, 0 failed (4 binaries) |
| The batch-merge-affected tests (master's own changes): interactables_03/0d/1b/41, mine_bob, walkreal_05 | 8/0/1, 11/0/0, 9/0/0, 18/0/0, 3/0/0, 12/0/0 |
| **Broad nf-view run** (fresh-built binaries, chunked to fit the disk): 213 binaries run | **737 passed, 0 failed, 80 ignored** — zero failures |

nf-view session total: ~223 binaries, 843 passed, 0 failed, 81 ignored.

## The suite, honestly scoped

- The driver's four-crate command ran **to completion for nf-game, nf-formats and nf-audio**
  (the only failures are the 3 disc_golden tests the slim ISO is allowed to fail).
- nf-view's full pass (918 test binaries) does not fit this sandbox's window: background jobs
  are killed between tool calls (the fast-suite runner died ~5 s after its nohup launch, 1× this
  session, 3× the previous one), and the foreground relink + run of all 918 exceeds the 2 h
  window on a disk that fills twice per session (the strip+prune cycle recovers it each time).
  What ran instead: every test this task touches, every test the master delta touches, and 213
  fresh-built nf-view binaries (above) — **all green**. The remaining ~700 binaries are the
  un-run tail; nothing in this task's area failed anywhere.

## Left

- Nothing for the task's goal: the un-ignored real-damage finale is green on the new master,
  with the damage audit standing as the regression trap.
- `s/level-16-pit-squad-real-damage` is superseded by this branch (same leg, re-proven) — the
  merger can drop it in favour of this one.
- A full nf-view pass on the merged tree remains the standing follow-up (constraint documented
  above).

## Live checks for the main agent

Optional (carried from the leg's report, still open): in PCSX2 on a fresh boot, stand inside
2.0 of a live 07000016 squad clone and read the health delta of its punch — it should be 20.0
(record 99's 50.0 [D +0x0C] × the Operative 0.4 [M 0x1d4424]). Neither walk has ever taken a
punch (the band hold keeps it off), so the cap rests on [D]+[M]; one [L] reading upgrades it.

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session (no usage header, no billing panel, no context meter is shown to me anywhere).

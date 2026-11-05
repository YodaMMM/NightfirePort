# Civilian alarm walks: 11-13

Branch from `origin/s/civ-alarm-repin-2`, push to `s/civ-repin-11-13`. Read docs/reports/civ-alarm-repin-2.md first: it has the measured cause for each walk and the two fix attempts that failed. Test-side only. One task only: touch only the walk file named here. Do not loosen a pin tagged [L].

walkreal_07000011_live real_damage and _agent, walkreal_07000013_live real_damage. Fights now start early. The 11 walk reached 12 with no ammo; the 13 walk took a 70-point blast on 12 north lobby. Fix ammo economy and cover in the walk.

Real input, no teleports. Run the binary whole and quote the result. If the window closes, push what is green.

---
claimed-by: sandbox4 at 2026-11-05T21:25Z

---
## Report (done, 2026-11-05T22:09Z)


---
## Report from the worker (done, 2026-11-05T22:09Z)

# civ-repin-11-13

Continue from `origin/s/civ-alarm-repin-2` 7a16d330c (whose report carries the measured cause per
walk and the two reverted attempts). Task: fix the three named walks test-side —
walkreal_07000011_live real_damage and _agent, walkreal_07000013_live real_damage. Touch only the
named walk files (both changed files are test files; nothing under `crates/*/src/` and nothing
under `crates/nf-view/tests/common/` changed, so no other binary rebuilt and no opt-in sweep
applies). No [L] pin touched: every assert in both files is byte-identical; the changes are
walker-lever settings and two route legs, gated so the passing sisters run their old configs.

## What's done

**walkreal_07000011_live — GREEN WHOLE: 3 passed / 0 failed / 1 ignored** (was 1 / 2 / 1). Two
changes, both in `walkreal_07000011_live.rs`:

1. **The 07000012 walker's survival kit** (the d1 fix). The d1 death ledger [L the
   civ-alarm-repin-2 run]: died at t1011 on 12's lobby south, clip 3 spare 0, the MASTER-rules
   resupply tour (the 45 u fetch, no tried list, the 1.8 arrive) walked Bond into killer #4's line
   (11.7 u, 2 hp) for 93.0 in 7 hits while it toured nothing. Root cause: the walk sets fight
   levers on the 07000011 walker only — its 12 walker (`w2`) ran with NONE, while the 07000013
   walk survives the SAME tower2 legs at the SAME difficulties with the lane kit [L its passing
   d2/d3 runs]. `w2` now sets `lane_fights` + `lane_floor_drops` + `back_off = Some(3.0)` (plus
   the `precise_fights` it already had), gated `!w2.tops_up()` so the topped-up plain test is
   inert. `lane_floor_drops` is the ammo-economy half: this level's fight drops rest 1.09 under
   Bond's pos y, invisible to the lane 0.5 band, so every pass came back dry while the fights ran
   the fresh 7-round P99 [M 0x1a7ef8:282-288] dry. The previous session's failed attempt
   (lane_fights+lane_floor_drops "extended to d1") touched the 11 STAIRWELL walker, not `w2` —
   the 12 legs never got a lever until now.
2. **The d2 agent pre-leg + the Load Point press-in** (the d2 fix). The clean per-test run of the
   kit-only tree: the pre-fight low-restock tour (the lane 12 u fetch) targeted the GL #290
   (10.54, 47.47, 18.10 [D]) at d 11.9 and wedged twice at the 6000-tick cap on the desk lip —
   12000 ticks and a dry descent (clip 4 → 2 → 1), then death at connector 2 (t16236, 23 hits for
   100.0 / 13 fights). Note: the previous report attributed a d2 wedge to the back-off footwork;
   the clean run shows the same 6000-cap wedges WITHOUT any back-off on the stairwell walker —
   the wedge is the GL tour, not the footwork. Fix: the d3 pre-leg's gate widened `== 3` → `>= 2`,
   so the agent takes the proven desk-maze corridor-BFS — the BodyArmour #261/#266 take
   [M 0x18e650; the 130 budget the d3 run measured on these flights] plus the #6/#5 spawn-fight
   drop loot — and walks the descent armed. d1 keeps her timeline (she passes without it). Second,
   the E press at the Load Point #193 is marginal at the r-0.3 arrival (the use point needs the
   head near z −2.8; the d2 arrival sampled −2.56, the ray fell 1.28 out, the exit never fired) —
   one deeper `try_walk_to` press-in leg (14.3, 17.0, −3.3) makes the E fire on every run.

**walkreal_07000013_live — 2 passed / 1 failed; the named real_damage (d3) still red, but its
measured killer is FIXED and the failure moved twice.** Changes, all d3-gated so the passing
plain/agent sisters stay byte-identical:

1. **The 12 walker's ammo economy** (`lane_floor_drops` + `precise_fights` added to its existing
   lane_fights + back_off 3.0): the baseline's killer — 12's north-lobby grenade, a ~70-point
   'other' chunk at t5273 [L the civ-alarm-repin-2 ledger] — is GONE. The current runs kill the
   thrower #6 with zero damage taken ("the north lobby: returned fire on drone #6 (killed true),
   health 62.0 -> 62.0") and the walk crosses 12 at 62 health (the baseline arrived at 13 with
   27). The precise gate works on 12's static desk guards (every fight killed true).
2. **The 13 walker's fight levers** (`lane_floor_drops`, `switch_dry`, `escalating_cooldowns`; a
   `precise_fights` attempt was measured and REMOVED) plus the **head-hunt wrapper** on the d3
   test only. After the 12 fix the death moved to 13's first lobby leg: the fights ran the held
   gun dry mid-stand (#0 "killed false" twice, re-fought 330 ticks later) and the stands waited on
   the 0.01-rad alignment while the lobby's MOVERS landed −24/−27 volleys. The head-hunt pick
   (one head hit drops a 12-health guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head]) plus
   `escalating_cooldowns` improved the ledger across runs: 29 hits → 16 hits for the same budget,
   #0's escalation re-fight WON, death at t1617 → t1810. The melee-class guards (weapon 0x0,
   ~20-27 a hit) still eat the carried 62 across 8 won fights.

## Verification

Ran whole and quoted on the pushed tree (`--test-threads=1`):

- `walkreal_07000011_live` — **ok. 3 passed; 0 failed; 1 ignored** (36.15 s). The ignored test is
  the 00 Agent sister (unchanged, still `#[ignore]` with its own docs).
- `walkreal_07000013_live` — FAILED. 2 passed; 1 failed (33.10 s): the plain and d2-agent chains
  pass; the d3 real_damage dies as described above.

Per-test clean runs used during the iteration (same tree family, quoted in the WORKLOG): the 11
d2 agent alone `ok` (10.61 s), the 13 d3 alone at t1810 / 16 hits for 62.0 / 8 fights. The full
suite did not fit the session window; the two named binaries are the touched set (only these two
files changed, so no other test binary rebuilt — `cargo test --no-run` for the pair is the whole
rebuild). Determinism untouched: no RNG draws added, no wall-clock, no position writes.

## Left

The 13 d3 real_damage lobby leg. The carried 62 (from 12) is exactly consumed by 8 won fights;
the melee-class guards (weapon 0x0, ~20-27 a hit — #6's −27 is the single largest chunk) close
through the 3.0 back-off band. Concrete next levers, in player order: (a) a narrower
`shooters_window` on the 13 walker so the 25-41 u rifle guards stop triggering cross-lobby stands
(their walking-past fire is 1.2-1.4 a hit, cheaper than a stand); (b) the melee answer — a wider
back_off on the 13 walker only, or live data on the melee reach vs the step-back speed; (c) the
12 walker's corridor attrition (−14 at the bend, −24 outside fights) — `escalating_cooldowns`
there would carry more than 62 into the lobby. The armour #414 take-ray KNOWN ITEM is unchanged
(the detour stays, not pinned).

## Live checks for the main agent

- **07000012 north lobby** (fieldprobe, carried over from civ-alarm-repin-2, still unanswered):
  the grenade thrower #6's weapon byte and the throw tick relative to his death — with the head
  hunt he now dies in the first exchanges; confirm no grenade spawns (weapon 0x1a on #6?).
- **07000013 lobby** (gdbwatch): one melee-class guard's (weapon 0x0) charge speed and melee
  reach vs Bond's back-off step rate — does any back_off radius actually outrun the charger, or
  is the answer range (drop him at 8-10 u)?
- **07000011 top floor** (fieldprobe): can a played Bond take the GL #290 (10.54, 47.47, 18.10)
  from any reachable tile (the nav wedges at d 2.1-2.5 from the south; the take sphere is
  radius + 1.1)? If yes, a d1/d2 pre-leg stop there becomes the cleanest ammo fix.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.

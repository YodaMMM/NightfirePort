# Level 0a walks: real damage

Branch from `origin/s/civ-combined`, push to `s/civ-repin-0a-2`. Read docs/reports/civ-repin-0a.md. Test-side only, touch only walkreal_0700000a_live.rs. Do not loosen a pin tagged [L].

The plain walk is green. The real_damage, _agent and _00agent walks die: _00agent at tick 12584 at (-12.73, 1.10, -16.90). Read the death ledgers and fix each route or fight. Bond must stay unseen by the team-3 civilians.

Run the binary whole and quote the result.

---
claimed-by: sandbox3 at 2026-11-05T22:23Z

---
## Report (done, 2026-11-05T22:45Z)

# civ-repin-0a-2

Branch cut from `origin/s/civ-combined` 4e9fadde4 (which already carries civ-repin-0a's ring
re-route + stealth gate and the merged civ-repin-09 takedown rebuild). Task: the three
real_damage sisters of `walkreal_0700000a_live` — test-side, `walkreal_0700000a_live.rs` only,
no [L] pin loosened. Result: **12 passed / 1 failed / 2 ignored whole** (the binary, this
branch) — `real_damage` and `_agent` are GREEN; `_00agent` remains red at a measured d3
exposure, resume recipe below.

## The pocket "world damage" root-caused: it is GT #639's minigun

The civ-repin-0a report read the ledger's `the world (live wire / blast / hurt volume)` bucket
as a world damage volume at the pocket. Measured this session (an env-gated probe at all four
`player_hits` push sites in `nf-game`, applied locally and reverted — the branch ships test
code only; the probe's findings are restated here):

- The hits are **`site-D(bullet)` rows: owner `usize::MAX`, weapon record 96, 2.4000015 damage
  at d1** [L this run] — record 96 = `0x2c55d0` = **`gt::TURRET_WEAPON_RECORD`**, and the
  owner is the GT Fire arm's not-a-drone mark ("the turret owner's obj+0xff is not the player's
  2 and the barrel obj is not a class-3 drone" [M drone.rs's GT Fire arm, the
  `spawn`-site comment]). The bullets come from **GT turret #639**.
- The turret list at the wedge: `#639 state 4 (COMBAT) at (−21.98, 2.28, −11.31)`, every other
  0a turret state 0 [L this run]. The 2.4000015 = the record's 5.0 damage × the d1 pain factor
  0.48; the _00agent ledger's 4.0 = 5.0 × the d3 factor 0.8 [M pain_generic's difficulty
  factors; L both ledgers].
- The turret is NOT positional damage — it is a TRACK-AND-FIRE gun whose LOS (`world.
  line_of_sight(turret_pos, cand.pos)` [M 0x17f9c8:271-278's port]) only opens fire on the
  keypad approach's last ~2 u: the first hit landed ~120 ticks after Bond reached the wedge,
  and the whole west lane (x −26..−14) plus the pocket's west half took ZERO hits on every
  run [L].
- The wedge itself: `walk_leg_stealth`'s straight ~4 u slices wedge on the pocket furniture
  ~1 u short of the far target and the 200-seg loop grinds IN PLACE (each re-aimed slice's
  `walk_to` returns inside its 1.1 arrive radius without progress) — the plain walk ground
  199 segs (~1200 ticks) there and escaped only via the cap; the tops-up suppressed the turret
  fire that whole time, which is why the plain walk stayed green. The real-damage sisters died
  in the grind (42 hits × 2.4 = 100.8).

## The fixes (all in walkreal_0700000a_live.rs)

1. **The pocket tail is capped** (`walk_leg_stealth_cap`, cap 4, the original stand line
   (−11.9,−17.5) kept): the seg walk CONVERGES to (−12.73,−16.90) by seg 1-2 on every run —
   the exact spot the plain walk PROVED the keypad E arms from (dxz 2.11, the use sphere
   1.0 + 1 forward [M 0x1a5170:101-108]) — and the cap stops the grind before #639's fire
   matures. (A south-east hook leg was tried and removed: its own line wedges at a different
   spot — the wedge position is target-dependent through the walker's stuck-recovery rolls.)
2. **The keypad tail runs from cover**: the armed keypad is a MENU state — the digits
   (`C_KEYPAD_Handler 0x216810 msg 0x4b`) and the door-unlock chain (`Lock_Unlock 0x1ae3d8
   case 4 → case 1`) are position-independent — so after the E-arm Bond WALKS BACK to the
   proven-quiet (−14.5,−15.0) (`walk_leg_stealth`, the lane's own quiet floor in reverse) and
   the 4 digits + the ~120-tick door sweep run from there. An assert pins the menu surviving
   the retreat. The exposure shrinks from ~400 ticks of standing to ~100 ticks of moving.
3. **The east strip crosses inside a distance window** (`wait_distance_clear` —
   distance-only, 12.5 u, cap 3600): the world-segment LOS probe over-blocks the strip [L the
   t14717 note in the parent report], so the gate is the sight RANGE itself — no team-3 drone
   within 12.5 u of Bond or the strip's leg targets. This is what fixed the SECOND failure the
   fixes above exposed: with the sisters alive past the pocket, the plain walk's documented
   racy g17 sighting (his SW beat converges on the strip mouth) cascaded — a converging runner
   crossed the laser web and the alert OR mux wrote **ch 0x12** — the laser pin popped at the
   hatch (Bond alive, 28 hp, the LOAD_LEVEL to 0x0700000b already in reach [L that run]). The
   hold crosses the strip during g17's NE return (his spawn (24.6,−6.8) is 15.6 u from the
   strip mouth — out of his 12.0 sight) instead of during his SW beat.

## Verification, quoted

- The whole binary (`--test-threads=1`, this branch): **12 passed / 1 failed / 2 ignored
  (50.40 s)** — the branch point's baseline was 9 passed / 4 failed / 1 ignored.
- `walk_office_floor_qworm_keyboard_hatch_real_damage`: **PASS** (6.57 s single) — the full
  mission on real health: the pocket, the keypad E + the cover-typed code, the door, the ring,
  the strip window, the fence, the hatch E. Bond takes the turret's fire only in the designed
  exposure windows and survives it.
- `..._real_damage_agent`: **PASS** (in the whole run).
- `..._real_damage_00agent`: **FAILED** — died t12619 at (−10.45, 1.10, −18.65) (the doorway
  crossing after the re-approach), the d3 4.0-per-hit turret fire over the ~150-tick
  re-approach + the E exposure ≈ 109 hp against 100 [L the ledger].
- The 9 non-walk tests (laser-band phases, doorway/wedge, keyboard, outbound-only) all pass
  (in the whole run).

## Left (the _00agent resume)

The d3 budget needs ~20-30 hp less exposure on the re-approach, in the order to try:

1. **Re-approach south of the console**: the RING2 re-approach rides (−14.5,−15.0) →
   (−11.0,−18.7) straight through the arc's deep zone. The pocket's SOUTH floor is the
   Q-Worm area's own proven-quiet lane (the outbound walk's nav end (−14.88,−29.15) [D]) — a
   hook (−13.5,−19.5) → (−11.0,−18.7) may ride #639's LOS edge. Measure first (one probe run
   with the per-hit log).
2. **Pre-aim the E**: the walk arrives at the stand facing the leg direction; snapping the aim
   during the LAST capped seg (a turn-only tail) cuts the E loop's ~40 ticks.
3. **The d3-gated lever** (the file's own pattern): only at difficulty 3, crouch-walk the
   re-approach (if the port's crouch changes the turret's candidate box — the LOS reads
   `cand.pos` [M], so probably not) — or accept and heal: the level's armour/health pickups
   east of the door are reachable AFTER the crossing; the death is BEFORE it, so this only
   helps if a pickup sits inside the pocket — the live-check ask below covers it.
4. If the doorway itself is the killer: cross the moment the sweep ends (the wait loop's break
   is already the sweep's state 4 — the crossing legs run immediately after; the exposure is
   the re-approach, not the wait).

## Live checks for the main agent

- **0700000a (fieldprobe)**: GT #639 (placement −21.98, 2.28, −11.31, the pocket's west mount)
  — its facing arc and whether the GAME's GT_Track LOS ray from its muzzle clears the pocket
  floor at (−12.7,−16.9) but not at (−13.5,−19.5) (the resume lever 1's premise). Also: does
  any health/armour pickup sit in the pocket west of the door (lever 3)?
- **0700000a (gdbwatch)**: which beam row's touch wrote ch 0x12 in the pre-hold run (the
  runner's crossing point) — confirms the runner path the strip window now avoids.
- **0700000a (gdbwatch)**: #639's ARM trigger — what moved it from state 0 to COMBAT at
  ~t12400 (a script event, a proximity gate, or the track's first LOS) — if it is a channel,
  the disarm-channel question from the parent report (ch 0x37 has no writer on 0a [D]) gets a
  second look.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.

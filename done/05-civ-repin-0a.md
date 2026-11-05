# Civilian alarm walks: 0a

Branch from `origin/s/civ-alarm-repin-2`, push to `s/civ-repin-0a`. Read docs/reports/civ-alarm-repin-2.md first: it has the measured cause for each walk and the two fix attempts that failed. Test-side only. One task only: touch only the walk file named here. Do not loosen a pin tagged [L].

walkreal_0700000a_live: 4 red, "the mission failed at ring: the west corridor jink 3" with full health, so the alarm chain failed it. Same stealth rule as 09: drop the office guards before they ack.

Real input, no teleports. Run the binary whole and quote the result. If the window closes, push what is green.

---
claimed-by: sandbox1 at 2026-11-05T21:22Z

---
## Report (done, 2026-11-05T22:15Z)


---
## Report from the worker (done, 2026-11-05T22:15Z)

# civ-repin-0a

Branch cut from `origin/s/civ-alarm-repin-2` 7a16d330c. Task: fix `walkreal_0700000a_live` (4 red:
the office-floor walk + the three real_damage sisters, all failing at "the west corridor jink 3"
with full health — an alarm-chain fail, not damage). Test-side only; only the 0A walk file touched;
no [L] pin loosened.

## The measured cause ([L] 2026-11-05, `NF_ALARM_TRACE=1` on `walk_office_floor_qworm_keyboard_hatch --exact`)

The ring's central ai-lane crossing is the leak. The office floor's ackers are TEAM-3 CIVILIANS,
not security guards:

- **g7** (skin 0x500002a, sight 12, 180° cone, patrol lane = the route's own ai lane, waypoints
  (−18.6, 5.4) / (−23.8, 7.2) / (−25.6, 7.5) / (−20.6, 5.3) with an east excursion to (−13.1, 2.3),
  cycle ≈ 3685 ticks) challenged Bond at the node-17 leg: `t11812 g7 0x14 → 0x3e d 3.3 seen true
  los true`.
- **The shout chain is instant**: one tick later `t11813 g0 0x14 → 0x63` (GoToGoalPosition — the
  alarm panel run) from (−29.1, 19.6), 17.9 u away.
- **g0 ran 12.6 u to the panel at (−21, 10)** and pressed: `t12163 g0 0x63 → 0x8a`; the press
  timer landed the fail `0x4000032` at t12400 (= the press + 237 ticks [M
  `NDrone2_DSTATE_PressAlarm` 0x168ea8's timer C]) → the fail-menu auto-restart stopped every
  later leg → the jink-3 assert.
- **The civilians cannot be dropped**: props[2] = 0x45B92400 has bits 0x53 AND 0x54 set — a
  player kill is fail reason 3 (team 3 + 0x53 + player killer → 3 [M `DroneFunc_OnInitDeath`
  0x145bd0's verdict; docs/fields/hitdata-spawner-ally.md:203-206]) — a started press cannot be
  undone either. The only player move is to be seen by nobody.

## The fix (this branch, test-side, walkreal_0700000a_live.rs only)

1. **The ai-lane crossing is REMOVED.** The ring's middle section (the old node-17..34 legs, the
   west-door jinks, the south-west jinks) is replaced by the **south-west lane splice**: the
   outbound walk's own proven-quiet lane (the "to the Q-Worm PC" nav nodes and the walk-back's
   return nav both ran it unseen on the same branch [L 2026-11-05]) — the arrived node positions
   hard-coded as the lane legs: (−22.12, 8.60) → (−22.18, 6.50) → (−23.04, 2.48) → (−25.99, 0.48)
   → (−26.46, −3.93) → (−26.06, −7.36) → (−22.21, −11.97) → (−16.88, −15.65) → (−14.5, −15.0)
   (the old pillar-crossing pocket entry), then the keypad's wall as before.
2. **A stealth sight-gate** (`watcher_sees` / `sight_watchers` / `wait_sight_clear` /
   `walk_leg_stealth`, all local to the walk file — no shared-helper change): before each ~4-unit
   segment the gate holds until no live non-team-0 drone's sight triple reaches Bond's spot or the
   segment's end — distance ≤ sight_range, |yaw error| ≤ half_fov [M 0x175ef0 family], LOS probed
   at six body heights (0.4..2.0; any clear ray = exposed). The gate covers the splice legs, the
   keypad's wall leg, and the fence-wait's break condition was gated too (later reverted — see
   the residual). It fired zero times on the passing run — the lane is genuinely quiet.
3. The keypad-wall leg keeps the stealth gate (g10's patrol reaches (−15.0, −22.1), 5.5 u from the
   stand).

## Verification (the binary run whole, `--test-threads=1`, this branch)

- **walkreal_0700000a_live — 10 passed / 3 failed / 2 ignored (46.63 s)** — the baseline on this
  branch's parent was 9 passed / 4 failed / 1 ignored (47.84 s).
- **walk_office_floor_qworm_keyboard_hatch — PASS** (6.54 s single, 15824 ticks, stuck 0, falls 0):
  the whole mission on real input — the arrival plate, the walked Q-Worm install, the keyboard,
  the re-routed ring, the keypad through the real E + the snapshot code [L 0x2beff8], the door
  #724 chain, the laser fence, the hatch E. Zero civilian sightings on the west/ring half; the
  laser pin holds (no mask-8 beam out, gun #640 never armed); every channel pin unchanged.
- The 9 non-walk tests of the binary (laser-band phases, doorway/wedge, keyboard, outbound-only)
  all still pass.

## The 3 remaining reds (measured, not fixed this window)

- **real_damage / _agent / _00agent die at the POCKET HAZARD**, not on the ring: the ledger —
  died t12703 at (−12.73, −16.90), −2.4 (−4.0 on 00agent) every ~7 ticks from "the world (live
  wire / blast / hurt volume)", 42 hits for 100.0, 0 return-fire fights. The pocket's east
  approach band (≈ (−12.7, −16.9) to the keypad stand) is a world-damage source the tops-up walk
  crosses standing (damage suppressed). The stealth gate's waits are unsound inside the band (the
  ledger shows ~300 stationary gate-held ticks burning the health). The sisters' failure MOVED:
  they now die at the keypad approach instead of the ring — the whole ring + south-west lane is
  clean under real damage.
- **The east-strip race (documented residual on the green walk)**: on the passing run g17 (spawn
  (24.6, −6.8), east-side patrol civilian) saw Bond at the south-gap leg (t14583, d 8.6) and g3
  (spawn (9.8, 21.3), the hatch-antechamber civilian) at the north corridor (t15305, d 4.9); both
  ran 0x3e → 0x63 (panel runs) and the walk finished BEFORE their presses landed (~1-2 s margins)
  — a racy pass. A gated variant of those legs was tried and reverted: the gate's world-segment
  probe reads the g17 ray BLOCKED at every probe height while the game's LOS reads true at the
  same geometry (the t14717 sighting repeated identically with the gate live [L]), so the gate
  cannot see that exposure — the probe's semantics vs the game's sight ray need the real
  ray heights/sets before the east legs can be gated soundly.

## Live checks for the main agent

- **0700000a (fieldprobe/gdbwatch)**: what is the world-damage volume at the pocket (≈
  (−12.7, −16.9) → the keypad wall (−11.9, −17.5))? Stand a played Bond at (−12.7, −16.9) and
  screenshot/record the damage source (a live-wire actor? a laser row? a HurtPoly?). The
  sisters' dodge leg cannot be designed blind — the volume's shape decides whether the approach
  goes south of it or the stand moves east.
- **0700000a (gdbwatch)**: the drone sight LOS ray's exact height/set ([M 0x175ef0 family]):
  during a played east-strip crossing, record the ray args the game passes when g17 (east
  patrol, spawn (24.6, −6.8)) acquires Bond — the port's `world.segment` reads the same line
  blocked at heights 1.4-3.2 while the game's `los` reads true (measured t14717 [L]). The true
  heights unblock the stealth gate for the east legs.
- **0700000a (gdbwatch)**: g17's full patrol cycle (spawn (24.6, −6.8), observed SW leg to
  (20.6, −12.8) at ~0.015 u/tick) — the SW end, the turn, and the NE-return duration. The strip
  crossing window is his NE return (his back to the west lane); the cycle decides the hold spot.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.


## ntfy failed

All three fallbacks failed (curl exit 28 connection timed out; python urllib [Errno 101] Network is unreachable; wget killed at 35s, exit 124). The report is on the branch s/civ-repin-0a at 7e506273d; the driver can read it there.

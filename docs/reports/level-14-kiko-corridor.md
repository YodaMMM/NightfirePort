# level-14-kiko-corridor — report

Branch `s/level-14-kiko-corridor` = `origin/s/level-14-fusebox-zap` 23b0e620f (origin/master
already an ancestor — no further master commits existed at branch time). Task: 07000014's
Operative real-damage walk gets through the Kiko corridor. All landed changes are in
`crates/nf-view/tests/walkreal_07000014_live.rs` and docs; **no `crates/*/src/` code landed**
(the one diagnostic patch used to trace the channel-60 writer in `triggers.rs` was reverted
before commit; `git diff` on `crates/*/src/` is empty), so the every-walkreal rule was not
triggered — walkreal and interactables were re-run whole anyway.

## Done — the door's lock read in the game (the port matches it), the corridor leg routed on foot

**The door's lock, the full chain [M]+[D].** The wedge's nearby door is **#111/#112
"Door_Slide_14A/14B"** at (−42.7126, 1.2030, 29.8977) [D the placements census]. What locks
it: its lock channel = placement param 3 = **60 (0x3C)** [D #111/#112] — `Door_IsLocked`
0x136810:10-13 reads "ch_lock ≠ 0 and the channel off" [M], and `Door_Activate` 0x135be0:53,58
refuses activation while closed/closing [M]. What unlocks it and when: **channel 60's rising
edge** — `Door_Update` 0x1363c8:88-93 tests `prev == 0 && now ≠ 0` against
`switch_channels_prev`, unlocks for good (one-shot: the field is cleared, :106) and
auto-opens the door [M :97-107]. The writer of ch 60: **keypad lock #776 "Lock -
KeypadLocked"** (kind 0x2e, (−41.4377, 1.3037, 29.7474) — 1.3 u from the door), out channel =
param 5 = 60 with the Operative create-unlock tag param 6 = 1 [D]: `Lock_Create`
0x1ae1f8:67-69 unlocks it at create on difficulty 1 (Operative) [M], and `Lock_Update` case 1
0x1ae3d8 holds `switch_channels[60] = 1` EVERY tick while the lock sits unlocked [M] — so on
the Operative walk the door is unlocked and auto-opened from the first tick of the level. No
other disc writer of ch 60 exists (census: no trigger/sensor/switch/drone-death/NIS key —
checked `plc14` placements and both level-14 NIS pack dumps). The port reproduces this
exactly: `Drones::attach_level_world` holds each create-unlocked lock's out channel at attach
(the level-latched form of the game's every-tick write), the door unlocks on the rising edge
and auto-opens. **[L the NF_LEVELLOG run]: `[levellog] frame 0 channel 0x3c = 1`**, and at the
wedge both leaves read state 0x4 pos01 1.00 ch_lock 0 — open and unlocked. The door is NOT
the wedge blocker (it stands open BEHIND Bond at the wedge). The port's lock semantics match
the game end to end — **nothing to fix**; on Agent/00 Agent the param-6 gate keeps the lock
closed and the A* must route around the leaf. This chain is now pinned in the walk body
(difficulty == 1, both walks): ch 60 on at load and door #111 not locked.

**The corridor leg, routed.** The wedge's real cause: the fights' backpedal parks Bond in the
machine-room floor pocket (nav cell 44) and EVERY straight steer out of the pocket clips the
machinery 0.11–0.17 u in (the NW corner at (−44.42,31.06), the west machine face at
(−44.47,35.75) [L the run 2-5 STUCK dumps]); the nav web is blind to the machinery INSIDE
cell 44, so `walk_nav_to` hops the blocked diagonal. The through-route is the catwalk OVER
the machines — the leg's own resupply A* walked it [L run1]: (−49.51,1.04,36.22) floor →
(−47.96,2.23,33.20) catwalk → (−41.29,1.53,36.48) floor. The walk now takes, all
`!tops_up`-gated (the plain walk byte-identical): `clear_hostiles(14 u)` first (a player
clears the room from the pocket's open floor — run1's fights killed #19/#18/#23 with Bond
holding 79.6–85, while the quiet-route runs died at the mouth to the accumulated guard #20's
free point-blank bursts [L runs 6/7]), fight-quiet pocket legs (`dmg.return_fire` off — no
backpedal re-parks Bond mid-route), the catwalk route reversed with fire restored on the
catwalk, lenient legs (the mid-leg resupply resets the walker's lenient flag — the run2/3
panic path) and a strict waypoint leg. **Result [L run9]: the Kiko-corridor waypoint and the
corridor mouth are PASSED** — the old wedge (the 6000-tick cap at (−44.35,1.01,30.95)) is
gone; the walk now dies FURTHER ON, at tick 43690 at (−58.92,0.73,37.67), to the west-hunting
guards #19/#23/#24 (skin 0x5000087, weapon 0x14, d 1.7–2.3 — the FirstAttack hunt blocks [M
0x148010:100-131]) with the health budget drained by the fight trades. Real input only
throughout: no health/armour/ammo writes, no teleports beyond the walk's documented
stand-ins; the d1 test stays `#[ignore]`d with the full ledger and the next levers in its
ignore message.

## What ran

- `walkreal_07000014_live` whole: **8 passed, 0 failed, 2 ignored** (the plain walk — which
  also runs the new difficulty-1 door pin — and the tripbomb route green; the d1 walk and
  the 00 Agent walk carry their ledgers).
- `interactables_07000014_live` whole: **10 passed, 0 failed**.
- The d1 walk's measured runs (this task): run1 (baseline repro with the channel trace), run2
  (nav leg — the A* hops the blocked diagonal), run3/4 (via probes — the pocket's west seal
  mapped), run5 (catwalk route, fights re-park), run6/7 (fight-quiet — through the pocket,
  died at the mouth), run9 (pre-clear + quiet + catwalk — **through the corridor**, died at
  (−58.92,37.67)). Logs in `scratch/kiko_run*.log` (not committed).
- `nf-game --lib` not re-run: no `crates/*/src/` change landed.

## What's left

- The d1 walk's new blocker: the west-hunting guards at/behind the corridor mouth kill Bond
  at (−58.92,0.73,37.67) (tick 43690). Levers in the test's ignore message: a second
  `clear_hostiles` pass after the corridor waypoint; the fight-quiet window extended over
  `follow_nav_route`'s first legs; the resupply/loot pass on the corridor floor before
  re-engaging.
- The 00 Agent walk's ledger is unchanged (the "1.72 landing" hover — not this task).

## Live checks for the main agent

1. **The boot unlock, fresh boot** (no level-14 EE snapshot exists in `work/`, so this [L]
   was taken from the port only): load 07000014 at **Operative**, and at the first frame read
   `switch_channels[60]` @ **0x26fccc** (base 0x26fc90 + 60) — expected **1** (Lock_Update
   case 1's hold [M 0x1ae3d8]); then the door exts of placements #111/#112 (class 0x1C,
   `DynamicObjList` 0x2705a0 walk): +0x68 expected **0** (cleared after the rising edge [M
   0x1363c8:106]) and obj+0xF4 expected **4/8** (open/opening) shortly after boot. The same
   reads at **Agent** difficulty: switch_channels[60] expected **0**, the doors locked closed
   (state 2, +0x68 = 60).
2. The machine-room pocket: stand Bond at (−44.35, 1.0, 30.95) and steer north-west — the
   game's own collision should refuse the first ~0.13 u exactly as the port's wedge dumps
   show (the machinery corner), confirming cell 44's blindness is the game's own nav
   granularity, not a port parse gap.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (no usage panel is exposed to the agent).

## ntfy failed

(pending — filled at finish)

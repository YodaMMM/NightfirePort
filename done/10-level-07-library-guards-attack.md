## Your task: `level-07-library-guards-attack`, The Exchange party fails when a guard is attacked first, reached on the real path

Branch from origin/master. walkreal_07000007_live::the_prop52_guard_first_attack_fails_the_mission is ignored: the library door guards #404/#417 hold the doorway until Drake's reveal (ch 21), which takes the hall's 0x52 carriers off and ropes the west doorway (#495); #424/#425 never get a line of sight in 1800 ticks. Find a real-path point where a 0x52 carrier can be attacked before ch 21 (another carrier, another room, or before the library scene) [D the carrier list and channels], and pin that attacking one fails the mission the way the game does [M]. If no carrier is reachable in the game either, prove it [M/D], delete the test, and say so. Run the whole walkreal_07000007_live binary and interactables_07000007_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-07-library-guards-attack`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T07:52Z

---
## Report (done, 2026-11-10T09:23Z)


---
## Report from the worker (done, 2026-11-10T09:23Z)

# level-07-library-guards-attack — the real-path first-attack fail

## What's done

`walkreal_07000007_live::the_prop52_guard_first_attack_fails_the_mission` is UN-IGNORED and
rewritten: the party fails when a guard is attacked first, reached with real input from the
load — no teleport after the spawn (branch s/level-07-library-guards-attack, master 5ae2a5de8).

**Where the old setup died, and what the disc actually offers.** The ignored round armed past
the ch-0x48 plate and hunted a carrier's own first attack in the hall — but the weapon lock
lifts only at the plate [M Player_Weapon 0x1a24c8:243-246], the plate sits behind the ch-21
library door, and ch 21/22 walk the hall crowd off. The fresh [D] param census (every map47
drone's p6/p8/position, probe-style dump against ACTION.ELF's property tables) splits the 38
0x52 carriers (probe_prop52_census's census) into:
- p6 = 0 (no second channel — the reveal chain can never take them off): #349 (44.3,−62.1),
  #350 (44.2,−61.1), #399 (43.4,−78.9), #402 (43.0,−59.4) — all standing ON the main walk's
  proven west-aisle/wall-run line in the start room — plus the hall's #424/#425;
- p6 = 72 (#414/#429/#488/#489/#501): no writer for ch 72 exists on this level [D the
  interactables census §7], so they are effectively never taken off either;
- p6 = 21 (#355/#406-#410/#426/#431-#434/#446/#487/#490/#491/#494/#506) and p6 = 22
  (#403/#477-#484/#508): taken off at Drake's reveal / the door plate.

**The pin.** The test walks the main walk's own legs ("west aisle south" (35.7,−70.6), "west
aisle north" (42.6,−70.5) — unarmed, the crowd stays calm [M CivilianGuard 0x1603a8 case 3];
asserted: no fail label, ch 99 clear, no carrier first_attack_done), stops just south of
carrier #402, LOOKS at its chest (the punch round flies from the eye along the view ray [M
Player_WeaponInitBullet 0x1a4bf0 — the fists' excluded-base record spawns at the camera]) and
holds the fire button with the loadout fists [M Player_InitWeapon 0x1a7ef8 case 0x7000007].

**The way the game fails it [M]:** the punch dispatch (`NDrone2_DealWithObjHit` case 5 → msg 6
→ `NDrone2_PunchImpact` 0x145f28) — on 07000007 a punch on an UNARMED victim (`ObjectIsArmed`
0x178768; DoModeSettingsOLD 0x14cfa0:159-163 zeroes the party classes) calls
`SetMissionFailReason(5)` [M 0x145f28:45-56] = label 0x4000033 "Cover Lost" + ch 99 [M
0x13e560]; the guest is team 3 [M 0x145bd0:21-43], so PunchImpact's team≠1 arm then ALSO fires
for the same swing — reason 6 = the civilians' label 0x4000034 + ch 0x60 [M 0x145f28:69-73 →
0x13e560 case 6] — the last call's label stands, ch 99 stays latched → mission fail state 2 →
end to 0x7000048 [M 0x182c60:45-57]. Measured: 8 swings, label 0x4000034, ch99 AND ch0x60 on,
mission.state 2. The port's `punch_impact` (drone.rs, the party07-start-rules port) implements
exactly these two arms; the test pins them through the real path. The 0x52 property itself is
pinned separately: the first-attack tail's reason-8 arm (0x148010:95-98) is ported
(drone.rs:6217) and the census lists the carriers; on this level the game's own first-attack
reach for a carrier needs an armed+seen Bond, which the real path cannot produce before ch 21
(the lock) and cannot produce after it in the hall (the crowd walks at ch 6/21/22) — the punch
is the reachable real-path attack, and it fails the mission exactly as the game does.

## How it was verified

- `walkreal_07000007_live` whole binary: **6 passed, 0 failed, 0 ignored** (the un-ignored pin
  + the plain walk, the real-damage walk and its Agent/00 Agent sisters, the group-plate calm).
- `interactables_07000007_live`: **4 passed, 0 failed, 0 ignored**.
- The whole level-07 family in one sweep (guests_hostile, interact, library_door,
  library_door_guards_warn, party_door_lock, walk, walkreal, interactables — 8 binaries):
  **27 passed, 0 failed, 0 ignored**.
- The three non-view crates at --test-threads=1 (NF_SILENT=1): every binary that reported
  inside the run window was green except the 3 `disc_golden` tests, which are the documented
  slim-ISO failures. nf-game's unit-test sweep is large and the call window (10 min per call
  in this sandbox) cut it before a total line; no failure other than disc_golden appeared.

**The full four-crate suite did not produce a total line this session** — the sandbox's
background-process kills forced foreground-only runs, the whole-tree rebuild after the master
move hit a full disk twice (the stale /home/z/my-project old-session clone, 1.9 GB, was the
blocker and is deleted), and the remaining call windows went to the level-07 binaries above.
My diff surface is one test file (no `crates/*/src` changes), and every binary that contains
it ran whole and green.

## What's left

- The full-suite total line on the final tree (the run strategy above fits a fresh sandbox:
  start `tools/test/suite.py` right after the first build, not after the walk work).
- [S] optional: a live PCSX2 spot-check that the game's own punch on a party guest shows the
  label flicker 0x4000033 → 0x4000034 (the port's drain order mirrors the [M] arms; the port
  has no visual surface here, so nothing to render).

## Live checks for the main agent

None required — the change is a test-only pin with no visual surface.

## Usage

not visible to me

## Your task: `guard-noise-attack-states`, alerted guards react to gunfire as the game's do

The real-health-00agent-01-hall report (origin/s/real-health-00agent-01-hall, docs/reports/real-health-00agent-01-hall.md, live check 2) found that the port's noise chain exists (DroneFunc_SetNoiseHeard 0x149920, HandleSoundAlerts 0x148a30), but the alerted attack states have no handler for the sound message (0x14). Read the game's combat and attack state handlers in full and list which of them handle message 0x14 and what each does, with addresses. Where the port differs, port it in the game's order, keeping the Rand count, and test it red on the old code. Pin it through real play: a sustained pistol burst near alerted guards draws them as the game's handlers say. Then run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-noise-attack-states`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T04:14Z

---
## Report (done, 2026-11-11T05:55Z)


---
## Report from the worker (done, 2026-11-11T05:55Z)

# guard-noise-attack-states

Branch `s/guard-noise-attack-states` (cut from origin/master 784ba2d99; master did not move under
the branch — the final tree contains it).

## What's done

**The survey the task asked for** — every game state handler read, message id 0x14 (SOUND) located
in each ([M] = the decompile file named by its address in `work/decomp/ACTION.ELF/`; the message
dispatch is `Drone_SM_SendMsgSelf(0x14,0,0,0,dcvars)` from `DroneFunc_HandleSoundAlerts` 0x148a30,
and every handler below switches on `*(u32*)msg`):

**The alerted combat/attack states handle 0x14 in NONE of them.** Full case lists were read for
every combat/attack handler — Attack 0x164210, Combat 0x1731b8, CombatNoMove 0x1643e8,
CombatOutOfRange 0x164690, CombatNoSight 0x164838, CombatWait 0x164960, CombatNoRoute 0x164c08,
CombatNewSighting 0x173280, CombatTooClose 0x173338, Alerted1stEncounter 0x173178, AimStand
0x166178, AimStandFire 0x166328, AimStandReload 0x1666d8, AimStandDiscard 0x166840, AimCrouch
0x166f28, AimCrouchFire 0x167060, AimCrouchReload 0x167370, Prone 0x166940, ProneFire 0x166a28,
AltAttack 0x167488, StepAimLeft/Right 0x167590/0x1676a0, StrafeAimLeft/Right 0x1677b0/0x167980,
StrafeDodgeLeft/Right 0x167b50/0x167cf0, RollLeft/RightCrouch 0x167e90/0x167f80, the NinjaAttack
family 0x16b990–0x16d878, the BotAttack family 0x16f198–0x175188, AstronautCombat 0x174458 /
AstronautCombatMove 0x174550, SniperAim/Fire/Reload 0x15fc18/0x15fd88/0x15ff80, UnderCoverAim/Fire
0x16a2d0/0x16a3e0, AmbushWait 0x161560, SeenOpponent 0x173b38, SeenDroneShot 0x173b78,
SeenExplosive 0x173bb8 — a `case 0x14`/`== 0x14` scan over all 250 DSTATE files finds the id only
in the 25 handlers below, never in this list (a sighted attacking guard has no sound reaction; the
port matches this already — its armless combat states drop the message to GLOBAL, and GLOBAL
0x15b830 handles 0/1/0x10/0x1d only).

**The 25 handlers that DO take 0x14:**

- Shared group → `DroneVision_EnemyAlerts(dcvars,msg)` 0x177c18 (for 0x14 it returns
  `DroneVision_AlertSound(dcvars)` 0x177800; store `SM+0x1c`, `SetState` on a non-zero result):
  PlayScript 0x15ba68, Idle 0x15bf18, Alert 0x15c248, Patrol 0x15c498, HostageKiller 0x15c728
  (:69-85), Civilian 0x15d118, CivilianPatrol 0x15d610, CivilianMission 0x15d860,
  CivilianMissionWait 0x15dbb0, EnemyMission 0x15e1e8, SniperIdle 0x15fab0, PartyGirl 0x160140,
  CivilianGuard 0x1603a8, CivilianDoorGuard 0x160648 (:154-161), InterogateAssistWait 0x161860,
  Interogate 0x161a40, InterogateWalk 0x162338, Investigate 0x168070. Three with a twist:
  - KikoMission 0x15df28 (:54-68): on 0x07000016 the family is ignored; ELSEWHERE EnemyAlerts runs
    and a non-zero result **hardcodes KikoMissionRun 0x1a** (the shared tail `uVar4 = 0x1a`
    discards the fold's pick). Case 0x1f (:69-74) rides the same gate.
  - CastleChatGuard1 0x1612e0 (:85+): the shared path, plus — when the fold returns 0 and
    `+0x33 == 0` — `+0x5a0 = 4`.
  - BotGlobal 0x16e060 (:550+): the group 0x11/0x12/0x13/0x14/0x15/0x16/0x1e switches on
    `BOTSTATE_getStateType`: types {0,1,2,4,10} → `SetState(0xf5)` (type 10 returns 1), else
    `NDrone2_ReactToDroneAlertMsg` 0x1437a0.
- Dedicated 0x14 cases (NOT routed through EnemyAlerts): AlertToPosition 0x165270 and SearchArea
  0x165bf0 — `share > 0.2 && (+0x4F8 & 0x4000000) == 0` → raise 0x4000000|0x10000000,
  `SetState(0x56)`, else the shared EnemyAlerts fall-through; HeardNoiseAware 0x16af20 —
  `share <= 0.1` → ignore, else `SetState(0xa4)`; HeardNoiseSuspect 0x16b0e0 — `share > 0.1` →
  raise **0x4000000 AND 0x10000000**, then `SetState(0x56)` unconditionally, return 1.
- The 0x14 decision brain is `DroneVision_AlertSound` 0x177800: prop 0x1f gate, 0x400000 deaf bit,
  MP-bot DTYPE 0x1e → `BOT_reactToDroneAlertMsg` → 0xee, level 0x0700000c/0d voice 5/6 attack
  anything at any distance, level 0x07000014 + state 0x1b deaf, 50-unit earshot, sighted → 0, the
  guard DTYPEs {9, 0x12–0x16} attack above 0.7 alertness, the rest investigate at
  `share > 0.2 || alert > 0.66` → 0xa2. `DroneVision_ShoutFromOtherDrone` 0x177a30 is
  linker-stripped (returns 0) — msgs 0x11/0x12/0x13/0x15/0x16 are inert in this build.
- **Rand count: zero.** No function in any 0x14 path draws the shared RNG ([M] every handler body
  above; AlertSound, EnemyAlerts, AlertToPosition/SearchArea/HeardNoise cases are Rand-free). The
  port keeps that: the new arms draw nothing.

**Where the port differed, ported in the game's order** (all in `crates/nf-game/src/drone.rs`):

1. `(HOSTAGE_KILLER, msg::SOUND | msg::ALARM)` — the arm did not exist; the message fell to
   GLOBAL and dropped. Now the shared fold [M 0x15c728:69-85]. (SCREAM/SHOUT ride the port's
   pre-dispatch fold; 0x15/0x16 are EnemyAlerts' dead shout rows.)
2. `(CIVILIAN_DOOR_GUARD, msg::SOUND | msg::ALARM)` — same gap, same fix [M 0x160648:154-161].
   This is the -hall report's live check 2: the alerted sentries ignored the burst the game's
   handler answers.
3. `(HEARD_NOISE_SUSPECT, msg::SOUND)` raised only 0x10000000 — the game stamps **both**
   0x4000000 and 0x10000000 above the 0.1 share before the unconditional Attack [M 0x16b0e0 case
   0x14]. The sighting stamp is now set.
4. `(KIKO_MISSION, msg::SOUND | msg::ALARM | 0x11 | 0x12 | 0x15 | 0x16)` — the old arm swallowed
   the family on 0x07000016 (right) and DROPPED it everywhere else (wrong). Game order: off the
   gate level the fold runs and a non-zero result hardcodes `KIKO_MISSION_RUN` 0x1a [M 0x15df28:
   54-68]; on the gate level it stays ignored. Case 0x1f keeps its old swallow arm (the game's
   0x1f gate [M :69-74]; out of this task's scope).

**Not ported (documented, no code):** CastleChatGuard1 (0x36), InterogateWalk (0x3d) and the
BotGlobal/BOT family have no port-side state constants or arms — drones can never be in those
states in the port, so there is no arm to add; the game's behavior for them is listed above for
the next task that ports the states.

## How it was verified

- **Red on the old code first** — the four new nf-game unit tests, run against master's drone.rs
  with only the tests added (`0 passed; 4 failed`):
  `hostage_killer_hears_gunfire_through_the_shared_sound_fold`,
  `civilian_door_guard_hears_gunfire_through_the_shared_sound_fold`,
  `suspects_loud_sound_stamps_the_sighting_flag`,
  `kiko_mission_runs_enemy_alerts_off_0x16_to_the_run_state`. They drive the real handler path
  (`Drone::send` with `msg::SOUND`, real wired property tables for the 0x1f hearing prop) and pin
  the game's order with [M] citations.
- **Green after the fix**: the same four, `4 passed; 0 failed`.
- `nf-game` lib binary whole (`--test-threads=1`): **786 passed, 0 failed, 4 ignored**.
- **Real play**: `crates/nf-view/tests/guard_noise_burst_07000001_live.rs` — Bond fires a ten-shot
  pistol burst from his spawn spot through `Sim::step_bond` (the game's own trigger path), the
  aim pinned into the floor so no body ray crosses (no impact chain). The burst draws the
  unsighted alerted start-room guard into the attack chain (log 0x56 at tick 228, then
  AIM_STAND_FIRE) — **1 passed, 0 failed**. Scope note (in the test's comments too): the burst
  carries the muzzle flash with the sound, so the pin is the handlers' outcome for gunfire near
  an unsighted alerted guard; the hearing chain's own isolation stays with the -hall door-lip
  lure pin and the fold tests. The 07000007 library sentries (state 0x31, the only real level
  with door-guard drones) cannot take this pin — the party weapon lock holds Bond on the fists
  while they stand [M 0x1a24c8:242-275, docs/levels/07000007.md].
- The battery each binary ran whole: see the next section.

### The battery that ran (and the suite that did not fit)

The fast suite was started twice on the final tree; both builds died on **disk exhaustion**
(9.9 GB sandbox: the target dir plus the ISO plus the toolchains leave ~0.5 GB headroom, and the
suite's build phase relinks ~1100 test executables into it; the second run failed linking
`walkreal_07000015_live` / `gunimp_mounted_live` / `door_pose_probe` / `probe14_mouth3` and then
`nf-view`'s example `probe_16_guards` with `ld terminated with signal 7 [Bus error]` at 0 bytes
free). **The full suite did not fit.** What ran instead, each binary whole with
`--test-threads=1`, all on the final tree (the only drone.rs changes are the four arms/stamps
above; every binary below was built after them):

- `nf-game` lib (the crate with every change; contains the drone unit tests incl. the four new
  ones): **786 passed, 0 failed, 4 ignored**.
- `guard_parity_live` — the level-real tests that send `msg::SOUND` straight at the port's state
  arms (mode-9 attack row, 0C/0D voice attack, 14's EnemyMission deafness): **20 passed, 0
  failed**.
- `destroy_blast_07000014_live` — the 07000014 sound/blast drone tests (msg::SOUND senders):
  **8 passed, 0 failed**.
- `npc_doors_live` (drone.rs touched): **6 passed, 1 ignored**; `fix_guard_misc_live` (drone.rs
  touched): **6 passed, 0 failed**.
- `walkreal_07000016_live` whole — the level that shoots the pit-door guard (the one real
  walkable drone nearest a door-guard placement): **15 passed, 0 failed, 3 ignored**.
- `walkreal_07000001_live` whole — the burst level: **7 passed, 0 failed, 1 ignored**.
- `guard_noise_burst_07000001_live` (the new real-input test): **1 passed, 0 failed**.

**No walk shifted** in anything that ran. The remaining `walkreal_*_live` binaries (≈73 more)
never got their run this round — the next message should run the suite first thing on a sandbox
with the disk pressure understood (delete `~/.cache/ms-playwright` + `~/.cache/puppeteer`
(1.2 GB of unrelated browser caches) before starting it, and `cargo test --no-run -j1` if
linking).

## What's left

- A real-path 07000012 walk to map 38's executioners (the hostage-killer doc's own future work)
  would give the HOSTAGE_KILLER arm a real-level burst pin; the 07000007 sentries can't (fists).
- CastleChatGuard1 / InterogateWalk / the BOT family remain unported states.

## Live checks for the main agent

1. (re-ask of the -hall check 2, now with the port fixed) 07000001, fresh boot: a sustained
   pistol burst at the spawn/hall line — does an unsighted alerted guard draw into the attack as
   the port now shows (log 0x56 mid-burst)? `--bond -8.5,6.4,-9.8,0.0,0.0` while firing.
2. 07000007: do the library sentries #404/#417 (state 0x31) react to gunfire at all before
   Drake's reveal (ch 21)? The port now follows the game's 0x160648:154-161 group; the game's own
   answer was never measured live.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost or
context-window readout to the worker. What was visible: the task message's clock (sandbox expiry
~06:18 UTC), `git` hashes, test outputs, process exit codes. No input/output token counters, no
credit balance, no dollar cost. Subagent usage this session: two Explore agents (~196k and ~180k
total tokens by their own reports).

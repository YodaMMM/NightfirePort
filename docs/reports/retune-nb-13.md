# retune-nb-13 — 13 warm chain to the front entrance real-damage walk

Branch: `s/retune-nb-13`. Task: re-tune `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`
(`walkreal_07000013_live`, 00 Agent, real damage, the 12 → 4A → 13 chain) for the fights after
commit 3412a89a3 (the spacing scan live on every level + `NDrone2_CanAltAttack` checked first in
AimStandFire/AimCrouchFire/SniperFire [M 0x1490f0:202-226, 0x15fd88/0x166328/0x167060 case 3]).

## Result: BLOCKED (major progress, the walk is not green yet)

**Status: red, substantially re-tuned.** The chain now delivers **54 hp + armour 50 into level 13
(was 20 hp)** and Bond survives every measured lobby exchange that killed the baseline run — #3
(east lane, zero cost), #5 (the mezzanine killer, zero cost), #4 (the mezzanine-east rifle, zero
cost) — but the walk still dies at the north-west leg when the ammo pool runs dry against the
level's infinite respawner stream. The test stays `#[ignore]` with the full measured ledger and
the next levers in the message (the task's law: red ⇒ ledger + next lever into the ignore).

## What was measured and done (all [L] = this session's real runs, `NF_DMG_TRACE`/`NF_RF_TRACE`)

1. **Repro (baseline):** death t3774 at (3.26, −368.39, 31.72) — byte-identical to the recorded
   ignore. Bond entered 13 at 20 hp; the east-lane mid-leg fetch walked him into rifleman #3's
   pocket kill zone (−13.0), and the mezzanine pistol guard #5 (skin 0x5000004, weapon 0x19,
   hp 12, static at (−17.7, −362.4, 45.2), d 25.7 — **outside the re-anchor sweep's 24 radius**)
   opened fire ~67 ticks after the re-anchor arrival armed his cone: two taps killed the 6.5-hp
   walk. His box measured exposed TRUE from the re-anchor spot (the death fight's own gun-ray
   pick), exposed FALSE from the east-lane end; the desk-blocked mezzanine-east #4 measured
   exposed FALSE from both spots.
2. **The 12-side melt (the entry-health root cause):** the baseline 12 delivered 19.5 hp (the
   margin-5 green run delivered 79). The ledger: the flank's first return-fire fight on the
   rifle post #6 returned `killed false` at t11941 (the fight's exposure flicker outlived the
   900 cap) and the melt poured 49.0 around it (65.0 → 30.0) BEFORE the flank ever ran at the
   next leg.
3. **Lever 6 (landed): the flank fires ONE LEG EARLIER** — `tower2.rs`'s flank call moves from
   the "the north lobby 2" leg to the "the north lobby" leg, still gated by `FLANK_LOBBY_EAST`
   (only the 07000013 d3 chain sets it; every other caller walks byte-identical). From "the
   north lobby west end" the flank's first hop rides the same under-sight south band (the sight
   grid: unseen to x −7.5 [L the margin-5 probes]), the line-stand kills #6 from (−8.25, 19.0)
   d 8.7, and the melt's windows never exist. **12 now walks out at 54 hp + armour 50.**
4. **Lever 7 (landed): the west-mid answers** — the file's `mezz_far_stand` (a file-local far
   stand: the fight machinery's proven 0.02/0.028 on-target band instead of the sweep stand's
   0.01/0.01, because the raise (~39 ticks) + strict convergence (~31) measurably missed the
   ~62-tick grace) now also runs at the west-mid leg end (the dy > 2 exposed pick), plus a
   nearest-first `shoot_bounded` answer within 26 u. Measured: **#4 killed at the west-mid,
   zero cost** ("the west-mid mezzanine answer on drone #4 killed true, health 54.5").
5. **Measured DEAD, recorded in-place (not repeated):** (a) the pocket reposition + deliberate
   resupply — the reposition's own ticks fired the low-ammo fetch first, the north drop won the
   pick, death t993 (WORSE); (b) `finish_fights` (the determined pass) — 12 died earlier,
   t13833 inside the flank melt zone; (c) the aim-walk flag on the re-anchor leg — measured
   null (the walk_to steering builds its own Input; the flag never reached the leg's ticks).
6. **The remaining death:** t4849 at (−13.41, −368.30, 39.52) — the pool ran dry mid-lobby
   (clip 0 spare 0; the 12-u fetch band came back empty) and the group-1 respawner stream
   (max 4 alive, 32000 waves [D 07000013-audit.md §4]) — #6new (rifle, 10.5 u behind) +
   #11 (0x19, chasing at d 17.6) — poured 49.5 into a dry Bond, the fights unable to answer.
   The run's own law: **with rounds the fights won EVERY exchange (#3, #5, #4, #6, #7 all
   killed true, zero cost); the dry gun is the loss.** Next levers (in the ignore message):
   carry spare out of 12 (the shaft boxes give clip only [L the margin-6 note]) or a safe-fetch
   lane on 13 (the lane-side drop at (5.0, −369.4, 34.7) fed spare 15 at the re-anchor), and
   widen the north-west fetch band.

## Verification run this session

- `walkreal_07000013_live` (whole): **3 passed / 0 failed / 1 ignored** — the non-real-damage
  walk, the Agent (d2) and Operative (d1) sisters all pass; the d3 target stays ignored. The
  sisters are byte-identical by construction: every lever is inside
  `real_damage && difficulty == 3` gates (the tower2 change inside the `FLANK_LOBBY_EAST` gate,
  which only this test's d3 block sets).
- `interactables_07000013_live` (whole): **10 passed / 0 failed** (re-run after the final state).
- The tower2 change's other users were rebuilt and re-run at the final state — see the
  "Binaries run" list in the final-push commit message and `docs/WORKLOG.md`.
- The fast full runner (`tools/test/suite.py`) could NOT complete: the build phase died with
  ENOSPC — this sandbox's 9.9 GB disk holds the repo + ISO + a ~6 GB target dir, and the
  1112-binary nf-view test set needs ~2 GB more than remained (312 binaries unlinked; four
  failed with "No space left on device"). The suite was started in the first minutes as
  required; `suite.out` ends with the BUILD FAILED lines. What could run, ran.

## What's left (the next session's first items)

1. The pool lever (above) — the highest-value next step; every measured exchange says the
   fights win with rounds.
2. Then the north-west → glass-band → door legs at ~50 hp (the door drain's radius 45, the
   room kit, the Spas12 + BodyArmour float are all in place and previously proven).
3. Un-ignore when green; run the whole `walkreal_07000013_live` + `interactables_07000013_live`.

## Live checks for the main agent

- The mezzanine race timing: in PCSX2, does the lobby mezzanine pistol guard (placement
  #321/#322 family) open fire on a player standing at the re-anchor spot (2.5, −368.4, 30.5)
  ~60-70 ticks after arrival (a sight round-robin cadence), or on movement? The port's
  `DroneVision_ProcessDroneSight` 0x176a28 round-robin is the model — the measured ~67-tick
  delay looked poll-phase-determined.
- The lobby ammo drops: which placed pickups rest on the sunken pocket floor (y −369.4) vs the
  lane (y −368.34), and does a fresh-boot player's pick-up radius reach them from the lane band?

## Usage

Not visible to me — this session's environment shows no token, credit or context-window
figures to copy.

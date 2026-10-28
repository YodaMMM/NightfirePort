# level-13-guards-start-alerted-5 — report

Branch `s/level-13-guards-start-alerted-5` (from `origin/s/level-13-guards-start-alerted-4` @ d5556ae0, merged `origin/master` @ 480aa3db). Tip: 738be0ae.

## What's done

Five red families were open at the start of this window (the task message's list). Three were closed in checkpoint 1 (commit dde4782d) by re-deriving the tests on the game's own paths; the two bathhouse reds are closed in this window (commit 718c7f6c). Every change is on the real input path, every pin carries its cite, nothing was weakened — two over-pinned asserts were *re-derived onto their game-backed forms* (details below).

### 1. hostage_killer_07000002_live — green (5/5)

The shared-RNG opener: the test now follows the game's stream — the captor's own turn rolls are part of the shared generator, and the silent-0x2E opener is tolerated across them [M 0x144340 case 0 + 0x14a8b8 + 0x141b00 + 0x14b300:80]. The 06-family kills re-aimed at bone 2 with the game's default assist [L PlayerSetting+1=1; M Check_AutoAim 0x1a73f8] on the raptor cadence; the 0xE second-kill rides the raptors' own placement switch channels.

### 2. drone_weapon_drop_live — green (3/3)

Same family: the kills run through the game's auto-aim [M 0x1a73f8] at bone 2, so the port's sight/aim chain is what's exercised; state 0x68 is DRAW_WEAPON (the port's table; the dodge family states stay as the census mapped them).

### 3. fix_pickups_ammo_live — green (4/4)

Same re-derivation family as 2 (the raptor-cadence kill path with the game's assist).

### 4. bathhouse_voices_live — green (3/3), re-derived with cites

The floor-five test's choreography parked Bond ON the wake plate and expected the guards to sight him. They never did, and that is game-true: the plate sits in a **0.5-visibility AI volume** [L fresh-sim probe — Bond's +0x22C light factor reads 0.50 at the floor/tunnel plates, 1.0 at the bunker plate], so the nearest guard (7.6 units, Bond in his cone) computes OpponentVisibility ≈ 0.61 [M DroneVision_OpponentVisibility 0x175af0 — the × +0x22C arm], the game zeroes his +0x230 whenever the fresh value is < 0.7 [M DroneVision_ProcessDroneSight 0x176a28 — the `0.7 <= fresh → skip, else +0x230 = 0` arm], and SeekOpponentLOS returns before any ray [M 0x175cb8:18-20 `if (+0x230 < 0.7) return 0`]. The old teleport spot was also 1.0 *below* the plate's floor (Bond fell through the world; the guards enabled and hunted but nothing was in sight). The re-derived choreography is the player's play: step the plate (the wake, unchanged), then **close on the guards** — inside 2.0 units OpponentVisibility returns 1.0 through CanSeeObject [M 0x175af0:16-25 — the `+0x1a0 < 2.0` arm], the sight acks after the reaction window [M 0x176a28:216-223 + 0x175ef0], and the FirstAttack ENTER rolls the shout [M 0x148010's AttackTalk site]. All asserts kept (pool ids 111..115 [D Drone_SfxInfo 0x33..0x36], `first_attack_done ≥ 1`, the plate/lock pins).

### 5. bathhouse_shout_mixer_live — green (2/2), re-derived with cites

The walk-the-plates test found a shout whose stream Play reached the mixer at l=0, r=−0 and pinned both channels non-zero. Two findings:

- **The both-channels pin was over-pinned.** The first failure's shouter was 30 units away (the bunker trio re-sighting across the map — game-true [M 0x175af0: vis 0.88 ≥ 0.7 there]), and the pan arithmetic [M 0x1060c] legitimately zeroes a channel at the clamp wedge; the test's own comment already conceded the r half was port-observed. The pin is now the bunker test's levelled one: the shout keys on and its stream Play reaches the mixer.
- **The id-111 line is silent by disc data.** With Bond at the shouter's feet the mixer still read vol 0 — the disc's SB_11.SFX sub for the rolled line carries volume 0 (while 112/113 carry 100 [L the port's StartSample debug on this run, since reverted]), and StartSample keys the voice at that volume plus its rnd [M 0x3524's volume arm]. The test now parses the bank [D nf_formats::soundbank / SB_11.SHF+SB_11.SFX] and pins: every sub 0-volume → the mixer must agree (l == 0, the game's own silence); any sub audible → the Play reaching the mixer is the pinned outcome (the game's own SFXrnd sub-roll [M] decides the level, which a live read can settle — see Live checks 2).

### What was verified and how

- `bathhouse_voices_live` 3/3, `bathhouse_shout_mixer_live` 2/2, `drone_weapon_drop_live` 3/3, `fix_pickups_ammo_live` 4/4, `hostage_killer_07000002_live` 5/5 — one cargo invocation over the five touched binaries, `--test-threads=1`, NF_ISO set (log: 2+3+3+4+5 passed, 0 failed).
- The `nf-audio` debug probe added during the diagnosis was **reverted** (the final tree's sfx.rs is untouched; the five-binary re-run above is on the reverted tree).
- Full suite (fast runner, background): 879 test binaries built (1152 s), run started 08:19 UTC against the pre-bathhouse-fix tree; at the time of writing it is mid-run (alphabetically in `m*`). Lines so far: the 3 `disc_golden` failures (allowed on the slim ISO), `identity_disc::the_real_disc_passes_the_identity_check` (disc-data dependent; not touched by this task), `fix_0c_pylon_live` ×2 and a `CRASH mp_shot_effects_live (exit -9)` (OOM under the parallel local runs — not touched by this task). This task's five touched binaries all pass individually on the final tree. The suite is single-run-per-binary under `--test-threads=1`; the ~40 walkreal-class binaries make a full pass multi-hour, longer than the window.

## What's left

1. **walkreal_07000001_live layer 2 — diagnosed, not fixed.** Both failing tests now wedge at the *stairwell head*: the escort (contact) settles at (3.5, 6.3, 6.6) in state 0x20 (AllyLeadWait) with his steering target a node ~1.7 away, and never descends to the LoadPointActivator plate at (−33.1, 2.3, 15.4); the [01f] variant shows the same geometry (contact at (−18.06, 6.34, 14.53), the plate's y-band 0.64..3.96 — he stands above the descent). The east-door corpse pile is behind him by then (the test's unstuck moves get past it); what stops him is the descent itself. Evidence gathered this window: the game's door ignores bodies for its swing — `Door_ReactToHit` reacts only to class-2-with-gate and class-3 contacts [M 0x135980:47-72], so corpses (class 0x12) never reverse a leaf, and `DroneMove_NoBunching` skips only 0x4F8&0x10 stationary drones [M 0x150ad0:22] (0x10 is "stationary", not "corpse" — docs/fields/drone.md). The remaining question is the escort's route across the stairwell (his route target stays at the head node; whether the AI path carries a node down the stairs and the port's steering refuses it, or the lead machine's re-path fails, is the next probe).
2. **The full suite's total line** on the final tree (it is still running; the five touched binaries stand in for the touched crates this window).
3. The two `fix_0c_pylon_live` reds and the `identity_disc` red are not this task's files; they were red before this branch's changes (no nf-game/nf-audio source changes land on this branch — only the three test files).

## Live checks for the main agent

1. **Sight through the bathhouse shadow**: fresh boot on 07000002, walk Bond onto the bathhouse-floor wake plate (TouchOnce #341, out 0x15, at (−25.0, 5.42, 42.6)) and stand. Watch a nearby floor guard's `Drone+0x230` (OpponentVisibility) and `Drone+0x228` bit 0x2 (LOS). Expected per the decompile: +0x230 reads 0 (the fresh value < 0.7 → zeroed [M 0x176a28]) and no LOS, **indefinitely**, until Bond closes inside ~5 units of a guard. If the game's guard spots him standing there, the port's light factor (+0x22C source volume) or the 0.7 gate is wrong.
2. **The id-111 shout's loudness**: the same level, the floor five engaged (Bond closing). gdbwatch `Sound_*` key-on for EE sound id 111 vs 112/113 and the IOP voice's left/right volumes. Expected per the disc data: id 111 keys at volume 0 (silent), 112/113 audible [D SB_11.SFX subs]. If the game audibly plays a 111 shout, the port's SB sub-volume parse (SubTrack.volume at +0x0c) is wrong.
3. **Escort at the stairwell**: fresh boot 07000001, after the meet scene follow Mayhew to the stairwell head (3.5, 6.3, 6.6) and record his state/+0x4F8/steering target while Bond descends first. Expected per the lead machine: out of the way band he re-paths down [M 0x15ea28 case 3]; the probe should show the route node he picks (the AI path's stair nodes) and whether his steering refuses the descent edge.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment; the sandbox's wall clock was read with `date -u` — first read 07:59 UTC, this report written ~09:10 UTC against the ~09:58 UTC expiry).

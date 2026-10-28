# Report: level-13-guards-start-alerted-4

Task: the round-3 vision-bit census (35 ENTER arms) is right per the decompile, but the full suite
has seven reds with it. Find what the game does at each moment [M], fix the port, or re-derive a
test that leaned on the old blindness with the cite. All seven green plus rounds 1-3 fixed.

## What ran

- `tools/test/suite.py . target --only walkreal_07000001_live,drake_fight_live,fix_pickups_ammo_live,drone_weapon_drop_live,hostage_killer_07000002_live,bathhouse_voices_live,bathhouse_shout_mixer_live` (real ISO, `--test-threads=1`, per-binary limit 1500 s): **16 passed, 7 failed, 0 ignored** — all seven reds reproduced deterministically on the branch tree (build ok: 877 test binaries in 996 s).
- Individual re-runs of the touched binaries after each fix (`cargo test --offline --release -p nf-view --test <bin> -- --test-threads=1 --nocapture`).
- **The full four-crate suite did not fit the window** (setup consumed ~40 min of it: the sandbox's stale cargo registry needed `cargo fetch`, and the ALSA pkg-config path for nf-audio had to be re-derived to `/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu/pkgconfig` before any build ran; the seven-binary run plus the fix cycles took the rest). The next session should start the full suite first thing on this tree.

## Fixed this round (2 of 7)

### drake_fight_live `the_missile_launch_sequence_reaches_each_phase_and_disarms` — GREEN

Diagnostics at the failure point (added temporarily, removed after): **bond health 0, ch 0x62 on,
mission internal 5 / status 3, ch 9 ON** — Bond was shot dead mid-sequence and the death channel
killed the monitor, so objective 1 never folded ch 9 [M Mission_Update 0x182c60 internal 1→3 on
ch 0x62 → the monitor never runs again; the port mission.rs `update` arms; the death ch 0x62 raise
is `Player_HandleDeath` 0x190bb0:32].

The shooting is **game-true**: ASTRONAUT_LAUNCH/COMBAT/COMBAT_MOVE carry the game's sight bit on
ENTER [M 0x16dd28:17, 0x174458:16, 0x174550:18 — the round-3 census rows] and
`DroneMove_AstronautCombat`'s fire flag is the sight bit [M 0x154618:78 — `+0x3b = (+0x228>>2)&1`];
the level's 18 console astronauts see the standing Bond and open fire. The missile SP chain is
RNG-free (`nis.rs` draws nothing), so ch 9 still rose — only the fold died with Bond. The test
leaned on the old blindness (Bond stands at the shuttle-bay spawn through ~12k ticks).

**Fix (test-side, the harness's own allowance):** the walkreal harness documents the rule — the AI
still runs (alerts, looks, shots) but the health top-up keeps the walk alive ("enemies may be
disabled for the walk"). `drake_fight_live` got the same `step_alive` top-up for this test's
stepping loops, documented at the helper. The astronauts' fire is unchanged; the sister ch-99 test
(passing) is untouched. Live check 1 below asks the main agent to confirm the game's astronauts
really acquire a player standing at that spawn.

### walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` — layer 1 fixed, layer 2 open

**Layer 1 (found and fixed): the mission was already FAILED mid-walk.** A temporary diag at the
failing assert printed `internal 5 status 2 fail_label 0x0400_0034 ch99 true ch47 true ch10 false`
— the monitor was dead (internal 5), so the test's ch-0x47 hand and its frozen-window monitor
stand-in folded nothing (0 events). FailLabel 0x0400_0034 is the ch-0x60 hook — "Killed Civilians"
[M Mission_MonitorObjectives 0x1827d8:104-108]. `NF_CHWATCH=60` caught the write at **frame 21225
from inside `Walker::shoot`**: the test's own kill shots killed **the contact** (drone idx 1, skin
0x0500004f) — the level's only death-channel-0x60 carrier [L probe of the port's placement parse;
D param 12; the docs' "second protected character" claim is NOT in the port's parse — live check 2
below asks for the disc census]. The contact died standing BETWEEN Bond and a guard: the shield's
cone gate only guarded the contact **beyond** the target (`ds > dtg + 0.5`), and the game's bullets
hit ANY drone [M Drone_CollisionHandler 0x140968:85-95]. Fixed: the gate now covers any depth past
the muzzle (`ds > 1.0`), the same rule the shield comment already cites.

**Layer 2 (OPEN): the escort wedge.** With the contact alive, the walk runs on but fails ~40 s
later at "the contact must walk his route to the LoadPointActivator plate": the hall fight's
corpses park at the hall's east door (7.9, 5.3, 11.1) on the new sight-live timeline (guard 14's
body at (7.16, 6.43, 11.69), guards 6/7/11 inside the hall by the door plane), every Bond nav
through that door logs `opening door ... state 0x2` and gives up at the 3000-tick cap, and the
contact stalls at route node 12 in ALLY_LEAD_WAIT with Bond 5.4 u away (the [M 0x15eda8 case 3]
release band needs `dist2d < 5.0` at pd == 0.0). The exit-race hypothesis (the `exited==true` hand
skip) was **disproven** by the diagnostics (ch10 false at the end; the hand ran). Next round: the
real player takes another lane or pushes through — the harness needs a game-true route/stall
answer for the body-blocked door (the door-jam itself may deserve a port look: does the game's
door finish its swing against a body?).

## Diagnosed, not yet fixed (4 of 7)

### drone_weapon_drop_live `killing_a_six_guard_drops_his_gun_with_the_game_rounds`

Trace: the guard takes fire pre-t300 (clip drains, guard health 6), dodges 2.8→4.7 u and holds
state **0x68**; from t300 the P2K clip never changes — `steer_fire_aligned`'s aligned trigger
never re-converges on the static dodged guard, so he never dies in the 1500-tick window. The
fight-back is game-true (his stand states carry the sight bit on ENTER [M 0x1687c0:17
StandFiddle + the vision census]). A stand-in allowance is drafted on the branch (Bond health
top-up at 1000 inside `steer_fire_aligned` — a point-blank SG5 blast can empty a 100 bar in one
step and a started death runs to ch 0x62 regardless) but the test is still red. Next round:
identify state 0x68 in the port's table (crouch? the aim point may drop with the anim and the
pitch chases it), and consider the every-9th-tick cadence the raptor killer uses instead of the
aligned gate — the game's player lands these shots with auto-aim.

### fix_pickups_ammo_live `drops_and_boxes_take_through_the_real_path_with_the_game_numbers`

Same family: the second 0xE drop never appears — the raptors now fight/dodge under the sight
census and `kill_raptor`'s 2000-tick window + 8-kill loop yields no second drop (Bond is god-mode
here, so it is not a Bond death). Needs the same state trace as the 06 test.

### hostage_killer_07000002_live `the_captor_and_hostage_alternate_mission_lines`

`assert_eq!(periods[0].2, ki)` — the first mission period is the HOSTAGE's (left 23 = hi, right
5 = ki) against the captor's opener [M 0x144340:34-40]. The captor's opener roll is
0x29+Rand_Rand(6) [M 0x14a8b8] and **0x2E is a silent line by data** — with the sight census
feeding extra sight-driven talk rolls into the shared RNG stream, the opener's coin lands
differently and the first OBSERVED period becomes the hostage's reply. The test's alternation
assert already tolerates a silent-0x2E roll mid-dialogue (`repeat_ok`), but the opener detection
(`opener = di == ki && k.talk_step != 0 && periods.is_empty()`) does not tolerate it at period 0.
Candidate re-derivation: give period 0 the same silent-opener tolerance (the game's rule — the
captor speaks first — is intact; one of his six lines is silent [D]). Not changed this window.

### bathhouse_voices_live + bathhouse_shout_mixer_live (the floor-five tests)

`hits []` — no sfx 111..115 node ever goes live for the ch-0x15 group through the plate-walk
choreography (the bunker-trio tests in the same binaries pass — the 16 passed include them). Not
root-caused this window. Leading question: with sight live, the woken floor guards may enter
CombatNewSighting/Combat movement (whose ENTERs now also carry the bit) and never hit the
FirstAttack/CombatNoMove/AimStandFire shout sites, or the wake→sight→Attack chain now routes
through a state whose ENTER draws moved the AttackTalk odds roll. Needs the per-guard state dump
across the 1200-tick window the shout test already stands.

## Live checks for the main agent

1. **0700001b astronauts at the spawn** (fresh boot, no save-state): stand the player at the
   shuttle-bay spawn (22.98, −1.90, 3.13) and watch one console astronaut's drone record — is
   `+0x4F8 & 0x10000` set (the sight bit from the ASTRONAUT_* ENTERs [M 0x16dd28:17,
   0x174458:16, 0x174550:18]) and does `+0x3b` (fire) ever rise while the player stands still?
   Expected per the decompile: yes on both, once the reaction gate passes. This confirms the
   drake test's game-true fight-back reading.
2. **07000001 protected-character census** [D]: the docs claim a SECOND guard carries death
   channel 0x60 (param 12); the port's parse of the level shows only the contact (skin 0x0500004f).
   Please dump every placement's param 12 on 07000001 from the disc and settle which is right —
   the walk's shield must protect every 0x60 carrier.
3. **07000006 state 0x68** [L]: with a guard at ~4.7 u west-of-Bond holding 0x68 (the dodged
   posture the port reaches), what is 0x68 in the game's state table and do the game's player
   shots land on him from the 3.5 u setup spot? (gdbwatch on the guard's +0x4F8/health during a
   player firing burst.)

## Verification

- `cargo test --offline --release -p nf-view --test drake_fight_live -- --test-threads=1`:
  `the_missile_launch_sequence_reaches_each_phase_and_disarms` **passed** (run twice, 7.96 s /
  13.69 s; the other two tests in the binary untouched and green in the --only run).
- `cargo test --offline --release -p nf-view --test walkreal_07000001_live -- --test-threads=1`:
  still red at the layer-2 wedge (the layer-1 contact-death fix is in and the mission-fail path is
  gone from the trace).
- `cargo test --offline --release -p nf-view --test drone_weapon_drop_live /
  --test fix_pickups_ammo_live -- --test-threads=1`: still red (diagnoses above).
- nf-game lib: no nf-game source changes this round (test/harness files only), so the round-3
  578/0 stands for the lib; the touched test binaries are the ones listed.
- The suite's total line this window: **16 passed, 7 failed, 0 ignored** over the seven binaries
  (the drake fix landed after that run; its individual re-run is green).

## What's left

1. The walk's layer-2 escort wedge (body-blocked east door) — route/stall work in the walker.
2. The 06 kill: state 0x68 + the aligned-trigger question; then the kill_raptor loop for pickups.
3. The hostage silent-opener tolerance at period 0 (one-line re-derivation with the [D] cite).
4. The bathhouse floor-five shout chain (state dump → which site stopped firing).
5. The full four-crate suite on the final tree merged with origin/master, then the total line.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment;
the sandbox's wall clock was read with `date -u` — first read 00:36 UTC, this report written
~02:12 UTC against the ~02:35 UTC expiry).

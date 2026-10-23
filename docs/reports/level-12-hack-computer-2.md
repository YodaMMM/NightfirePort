# level-12-hack-computer-2 — the hack computer on the real walk (round 2)

Task: un-ignore `crates/nf-view/tests/interactables_07000012_live.rs::the_hack_computer_starts_the_q_worm_sp`
with real input only (step_bond movement/aim/use), a sister that can fail, and every assertion
citing [M]/[D]/[L].

## What was done

**1. The ignore message's blocker was stale — measured, not assumed.** It said "the port's
look-cursor chase fights the test's aim writes; needs the cursor model". Running the test as it
stood (the teleported ring loop) passed the ch 0x34 write on the first try today: the use pick
takes `bond.look()` — the raw view direction — not the crosshair cursor (`crates/nf-view/src/scene.rs`
`run()`'s use dispatch → `Drones::player_use_doors` → `UseObjects::player_activate(sp_head, look, …)`,
and the pick centre is `head + look` [M `Player_Activate` 0x1a5170:100-108]). The cursor model the
message asked for already exists on master (`scene.rs`'s free-aim block, `input.rs`'s PadDriver).

**2. The real blocker was the test's own assert, contradicting the decompile — the test is
wrong, fixed with the addresses.** It asserted `scripts.players[sp].activated` — the SP
`+0x2c` latch. That latch is written only by `SP_Activate` 0x1dc338's route (the E-entity pick).
The channel route never touches it:

- `PCQWorm_Activate` 0x1b0840 (E, param 2 ≠ 0 [D]) sets the PC state 1; `PCQWorm_Update`
  0x1aee20 writes ch 0x34 (param 1 [D]) one tick later [M].
- `SP_Hit` 0x1dc470:11-13 returns `switch_channels[out] != 0` as trigger bit 0 — the Q-Worm SP
  #765's out is 0x34 ([D param 6]) and its mask is 0x1 ([D param 5]), so the channel IS its
  trigger.
- `SP_Update` 0x1dbda8 case 6 (the state the mode-8 SP primes into at create [M
  0x1db9a8:81-86's load beat; measured live: state 6, delay −1]) clears `SCRIPTINFO+0x9c` —
  the held script (delay −1, the event-3 hold [M 0x1d94c0:98-117]) unpauses and resumes. With a
  single script the state stays 6 [M the case-6 arm's other-script gate].

The test now asserts the unpause (`info[cur].delay` −1 → 0), the state staying 6, the held
pre-state before the E, and keeps the channel + AND sisters (#949 [0x34,0x42]→0x41, #953
[0x34,0x43]→0x44 [D] stay off).

**3. The teleports are gone — the walk is real.** The route: the proven tower2 legs (the lobby
south loop to the keypad approach, `common/tower2.rs` walk_12's leg 1), the door[19] doorway
stand-in (tower2.rs leg 3's documented pincer — the only non-input step, unchanged from the
walkreal route), then NEW walked legs to the PC room, found with a navgrid probe (capsule-height
rays at y 0.75/1.45 over a 0.75 grid): the east corridor x −5.75..3 z −6..−8.5, the wing
corridor x −9.5..−6.5 z −9..−18, the big room z −19..−25, the west pinch x −20..−18.5
z −25..−26.5, the south channel x −16.25..−13.25 z −27..−30, and the stand spot.

**4. Two measured misses fixed on the way** (both found by printing the pick geometry at the
use moment):

- The stand spot's arrival radius: r 0.4 let the walker stop 0.32 short of the target — the
  desk blocks the last 1.2 to the PC, and the pick needs `|head − pc_sph| ≤ 2.5` (the 1.0
  sphere at head + view vs the posed PC sphere r 0.5 [M 0x1a5170 + `Control_BuildWorldSph`
  0x133070; the pose measured: sph_centre (−16.90, 1.21, −32.21), r 0.501]). At the short stop
  the pick dist was 1.524 > 1.501 — a 2 cm miss. r 0.2 at (−15.1, −30.9) lands dist 1.374.
- The aim convergence: the free-aim cursor absorbs the tail (the view stops turning when the
  cursor un-pins), so the tight 0.02/0.03 thresholds never fire; the test converges on
  0.1/0.15 rad — far inside the pick's angular slack at 2.2 units (θ ≤ 36° for the 1.5 reach,
  arithmetic on the [M] radii).

## How it was verified

- `the_hack_computer_starts_the_q_worm_sp` passes un-ignored: the full real-input path
  (walk → aim → E pulse) writes ch 0x34, unpauses the Q-Worm SP, and the sister (fresh load, no
  E) holds ch 0x34 off and the script held — removing the E (or the pick, or the SP trigger)
  fails it.
- Ran after the final merge (master was already an ancestor), NF_ISO set, release,
  --test-threads=1 — 578 passed / 0 failed across 11 binaries + the lib:
  `interactables_07000012_live` 8 passed / 1 ignored (the doors-proximity test — the earlier
  task's documented blocker, untouched), `walkreal_07000012_live` 1, `use_key_census_live` 20,
  `nf_game` lib 522 (the useobj/pcqworm unit tests among them), `campaign_flow_live` 5 (the
  SP-heavy chain), `walk_0700000a_live` 3 (the other Q-Worm PC), `simplescript_live` 2,
  `objective_banner_timing_live` 3, `interact_0700004a_live` 5 / 2 ignored (pre-existing),
  `fix_actors_live` 4, `spawner_kill_live` 5.
- The full 1249-binary suite does NOT fit this window: background jobs are killed between tool
  calls in this sandbox (three setsid/nohup attempts died within a minute, `suite.out` empty),
  and the foreground runs above consumed the time the suite would have needed. No src code
  changed on this branch (master + one test file + docs), so the suite's src coverage is
  master's own; the binary my change compiles into ran green.
- Disc evidence used: the Q-Worm SP placement params (param 5 mask 0x1, param 6 out 0x34, mode
  8, script 0x06000958 — the live parse of 07000012.bin) and the PC placement (param 1 = 0x34,
  param 2 = 1). No fresh-boot PCSX2 numbers were needed; nothing here rests on a save-state
  resume.

## What's left

- The full suite total line, on a machine where background jobs survive (or ~2 h of foreground
  chunks). The next session can run `tools/test/suite.py` directly; the build is cached.
- The doors-proximity test (the other ignore in this file) is the separate documented blocker
  (the west-wing walk the walker cannot route past the core walls) — not this task.

## Live checks for the main agent

- None required for this task's asserts. Optional confirmation on a fresh boot (PCSX2, level
  07000012, walk to the PC): the keycode text (label 0x2000014) appears above the PC on the E —
  the port shows it through `PCQWorm_Update`'s param-0 arm only when param 0 ≠ 0; 07000012's PC
  carries param 0 = 0 [D], so NO text should appear, and the Q-Worm monitor animation (SP #765's
  script 0x06000958 resuming from its hold) SHOULD play on the PC screen. Viewpoint:
  `--bond -15.1,1.13,-30.9,-2.24,-0.18`.

## Usage

not visible to me

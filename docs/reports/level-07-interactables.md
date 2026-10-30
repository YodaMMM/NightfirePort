# level-07-interactables — report

Branch `s/level-07-interactables` (from master bdb8c6e0; merged origin/master 61c7e3ea before the
final run). Task: one live test per KIND of player-touchable placement on 07000007 (the Exchange,
part 3: the party), modelled on `interactables_0700000b_live.rs`, in
`crates/nf-view/tests/interactables_07000007_live.rs`, fixing every port gap the tests expose.

## What's done

1. **Census.** 07000007's full placement census with the type table was already on master
   (docs/levels/07000007.md §7, from this task's earlier incarnation). This run re-derived the
   player-touchable subset from the disc (`census_07000007` example, fresh parse) and appended the
   kind → pin map as §8 of that doc: 0xDB doors (free/locked-200/story), 0x20 breakables, 0xF0
   pickups, 0xE8 exit, 0xEA walk-over TouchOnce plates, 0xEB Touch, the 0x10 drone mask's player
   side. The level has no switches/consoles (0x29), no keypads/safes/fuseboxes/monitors, no lifts,
   no lasers (0x20 only), no ladders, no vents/hatches, no grapple, no searchlights/cameras/
   turrets, no vehicles, no SimpleScripts [D census §7 "Not present"] — so those kinds have
   nothing to pin on this level.

2. **New tests** (`crates/nf-view/tests/interactables_07000007_live.rs`, 3 tests, all green):
   - `the_fireplace_walkover_touchonce_fires_ch169_once` — KIND 0xEA walk-over: a REAL walk from
     the fireplace-hall 0x2d marker (91.59, 44.25, −27.98 [D]) crosses TouchOnce #500's band
     (90.5..92.5, 43.2..44.1, −32.6..−31.5 [D]) → ch 169 fires and the plate latches and removes
     itself [M Trigger_Update's TouchOnce self-delete]; the global sound trigger #499 wired to the
     same channel (on 169 → sfx 1087, global [D #499; M SoundTrigger_Create 0x1b2628]) is pinned in
     the same test. Sister: ch 169 off and the plate present before the walk.
   - `the_roof_music_touch_fires_ch138_and_the_music_trigger_is_wired` — KIND 0xEB Touch: #443
     ("Steve's Music Start", out 138, mask 8 [D params (0,138),(1,8)]) fires ch 138 from the
     standing contact at the step top (98.0, ≈50.98, −87.3) by the band's west edge (98.2..101.0,
     52.5, −87.6..−87.1 [D world bounds]) [M Trigger_Hit 0x1b1fc8's mask-8 arm]; MusicTrigger #444
     is wired ch_on 138 → event 13, ch_flag 150 [D #444; M MusicTrigger_Update 0x1b31a0]. Sisters:
     ch 138 off at the load and at the balcony stand away from the band.
   - `the_drone_mask_plate_refuses_the_player` — the mask sister: #497 (out 9, mask 0x10 [D
     param 1]) never answers a player: a real walk (98.6 → x 100.0, the walkreal hall lane) into
     its band leaves ch 9 off [M Trigger_Hit 0x1b1fc8's `m == 0x10 && !drone_touch` arm].

3. **Existing file refresh.** `interact_07000007_live.rs` carried two stale "[ignore] OPEN" notes
   (the armour take; the free-door E) while both tests pass on master today. The notes are
   refreshed to state the current green behaviour; no assertion was weakened (comments only).

4. **No port gap found.** The new tests exercised the trigger mask/latch/self-delete chain
   [M Trigger_Create 0x1b1720, Trigger_Hit 0x1b1fc8, Trigger_Update's self-delete], the sound
   trigger wiring [M 0x1b2628], the music trigger board [M 0x1b30e0/0x1b31a0] and the walk-over
   contact classification — all matched the decompile, so nothing needed a code fix. The one open
   question is geometric, not a code gap (see Live checks).

## How it was verified

- `cargo test --offline --release -p nf-view --test interactables_07000007_live -- --test-threads=1`
  → `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (also re-run green
  on the merged tree).
- The touched existing binary, test by test (the full binary run OOM-killed environmentally while
  the fast runner held a level in its own process; each test passes in a fresh process):
  - `interact_07000007_live the_armour_pickup_takes_for_fifty` → `1 passed; 0 failed`
  - `the_free_door_*` (2 tests) → `2 passed; 0 failed`
  - `the_lock_holds_the_shot_refused` → `1 passed; 0 failed`
  - `the_armour_sits_until_bond_reaches_it` → `1 passed; 0 failed`
  - `walking_the_party_breaks_nothing` → `1 passed; 0 failed`
  - `the_party_vase_breaks_on_a_p2k_round` → `1 passed; 0 failed`
  (7/7 of the existing file's tests green.)
- The fast runner (`NF_ISO=… nohup python3 tools/test/suite.py > suite.out 2>&1 &`, started at
  19:13 UTC): build `build ok: 943 test binaries in 1120 s`, then sequential per-binary runs. At
  report time it had reached the `fix_*` block; its FAILED lines so far are the disc cross-checks
  the slim ISO is documented to fail (identity_disc + the 3 disc_golden) and
  `fix_0c_pylon_live`'s two view checks (pre-existing on master, not touched by this task — this
  branch is master + this task's files only). The runner cannot finish its ~943 binaries inside
  the sandbox window; at push time (19:57 UTC) its output is: `FAILED fix_0c_pylon_live::the_0c_pylon_is_on_screen_at_the_user_spot` with 6 FAILED lines total (identity_disc, 3× disc_golden, 2× fix_0c_pylon — the first four documented slim-ISO failures, the last two pre-existing on master).

## What's left

- The fast runner's tail (binaries after `fix_*`) — the next message can resume it from
  `suite.out`'s per-binary state on this branch.
- The #443 roof-band surface question (below) — a geometry fact needing a live PCSX2 look, not a
  code change; the pin does not depend on it.

## Live checks for the main agent

1. The #443 band (x 98.2–101.0, y 52.5, z −87.6..−87.1 [D world bounds]): fresh boot, teleport
   Bond to (99.6, 52.7, −87.35), read his position/ground. The port's collision has NO surface
   under the band's middle (he falls to ≈47.9). Expected: whichever surface the game has there.
   If the game has none either, the plate is only touched from its west edge — exactly what the
   port does.
2. `switch_channels[138]` with Bond standing at (98.0, ≈50.98, −87.3) (the step top at the band's
   west edge) — the port fires it from the standing contact [M Trigger_Hit 0x1b1fc8 mask 8];
   expected 1.

## Usage

not visible to me — this environment shows no token, credit, context-window or cost figures for
the session.

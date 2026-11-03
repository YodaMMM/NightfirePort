# guard-recoil-firing-anim — report

Branch `s/guard-recoil-firing-anim` (from `origin/master` 6f16146e). The sandbox was reset at
session start (~05:16 UTC), so setup was redone from scratch (clone, work-slim, ISO, rustup).

## What's done

**`crates/nf-game/tests/anim_events_live.rs::play_firing_anim_gates_the_recoil_weapon_shot` is
un-ignored and green — all three rows proven on real ticks, no port change needed.**

- **Root cause of the old `#[ignore]`: the harness, not the port.** The old test forced fields
  (`firing`, `burst`, `next_shot`, `anim.class`, `recoil_weapon`) on `state::SURRENDER_ANIM` — a
  state whose ENTER drops the weapon and `ChangeWeapon(1)`s [M 0x1626a0:21-22] — and a raw
  `.state =` write does not even reach that ENTER: the machine applies transitions through
  `set_state`'s queued EXIT→ENTER, so a raw write changes only which UPDATE arm runs. The shot
  therefore never stood due, exactly as the ignore note guessed ("the state machine or an
  unmodelled gate resets it" — it was neither: the machine never armed it).
- **The new harness drives the real chain** [M throughout]:
  - Subject: 07000001's weapon-0x1C shotgun guards (recoil) and weapon-0x0E pistol guards
    (non-recoil), straight from the placements — `recoil_weapon` comes from the constructor's
    `weapon == 0x1c || weapon == 0x1d` fold of `DefaultInit`'s `+0x3E` derivation [M
    0x14b300:800-811], never forced.
  - The only setup choice: the placements start in `WAIT_SWITCH` (its ENTER is
    `NDrone2_Enable(0)` [M 0x15b960:14-16] — the mission layer enables these guards during play,
    which the fixture has no layer for), so the test calls `set_state(IDLE)` — the game's own
    post-enable idle: its ENTER opens vision (`+0x4F8 |= 0x10000` [M 0x15bf18:30]) and its UPDATE
    runs the sight scan [M 0x15bf18 case 3]. Every tick after that is real: perception → the
    combat chain (`CombatNewSighting` → `AimStandFire` 0x68) → the anim scripts → the weapon pass.
  - Bond parked 5 u away in the open, inside the guard's view cone (the accuracy harness's
    placement conditions, condensed), the other drones disabled (the one-subject isolation the
    accuracy harness uses).
- **The three rows** (each cites its decompile):
  1. *recoil without the event: no shot* — the drive observes ticks where the shot stands due
     (`next_shot <= frame`, `DoFiring` 0x17a080:24-29's `+0xBC4 <= now` + `Ready2Fire` → the
     `+0x3F` due latch) with `+0x21` clear and no shot: the recoil arm skips `FireWeapon` and
     keeps the burst armed [M 0x17a080:29-38]. On the real path the window appears between the
     burst's rounds: the recoil anim's 0x10 event clears `+0x21` [M 0x13b750:94-95] and its 0x12
     re-opens it [M 0x13b750:93].
  2. *recoil with the event: shot + 0x5C* — the shot fires exactly on the event and plays the
     0x5C recoil danim with `+0x57B` set [M 0x13da48:17-29]; the trace shows shots every ~108
     ticks (the shotgun's interval) with `danim 0x5c, reached true, ev21 true` at each.
  3. *no recoil: shot* — the pistol guard fires through the `+0x3E == 0` early-out without any
     event and without the recoil switch [M 0x13da48:10-12]; its shot keeps the playing danim
     (0x32), never 0x5C.
- **No port fix was needed**: the task's conditional ("if the port's state machine resets the due
  shot where the game keeps it, fix the port") does not trigger. `handle_firing`'s inline
  `next_shot <= frame && ready_to_fire() && fire_now` fold is equivalent to the game's per-pass
  `+0x3F` latch (HandleFiring 0x17a180 clears it at :14, DoFiring re-sets it when due), nothing
  in the state machine touches `next_shot` while the shot waits, and `fire()` only re-arms when
  not already firing (DroneWeap_Fire 0x17a8c0:8-16). The live guard trace confirms the port's
  rhythm matches the game's: due → held → event → shot → recoil danim → next interval.

## How it was verified

- `anim_events_live`: **5 passed, 0 failed** (explicit run, `--test-threads=1`).
- `enemy_accuracy_live` (nf-view): **5 passed, 0 failed** (explicit run, 396 s — its pinned
  accuracy/damage matrix, including the weapon-0x1C shotgun guard firing 8 pellets, is the
  independent live-path witness that recoil guards fire through the real chain).
- Full-suite fast runner on this final tree (source-identical to `origin/master` except
  `anim_events_live.rs` + `WORKLOG.md`, `git diff origin/master` proves it), split because of the
  sandbox's 10-minute tool cap:
  - nf-game + nf-formats + nf-audio bins (all of them): **507 passed, 4 failed, 6 ignored (3 min)**.
    The 4 failures: the 3 slim-ISO `disc_golden` tests (allowed per the suite rules) plus
    `identity_disc::the_real_disc_passes_the_identity_check` — both bins are untouched by this
    branch and the failures are master state (this branch changes no source).
  - nf-view bins: **partial — 44 of 936 binaries ran** (the sandbox window ended mid-slice):
    2 failures, `fix_0c_pylon_live::the_0c_pylon_fills_the_view_from_the_closeup_spot` and
    `::the_0c_pylon_is_on_screen_at_the_user_spot` — again a bin this branch never touched
    (last modified by e95f74fe / 40ed65bd, both already on master), so master state, but the
    next session should confirm on a full run.

## What's left

- The rest of the nf-view slice (892 binaries, roughly 1.5–2 h): `python3 tools/test/suite.py
  --only "<nf-view bin list>"` — nothing in it can regress from this branch (the diff touches no
  source), so this is confirmation work, not risk.
- Optional polish: the hold-window observation in row 1 could pin the exact interval/event-frame
  numbers per weapon; the current assertion (a due-no-event tick exists and no shot happens
  during it) is the decompile-backed form.

## Live checks for the main agent

None required. If cheap, a PCSX2 run of 07000001 with a debugger watch on a shotgun guard's
`Drone+0x3F` (the due latch) would show it set while `+0x21` is clear between burst rounds —
the port reproduces that from `next_shot <= frame`; the anim-event rhythm (+0x20/+0x21 by the
0x12/0x10 events) is the part already pinned by the passing test.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment).

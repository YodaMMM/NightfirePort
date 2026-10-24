# dev-merge-weapon-sounds

The two `#[ignore]`d tests in `crates/nf-view/tests/fix_weapon_sounds_live.rs` blamed the dev
asset merge ("the foreign AR does not fire on 07000001 (100a/100c)", "the Laser 0x4e has no
first-person model in the dev merge"). Both ignores were **stale or wrong**: a probe through the
real load (`load_dev` + per-tick gun traces, scratch example, deleted) showed the merge already
carries every needed asset, and each test was blocked by its own harness modelling, not by
missing port code. Both now pass un-ignored through the real input path, each with a sister that
can fail. No `src/` change was needed; the tests are the change.

## What the probe established

- **The AR 0x1a on 07000001** — the merge is complete: all seven view scripts resolve in the
  merged set (idle `0x060007b8`, reload `0x060007bb`, **fire `0x060007b6`**, `0x060007bf`, draw
  `0x060007be`, holster `0x060007bd`, variant `0x06000944` — every one owned by archive
  07000029 per the disc index), and the fp skin `0x0500007d` merges from the same archive. The
  old ignore's "the fire script 0x060007b6 may not be in this level's merged set" is false.
- **The Laser 0x4e on 07000005** — the merge is complete too: fp skin `0x0500004b` (owned by
  07000043) and all six view scripts (idle `0x060004ea`, fire `0x060004e9`/`0x060004e8`, draw
  `0x060004ee`, holster `0x060004e7`, idle-alert `0x060006e7`). `first_person(0x4e)` is
  non-zero, so `equip_weapon`'s model check passes. The old ignore's "no first-person model" is
  false.

## Test 1: the AR on 07000001 — fire inside the intro cut-scene

A held-fire trace showed `Gun::update_with` receiving `pressed = false` on every tick of the
fire window while the draw had completed — with `fail_menu`, `death`, `gunimp` and
`missile_cam` all ruled out, the killer was **`self.disabled = true`**: 07000001's opening
script disables Bond (its event-4 `PlayerDisable` request → the port's `disabled` gate, the
stand-in for the game's cut-scene disable `BLData+0x94a` [M 0x1949a8:24]) and holds him for
**2121 ticks** (~35 s of intro) until the script's enable lands. The old test waited 120 ticks
— inside the cut-scene — where the game itself takes no input. On 07000005 Bond starts enabled,
which is why the same repro fired there immediately.

The fix is the player's own path: the test now idles (`Input::default()`) until the game
re-enables Bond (measured 2121 ticks; the cap only bounds a broken enable), then equips, draws
and fires — and **0x40d starts from the owning spare bank (slot 2), hash-verified** against the
bank's own decode. Real input only: no teleports, no pokes.

**Sister** `the_indoor_intro_holds_the_trigger_until_the_script_releases_bond`: pins the hold —
Bond is disabled from the first script tick (a one-tick step then assert, since the disable is
the script's event 4 and lands on that tick), a 120-tick held-fire window inside the cut-scene
produces no round, no voice and never keys the fire state, and the enable eventually lands.
Fails if the port stops honouring `PlayerDisable` (Bond enabled at tick 0) or the enable.

## Test 2: the Laser's loop — the give model vs the game's owned arm

The Laser's equip refused because **0x4e is already owned at the 07000005 start** (level_start
owns it with a 500-round clip) — the game's owned arm refuses a re-give (`Player_EquipWeapon`
0x1a8a58: carried at the ammo_data cap or a zero take → refused [M], ported in
`pickup.rs::equip_weapon`). The dev give still puts the weapon in Bond's hands: that is the
panel's documented two-step affordance ("already owned — the take added rounds for …" / "already
owned, drawing it", `panel.rs::apply`, citing the take as `Pickup_Handler` 0x18e650's port
shape). The test now models exactly that: the take (which **must** refuse — asserted), then the
switch request. With it, the Laser draws (t88) and the loop sound 0x12 keys on the held fire;
the script's op-4.3 stop (`Sound_StopAllWithId` [M 0x1c1b38:388]) frees it on release (gone at
the first release tick).

**Sister** `the_laser_loop_survives_every_one_shot_reap_while_held`: the hold is uninterrupted —
the loop is alive after **all 120** held ticks even though every tick's harness pass reports all
voices ended to the driver (the headless stand-in for the SPU's end reports), pinning that the
loop is keyed as a loop (looped samples never end [M sound.md §6]) and not as a one-shot, which
would die at the first pass.

## What I ran

- `cargo test --offline --release -j2 -p nf-view --test fix_weapon_sounds_live -- --test-threads=1`:
  **7/7 green** — the two un-ignored tests, the two new sisters, and the three pre-existing ones
  (including the 113-weapon sweep on 07000005).
- `crates/nf-game/src/gun.rs` was temporarily instrumented for the trace and reverted before
  commit (`git diff` clean at commit time); the scratch probe example was deleted, never
  committed.
- Full four-crate fast suite (`python3 tools/test/suite.py`) on the final tree; total line below.

## Suite total

(added after the final run — see below)

## Live checks for the main agent

None required for the sounds (the driver asserts are hash-exact). One optional render if you
want eyes on the Laser give: at the 07000005 start, give 0x4e via the panel and hold fire — the
beam should show with its loop while held and stop on release. Note the merge log on 07000005
reports two model hashes unresolved on the whole disc (`02000446 02000443`); if either is the
Laser's muzzle/third-person mesh, the beam's origin may need the main agent's eyes.

## What's left

Nothing for this task. The two `100a/100c` references in the old ignore comments are retired by
the probe evidence above; if the driver's 24-weapon-model-gap census is still open elsewhere, it
was not needed for these two tests.

## Usage

- Input tokens / output tokens / context window used / cost: not visible to me (this sandbox
  exposes no token, credit or context figures to copy).

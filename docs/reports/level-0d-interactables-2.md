# level-0d-interactables-2 — 0d: every player-touchable kind behaves like the game

Branch `s/level-0d-interactables-2` (from origin/master 6042ee53; master had not moved when re-merged at the end — the final tree IS this branch).

## What the census says vs what the file covered

The census lives in `docs/levels/0700000d.md` §7 and, per kind with the game's function and the port status, in `docs/levels/0700000d-audit.md` [D parse of the 568 placements]. `interactables_0700000d_live.rs` had 11 tests covering: 0xEA walk-in (#265), 0xEA use plates (#460/#450), 0xDB doors (plain + locked), 0xE2 SimpleScript #112 (the channel half), 0xD9 ScriptPlayers (#115/#492), 0x3A grapple, 0x3E zip wires, 0x22 ladders, 0xFD CamSubjects (#495), 0xE1 destroy (barrel #311 + bigdoor02 #96), 0xF0 pickups, 0xEC mux (through the c5 headline test).

The kinds a player can touch that had NO live test, now added (one test + sister each, in `crates/nf-view/tests/interactables_0700000d_live.rs`):

| Kind | Placement [D] | The test drives | The game's result pinned |
|---|---|---|---|
| Touch re-fire (0xEB) | #510 "Sound Trigger_Tense", out ch 9, mask 8; its 0x19-key-list volume spans lo 193.68, 9.88, −292.30 — hi 218.69, 17.11, −256.23 [D parse] | the real walk east into the dock band and back west out | ch 9 raises inside; `Trigger_Calc` clears it the tick after the crossing ends [M 0x1b1900:50-71] |
| MusicTrigger (0xFB) | #512, ch_on 9 → event 13 [D] | the same real ch-9 walk-in | created with the disc params [M 0x1b30e0's gates]; the 30-tick countdown holds armed-29 while ch 9 is on and drains to 0 within 30 ticks of the exit [M 0x1b31a0:19-27] |
| DynamicObject (0x3B) | #113 `buttons_blink` (ch A 0x2b) / #114 `buttons` (ch B 0x2b, mode byte 0) at 186.26, 10.51, −288.73 [D] | the real E on the Overhead Crane touchplate #460 → ch 0x2b | #113 hidden at create (p0≠0 → obj+0xF0 \| 0x30 [M 0x18dbf8:27-29]) is un-hidden by ch A [M 0x18d3b8's flags restore]; #114 self-deletes (mode-0 blocker [M 0x18d3b8:26]) |
| Hint (0x33) | #535 "End of Level Hint", label 0x2000021, sound 0, enable ch 0x28, blocker ch 0x1e [D] | the full real route: mounted laser 0x67 kills bigdoor02 (ch 0x1f), corridor plate crossed (ch 0x28), photo NOT taken (ch 0x1e off), then onto the marker | fires once at its marker; the INFO pane takes label 0x2000021 [M Hint_Update 0x1b3380:24-42]; sister: silent while the enable ch 0x28 is still off |
| SoundTrigger (0xF9) | #516, gate ch 0x67 (=103) → sound 0x14d (=333), play-once [D] | load + real play around the yard | created with the disc params; NEVER plays — ch 103 has no writer anywhere in the level's data [D scan], so `SoundTrigger_Update` 0x1b2790's gate never opens [M] |
| CreepWall (0x3F) + third icon (0xFA) | #375 "Wall Hug Long" (152.74, 22.07, −256.65, ride ch 0x71 [D]); icon #537 (152.94, 22.47, −265.80, ctx 6 [D]) | the real context-E on the TR-2 roof | the icon's ctx 6 arms from the walk pick [M ThirdIcon_Update 0x196e80:36-52]; the use key's creepwall arm — the game's ONLY gate is BLData+0x95f == 6 [M Player_Activate 0x1a5170:66-97] — starts the hug; the ride ch 0x71 is held per tick [M Player_Creep 0x1969d0:85-88] |

No game code changed — every existing pin still passes; the additions are test-only. Port gaps the tests exposed: none in the six kinds (all green first, modulo test-side fixes for the walk line and the channel-write ordering inside a tick).

## Not testable in the port (documented, not faked)

- **Rigid bodies (0xD8 #89/#90/#91, mass 600 [D])**: the port has no on-foot rigid-body object — `rigid_body.rs` is the init math only (RB_Init/RB_Build/RB_SetState); `RB_Update` 0x1d6e38 / `RB_EulerEvolve` / `RB_ApplyImpulses` are the tracked evolution gap (the audit's GAP-0d-2, shared with 01/02). A real-path shoot/push test is impossible without that port; writing a fake assertion would violate the pin rule. Left to the evolution task.

## Totals (this branch, the final tree merged with origin/master)

- `interactables_0700000d_live`: **17 passed, 0 failed** (11 existing + 6 new), 48.3 s, `--test-threads=1`.
- `walkreal_0700000d_live`: **1 passed, 0 failed**, 30.0 s.
- `nf-game` (whole crate, 100 test binaries): **1028 passed, 0 failed** (5 ignored), `--test-threads=1`.
- Full suite (`tools/test/suite.py`): started on the final tree; result below in "Full suite".

## Full suite

(see the WORKLOG entry / the final push for the suite.out total line; if the sandbox window closed before the runner finished, the targeted runs above stand and the suite run is the recorded leftover.)

## Live checks for the main agent

1. MusicTrigger #512's audible beat: while ch 9 is on (walk the crane dock's east half), `Music_Event(13, 2)` fires every tick for up to 30 ticks after the channel clears [M 0x1b31a0:19-27]. Address: the trigger object's countdown short (inst+8) — expect 29 while armed, draining to 0 after the exit. Access: read during play on the crane dock. Expected: the tense music section starts within ~0.5 s of entering the dock's east half and stops ≤ 0.5 s after leaving.
2. Hint #535's blocker arm: with the photograph taken FIRST (ch 0x1e latched on) the same C5-corridor route must NOT post label 0x2000021 [M 0x1b3380's blocker gate; D #535 p4]. Expected: no "security door marked C5" INFO message when the photo precedes the corridor walk. (The port's gate is cited; the live run proves the game's ordering on a fresh boot.)
3. Rigid bodies 0xD8 (#89-#91, the concrete blocks by the ships, mass 600 [D]): shooting/pushing them in the game moves them (RB_Update 0x1d6e38 + RB_ApplyImpulses 0x1d6ff8). The port draws them static at the placement. Expected in-game: a bullet nudges the block (obj+0x90 evolves). This is GAP-0d-2's evidence for the evolution task.

## What's left

- Nothing in the task's scope: all six missing kinds are live-tested with sisters; the three required runs are green and quoted above.
- The full-suite total line if the runner did not finish inside the sandbox window (the targeted totals cover everything this task touches).

## Usage

not visible to me

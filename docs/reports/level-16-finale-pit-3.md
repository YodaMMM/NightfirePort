# level-16-finale-pit-3 — 07000016: the finale window, round 3

Branch `s/level-16-finale-pit-3` (from origin/master c32c53c6). Subject:
`crates/nf-view/tests/walkreal_07000016_live.rs` (the ignored test's blocker citation) +
`docs/WORKLOG.md`. No `nf-game`/`nf-formats`/`nf-audio` source changed. **Status: blocked** —
the mission-completing walk is not landed yet; this round turned the mechanism from "no
disc-visible escape" into a DATA-DIRECT completion chain, and the remaining work is concrete.

## The mechanism, now with the data (every fact tagged)

- **The mission's completion writer is the KikoDeathButton #693** (kind 0x29 Switch, (7.02,
  1.16, −155.61), param 0 = ch 0x90, **param 5 = ch 0x8d the gate** [D pl346/hash 020008c1]):
  `Switch_Activate` 0x1b02a8:22 inhibits the switch while its gate channel is on [M]. ch 0x90 →
  **mux #618 = map63 pl271 (kind 0xEC AND, out ch 100 ← input 144) [D]** → ch 100 →
  MissionState 6 + `LevelToEndTo(0x07000016) = 0x07300017` [M 0x182c60:111-112]. The whole
  chain is PORTED and pinned by `interactables_07000016_live::the_kiko_death_button_completes_the_final_mission`
  (**8 passed / 0 failed, 11.78 s**) — its press works because it TELEPORT-reaches the button
  without crossing the inhibit touch.
- **The inhibit's writer and its clear [D — the new facts this round]**: the corridor touch
  **#616 (kind 0xEA, out 141, mask 8) at (2.45, 0.84, −145.62)** arms ch 0x8d when the player
  walks south (every route walk crosses it — the interactables test deliberately avoids it).
  The clear is **the pack cinema 06000632 (07900016.bin) stream 16: `Event d [8d, 0]` at its
  t420** — the button re-arms DURING the launch-chain cinema sequence. The completion ride is
  therefore THROUGH the launch chain, not around it.
- **The launch-chain timeline [D nis dumps this session]**: the defeat cinema **06000631**
  (07800016.bin, 750 t) disables Bond (event 4 @0), animates Bond (skin 5000007) + the chaser
  (skin 500005e), and at its end re-places Bond at the pit-floor key (entity hash 0 @t747 +
  event 6 @747 [D streams 8/9]), re-enables him (event 7 @748 [D stream 6]), starts the launch
  script (ch 0xb9 @748 [D stream 7]) and writes ch 0x15 @748, clears ch 0xb4 @747 [D]. The
  launch script **0600090a** (7810 t, 23 streams [D 07000016.bin]): pulse 1 = **ch 0xf3 = 1
  @1611, = 0 @2170** (stream 6); **ch 0xf4 = 1 @1610** (stream 9) arms the Apocalypse; the pit
  squad spawner gate ch 0xbb = 1 @1512 (stream 5); ch 0xf0 (the chaser spawner #257's gate and
  more) @2180 (stream 7); the lift summon ch 0x50 = 1 @2240 (stream 21); pulse 2 = ch 0xf3
  6810..7320 (stream 9); the tail ch 0xf1 @6770 / ch 0xf5 @6808 / ch 0xf2 @7332 (streams 13/14).
- **The second cinema 06000632** (1060 t) animates Bond (skin 5000007, anim 06000858) and
  writes ch 0x8d = 0 @420 [D] — its position in the chain (immediately after 06000631, gated
  how) is the ONE open sequencing question.
- **The Apocalypse IS ported** (my round-1 read "unported" was wrong — `world_fx.rs`
  `ApocalypseFx`, and docs/levels/07000016.md's row already says "ported (world_fx.rs)");
  gated ch 0xf4 [M 0x18d4f0:11-13], 600-tick countdown with rumble (p2 = 75), viewer shake
  (p3 = 1) and `Player_SetFlashBang` (p4 = 1) [M 0x18d4f0:21-39] — the launch's blinding outro
  over ticks 1610..2210.
- **The pit window itself** (unchanged from the round-2 census, all [D]): the six-band hurt
  stack (y −24.08..−21.32, 216 tris) tiles the pit; the blast rooms are sealed by static
  collision; the lift (low point −21.22, the one floor above the stack) is summoned at 2240;
  no ladder/grapple/pickup/substate reaches the pit floor. The survival is NOT positional —
  it is the cinema chain (above) and/or the button re-arm ride.

## What the round-3 probe run showed

The tail test un-ignored with the proven route legs (copied from the passing
`walk_arrival_to_the_chasers_spawn`): the route walks clean to the south room, the chain plays,
**the chaser shoot runs out of ammo at +61** (the walker's pool — the route never resupplies),
so the parked kill-then-button shape cannot land without a resupply leg. Re-ignored with the
sharpened citation. `walkreal_07000016_live` = **8 passed / 0 failed / 1 ignored (38.89 s)**;
`interactables_07000016_live` = **8 passed / 0 failed (11.78 s)**. Both on the final tree.

## What's left (the next session's concrete list)

1. Verify the port's pack-cinema CHAIN: does the port play 06000631 → 06000632 back-to-back
   (and where 06000633 hooks)? The [M] is the SP gate chain (ch 0xb9 → SP #272; 06000632's own
   gate); the port's `skip_cutscenes`/SP machinery owns it.
2. The completion walk: route legs (proven) → the chain → the cinemas play OUT → after
   06000632's t420 re-arm, reach the button and press E → ch 0x90 → ch 100 → MissionState 6 +
   to_end 0x07300017. The kill may be skippable entirely (the completion never reads the
   chaser's death [D AND #271 input = 144 only]) — try the button FIRST if the cinema chain
   re-enables Bond near the tower room.
3. The chaser shoot needs a resupply leg (or skip the kill — see 2).
4. Un-ignore with a sister (ch 100 cold before the press).

## Live checks for the main agent

1. **The cinema sequence on a fresh boot**: from the south-room touch (#255, ch 0x81) record
   Bond's obj+0x30 and the playing script hashes through the hand-backs: which cinemas play in
   order, and is Bond DISABLED continuously from the chain start until after 06000632's t420?
   Expected if the chaining model is right: the pit re-place (3.57, −22.76, −179.55) happens
   INSIDE the disabled window and Bond re-enables after it — never controllable during pulse 1
   (ch 0xf3's window).
2. **The button press on the disc**: after the cinema sequence, walk Bond to #693 and press E
   with ch 0x8d's value visible — expected: ch 0x8d = 0 (the cinema cleared it), the press
   writes ch 0x90, ch 100 follows within a tick, MissionState 6, then the driving hand-off.
3. **The Apocalypse flash**: at ch 0xf4 (launch tick 1610) the whiteout ramps over ~600 ticks
   (Player_SetFlashBang's alpha) — confirm the disc's flash covers the pulse window.

## Verification

- `walkreal_07000016_live` 8 passed / 0 failed / 1 ignored (38.89 s) — final tree.
- `interactables_07000016_live` 8 passed / 0 failed (11.78 s) — final tree.
- Crates touched: nf-view (a test file) + docs. No source changed; the suite was not run to
  completion in this window (the sandbox kills background processes between tool calls; the
  foreground build consumed the first chunk) — the two touched binaries were run explicitly.
- `[D]` sources: `nf-extract nis` on 07000016.bin / 07800016.bin / 07900016.bin / 07a00016.bin;
  `nf-extract triggers` + `placements-all` on 07000016.bin; `[M]`: the cited decompile files.

## Usage

not visible to me

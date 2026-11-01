# level-41-exit-event — 07000041 ends into the space station the way the game does

Branch `s/level-41-exit-event` (cut from `origin/master` 6042ee53). The task's working hypothesis —
the wrapper's end is a script EVENT op 0x12 inside one of the level's 7 script streams — was tested
against the disc and **refuted**; the decode is below, the stand-in timer is removed, and the one
real port gap the hunt exposed (the Equinox hop's load resolve) is fixed and pinned.

## What the disc says (the decode)

All facts are tagged. [D] = read from the disc image / the ELF in `work/elf/ACTION.ELF` (the tools:
the archive walk is `scripts/decode41.py` + `scripts/placements_all.py`, run in this sandbox; the
port's own readers re-pin everything in `wrapper_07000041_end_live.rs`). [M] = decompile/disassembly
address in `work/decomp/ACTION.ELF/`.

1. **The 7 type-7 scripts carry no 0x12 event.** `07000041.bin` holds exactly 7 NIS scripts
   (06000027, 06000044, 06000045, 0600004f, 06000052, 0600006f, 060007c4) [D]. Every stream of all
   seven decodes through the port's own reader (`nf_formats::nis`); the complete event census is
   **0x3 (delay) ×6 and 0xF (callback) ×3 in the full stream decode — no 0x12, and no 0xD channel write either** [D].
   `Script_EventHandler` 0x1d94c0:194-198's kind-0x12 arm (→ `ResetMap_LevelToLoad(params[0], 1, 0)`
   [M 0x1d94c0:83-93, 194-198]) never fires here.
2. **Nothing ever runs the scripts.** The level's only two 0xD9 ScriptPlayer placements (map
   01000081, idx 0 and 42) both carry script hash **0x06000000 — a hash no archive on the disc
   carries** (all 99 FILES.BIN archives scanned for a type-7 `06000000` entry [D]) — and trigger
   mask p5 = 0 [D], so `SP_Update`'s trigger = flags & 0 and both idle in state 6 forever
   [M docs/systems/script-player.md]. The 7 scripts are dead data.
3. **No 0xE8 LOAD_LEVEL placement exists** in any map [D], and the mission system is inert:
   `MissionData` (ACTION.ELF 0x2a4350, 24 records) has **no 0x07000041 row** [D], so nothing writes
   ch 99/100 on the wrapper [M Mission_MonitorObjectives 0x1827d8 + Mission_Update 0x182c60's
   state-1 gate; the ch-100 map's default for 41 is 0x07000048, moot without a writer].
4. **The one 0xF4 movie trigger is E-armed but carries movie code 0** [D: params (0, 4, 0) — after
   `Trigger_MoviePlayer` 0x1b2a08's param transform, ext+0x20 = the placement's p0 = 0, the input
   slot ext+0x26 = p2 = 0, mask ext+0x36 = p1 = 4; verified against `Trigger_Create` 0x1b1720
   (`__s[8] = *(u32*)(param_3 + 0x30)` = ext+0x20, `*(u16*)(__s + 0xe) = *(u16*)(param_3 + 0x2c)` =
   the kind, `*(u16*)(__s + 0x26)` = ext+0x26)]. Its update path is inert (slot 0 fails the first
   guard [M 0x1b1f24-28]); the E-arm (`Trigger_Activate` 0x1b2518-55, disasm) only writes
   `GameState+0x10 = 0` and pushes states 6/7. With no movie started, state 6 pops the same tick
   (`psiMovieFinished` 0x1082e8 = `BGFMVPlaying == 0`) and state 7 — the fade state machine
   [M 0x1c9ca8-0x1c9cfc, disasm; the decomp hides it] — counts its 300 frames out and pops. **The
   game's own E-arm on this disc data never sets a load.**
5. **No other carrier exists in ACTION.ELF.** The whole-ELF scan finds 0x07000041 exactly six times:
   the five gameplay special-cases the earlier census knew plus one `.data` row (0x24fe0c) in the
   per-level attribute table alongside 0x07000048/43/46/49/4b/4c — no story-order table, no
   hardcoded 41→1b pair [D]. The `GetLevelToLoadFromMovieCode` table (0x1d3560) never returns
   0x0700001b; its only route to the station is the briefing special case in the load path itself.
6. **The game's way into 0x0700001b is the briefing ride**: any `ResetMap_LevelToLoad(0x0700001b, …,
   no_intro=0)` remaps to the 0x7100018 briefing [M 0x1d3800:95-97], `ResetMap_Load`'s phase 0 starts
   briefing movie 18 (`UsingMovie(0x7100018) = 1` [M 0x107eb8]) and maps the masked
   `0x07000018` slot back to **0x0700001b** [M 0x1d6360:99-102 — `if (uVar7 == 0x7000018)
   { GameState._8_4_ = 0x700001b; }`; there is no 07000018.bin archive on the disc [D FILES.BIN]].
   The story hop itself is the front end's/save chain's (outside ACTION.ELF), so the port's
   in-level simulation never sees one from the wrapper's own data — and neither does the game.

## What changed

- **`crates/nf-game/src/drone.rs`** — the 241-tick `wrapper_end_timer` stand-in (field, init, and
  the `tick_inner` arm) is **removed**. Its premise is refuted twice over: no script runs on the
  wrapper (so "the chain's longest script, 241 ticks" bounds nothing), and no 0x12 exists to port.
  The arm's comment block now carries the decode verdict with the addresses above. The approach
  chain (ch 1, the 0xEA pair's outs) is untouched.
- **`crates/nf-game/src/triggers.rs`** — `resolve_exit_code`'s 0x0710xxxx arm now applies the
  loader's phase-0 special case: the masked 0x07000018 slot resolves to **0x0700001b**
  [M 0x1d6360:99-102]. Before this fix, any port exit targeting 0x0700001b (the story's Equinox
  hop) landed on `0x07000018` — an archive that does not exist. The consumer in `nf-view`
  (`wrap_intro` → `resolve_exit_code`, lib.rs level-exit drain) picks the fix up unchanged.

## Tests

- `crates/nf-game/tests/wrapper_07000041_end_live.rs` (new, NF_ISO opt-in) pins the disc data
  through the port's own readers: no 0xE8; the 0xF4's (0, 4, 0) and `movie_code == 0`; no built
  trigger carries a `next_level`; the 0xEA outs (1, 4)/(0x6e, 8); the 7-script event census
  (0x3 ×6, 0xF ×3 — no 0x12, no 0xd); the two ScriptPlayers' 0x06000000 + mask 0; a headless
  sister that a real 0x12 decodes when present. Sisters are structural: the positive controls are
  the level's own other records (the 0x3/0xF events, the 0xEA outs, 0x06000027's presence) and the
  truck script's 0xD on 07000005.
- `triggers.rs::the_equinox_hop_wraps_to_the_briefing_and_resolves_to_1b` (new, headless): the full
  Equinox hop — `wrap_intro(0x0700001b) = 0x07100018` → `resolve_exit_code = 0x0700001b` — with the
  sister that every other briefing wrapper still masks by identity (only 0x18 remaps).
- The existing `movie_trigger_fires_on_armed_input_and_the_discs_record_cannot` already pins the
  0xF4's two guard shapes against the disc record's params.

## Verification

- `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`
  (the fast runner `tools/test/suite.py`): see "Suite totals" below.

## Remaining / open

- **The briefing movie ride on the level_exit path is view-side.** The sim's exit now resolves to
  the right archive; the observable briefing movie (0x7100018) before the Equinox loads is the
  front end's `movie_pending` flow, which the level_exit drain bypasses (the port's deliberate
  compression, documented at the drain site). Rendering/presentation work belongs to the main
  agent's pass.
- DRIVING.ELF is stripped and out of scope; if the driving side carries the wrapper's story
  hand-off, it would show up as a `ResetMap_LevelToLoad(0x0700001b)` call from the driving ELF —
  its 9,170 decompiled functions hold no 0x7000041/0x700001b literal [D grep], so the hand-off is
  data-driven there. Not ported.

## Live checks for the main agent

1. On a FRESH boot into 07000041 (`goto_level 0x7000041`), watch `GameState+0x10` (0x2a3778) with
   `gdbwatch.py` while walking to the 0xF4 volume at (16.1, 0, -0.4) and pressing E:
   expect a one-write pulse to 0 (the movie code) + the GameFlow stack gaining 6/7, and **no**
   `GameState+8` change (0x2a3770 stays 0x07000041) — the E-arm loads nothing.
2. Watch `switch_channels` (0x26fc90) channels 1 / 0x6e during the same walk: ch 0x6e rises on the
   auto-touch at (15.6, 0, -33.2), ch 1 rises only after E at (20.6, 0, -33.4).
3. `--bond 27.2,1.3,-30.2,0,0` (the Player1 start) and `--bond 20.6,0.5,-33.4,0,0` (at the E arm):
   the first shows the spawn dock, the second the approach chain's staging — visual reference only.

## Suite totals

(filled before the final push)

## Usage

(to be copied verbatim at the end of the session)

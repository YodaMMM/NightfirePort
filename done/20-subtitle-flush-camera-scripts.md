## Your task: `subtitle-flush-camera-scripts`, a new cut-scene clears the subtitles already on screen

`Script_Play` 0x1db588 calls `Text_FlushAllSubtitles` 0x1b15c8 on every camera script start (`+0xAA8 & 0x40`), not only on the three death NISs: the music event is gated, the flush that follows it is not. The port flushes only on the skip arm (drone.rs `subtitle_track.flush`, around line 16705), and the comment near nis.rs:1688 says the flush rides the view side; find out whether any view code does, and correct the comment either way. If nothing does, a line still showing when a camera script starts carries over into the scene. Add the flush at the `Script_Play` site for every camera script. Read 0x1b15c8 and match what it deletes (the type-4/5 messages) and leaves. Check that the start of a script does not eat the first line of that same script: the order of the flush against the script's own first subtitle must follow the game. Pin it through the real path on a level where a subtitle is live as a camera script starts, and pin that a non-camera script leaves it alone. Run the subtitle and cut-scene binaries (grep tests for `subtitle_track`), the nis tests and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/subtitle-flush-camera-scripts`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-05T13:15Z

---
## Report (done, 2026-11-05T15:07Z)


---
## Report from the worker (done, 2026-11-05T15:07Z)

# subtitle-flush-camera-scripts — the report

Branch `s/subtitle-flush-camera-scripts` (cut from origin/master a89818235; merged origin/master
at 3192a42c9+). Task: `Script_Play` 0x1db588 calls `Text_FlushAllSubtitles` 0x1b15c8 on every
camera script start (`+0xAA8 & 0x40`) — the music event is gated, the flush that follows it is
not — while the port flushed only on the skip arm, so a subtitle already on screen when a
cut-scene starts carried over into the scene.

## What the game does (all re-verified against the ELF this session)

**The camera tail.** `Script_Play` 0x1db588 (`work/decomp/ACTION.ELF/001db588_Script_Play__FP10SCRIPTINFOUc.c`,
disassembly of `work/elf/ACTION.ELF` — little-endian MIPS32, segment vaddr 0x107100, file off 0x1000):

- 0x1db5b8 `lhu $v0, 0xaa8($a0)` / 0x1db5bc `andi $v0, $v0, 0x40` / 0x1db5c0 `beqz $v0, 0x1db620`:
  the camera gate. Without the flag the function returns 0 here and **neither** the music event
  **nor** the flush runs [M disasm 0x1db5b8-0x1db5c0].
- 0x1db5c4-0x1db604: the three death-NIS hashes (0x600098a, 0x6000952, 0x6000991 —
  `Script_IsDeathNIS`'s set) build `bVar1`; 0x1db608 `beq $v0, $zero, 0x1db61c` skips only the
  next call when the hash is none of the three.
- 0x1db60c `addiu $a0, 6` / **0x1db610 `jal 0x1de328` (`Music_Event__FUiUi`, symtab) /
  0x1db614 `addiu $a1, 1`**: `Music_Event(6, 1)` — **gated** on the death hashes [M disasm 0x1db608-0x1db614].
- **0x1db618 `jal 0x1b15c8` (`Text_FlushAllSubtitles__Fv`, symtab)**: outside the gate — runs for
  EVERY camera script [M disasm 0x1db618].

**What the flush deletes.** `Text_FlushAllSubtitles` 0x1b15c8 (size 116): `SubTitleMsg = 0`, the
three subtitle sprites `SubTitleSpr/1/2 + 0x38 = 0xff`, then it walks the message list
(`DAT_0031997c`) and calls `Text_DelMsg` for every node whose type byte `+0x1c` satisfies
`type - 4 < 2` — **type 4 and type 5** [M disasm 0x1b1604-0x1b1610, visible in the decompile's
`bVar1 - 4 < 2`]. Types 1/2/3/6 stay. Type 5 is what `Text_AddMsg` 0x1b0c30 promotes a type-4 add
to (the type-4 arm, `param_3 = 5` [M 0x1b0c30]) — so the flushed class is the script op-0x1e
captions plus the id-0xff voice captions.

**The order against the script's own first line.** `SP_LoadScript` 0x1dbc08: every mode arm calls
`Script_Play` first, then reaches `Script_Update` (mode 1/8 arms fall through to the shared tail
`Script_Update`; the mode-64 arm runs play + update + stop, then the same tail) [M 0x1dbc08].
`SP_Update` case 1 (the trigger restart) also goes through `Script_Play` at LAB_001dc174
(`Script_Play(si, mode == 8)`) [M 0x1dbda8 case 1 → 0x1dc174]. So the flush always precedes the
(re)started script's own first `Script_Update`, and a first-line op-0x1e survives its own start.

## What the port did

- `crates/nf-game/src/nis.rs`, `Scripts::load` (the `SP_LoadScript` port): on a camera script
  (`s.flags & 0x40`) it now pushes `Request::TextFlush` for **every** camera script, with the
  death-NIS `Request::Event { kind: 0x41 }` (the `Music_Event(6,1)` port) additionally on the
  three hashes — pushed **before** the mode arms' `s.start(...)`/`s.update(...)`, so the flush is
  drained before the script's own first `Request::Text` [M 0x1db618 + 0x1dbc08].
- `crates/nf-game/src/nis.rs`, `Scripts::update` case-1 restart arm (the `0x1dc174` restart):
  same `Request::TextFlush` for camera scripts.
- The comment at nis.rs ~1685 corrected: the old text claimed "the subtitle flush rides the view
  side; the Sim carries no script subtitles here" — **nothing view-side does it**. The only
  producers of the view's `text_flush` flag were/are the Sim's `Request::TextFlush` arm
  (drone.rs 16806-16810: `script_msgs.clear()` + `text_flush = true` + `subtitle_track.flush`),
  and before this branch the only producer was the skip arm (`Script_FFwd` 0x1dac78:26-30, port
  nis.rs 467). The view (scene.rs 6102-6104) only *consumes* the flag by clearing `hud.subs`.
- `crates/nf-game/src/drone.rs` TextFlush-arm comment updated with the second call site and the
  type-4/5 scope. (Comment-only change.)

The three producers now mirror the game's three `Text_FlushAllSubtitles` call sites: the skip arm
[0x1dac78:26-30], the Script_Play camera tail on the load path [0x1db618], and the same tail on
the case-1 restart [0x1dc174 → 0x1db618].

## The pins (crates/nf-view/tests/subtitles_live.rs, +3)

Harness: no disc pack carries BOTH a camera script and an op-0x1e text script (the harness note
records the probe over the level packs; the flush-under-test needs both in one Sim). The real
camera NIS of 0700001b.bin (disc data) is inserted into the running 07000500.bin Sim's script lib
under its own hash; the spawn, the `SP_LoadScript` load and `Script_Play`'s camera tail are the
Sim's real path. Test 3 flips only the header's camera word on the real 0x06000961 op stream —
that flip is the one `[S] unverified` item, marked in the test; the flag's meaning itself is
[M 0x1d8248 → Script::load].

1. `camera_script_start_flushes_the_live_caption` — the live 0x06000068 caption (track, `hud.subs`
   and pending drains all observed) dies within the tick the camera script starts [M 0x1db618].
2. `non_camera_script_start_leaves_the_live_caption` — a non-camera script's start (no camera
   flag → the tail at 0x1db5c0 is never reached) leaves the live caption alone.
3. `camera_start_does_not_eat_the_script_s_own_first_line` — a camera-flagged script's own first
   op-0x1e line displays: the start flush ran before it [M 0x1dbc08 order].

## What ran

**Touched-code binaries run whole** (rule 10 — grep `subtitle_track|TextFlush` plus the nis
tests; I changed nis.rs's request production and drone.rs's comment; each run directly with
`--test-threads=1`, `NF_ISO` set):

| binary | result |
| --- | --- |
| subtitles_live | ok — **7 passed, 0 failed** (the 3 new pins + the 4 existing) |
| level06_start_subtitles_live | ok — 6 passed, 0 failed |
| fix_script_commands_live | ok — 4 passed, 0 failed |
| mission_outros_live | ok — 3 passed, 0 failed |
| nis_coder_break_live | ok — 3 passed, 0 failed |
| nis_event9_live | ok — 2 passed, 0 failed |
| nis_fade_lifecycle_live | ok — 2 passed, 0 failed |
| nis_page_boot_live | ok — 1 passed, 0 failed |
| nis_footsteps_live | ok — 1 passed, 0 failed |
| nis_handback_census | ok — 1 passed, 0 failed, 1 ignored |
| nis_skip_handback | ok — 2 passed, 0 failed |
| probe1b_nis211 | ok — 0 passed, 0 failed, 1 ignored (skips without its probe flag) |
| rappel_vanish_0700004a_live | ok — 2 passed, 0 failed |
| cross_process_determinism | ok — 3 passed, 0 failed (from the suite log) |
| fps_determinism_live | ok — 4 passed, 0 failed |
| nf-game lib unittests | ok — **675 passed, 0 failed, 5 ignored** |
| mp_scenarios_live | **not finished** — 19 tests, still running at the session's stop-work mark (my direct run showed the first tests passing; the suite's copy had not flushed its section when the session ended) |

**Full suite** (`tools/test/suite.py`, final tree merged with origin/master, 1125 test binaries,
started 14:17 UTC): the run did NOT complete before this session's 15:01 UTC stop-work mark — the
walkreal_*_live stretch was still running at 15:06. No total line, so the suite result is the
partial state at that point:

- `FAILED probe_pylon_render::probe_render_pylon_only`, `FAILED fix_0c_pylon_live::...` ×2 — all
  three panic at the wgpu `adapter: NotFound` request (crates/nf-view/tests/probe_pylon_render.rs:25):
  **no GPU adapter in this sandbox**, an environment failure class, unrelated to this task's
  Sim-side change.
- `CRASH mp_stress_explosives_live (exit -9)` — SIGKILL, the OOM class, also unrelated.
- Every other binary that completed while the run was live was green (no other FAILED/TIMEOUT
  line in `suite.out`).
- The 3 `disc_golden` failures expected on the slim ISO were not reached before the run stopped.

Sandbox notes (not repo issues): the first suite attempts this session died on build/link
problems caused by the sandbox, not the branch — a stale user-prefix `alsa.pc` pointing at /usr
(no dev symlink there) and a 100%-full disk (linker SIGBUS). Fixed the prefix libdir and freed
the disk; no repo file changed for either.

## What's left

- The death-NIS `Music_Event(6,1)` of the same camera tail is not ported on the case-1 restart
  arm (the load arm has it). Pre-existing gap, unrelated to the flush; noted in the code comment.
- A disc pack carrying both a camera script and an op-0x1e script would let pin 3 lose its [S]
  flag flip; none found in the packs probed.

## Live checks for the main agent

- `tools/trace/gdbwatch.py` on 0x1b15c8 (`Text_FlushAllSubtitles` entry, execute): FRESH boot of a
  level with a camera script that is not a death NIS (e.g. 07000005's cut-scene SPs) — expected:
  the function is entered when the camera script starts (any camera script), not only on the three
  death NISs; count ≥ 1 per camera script start.
- `tools/trace/fieldprobe.py` on `SubTitleSpr + 0x38` at the camera script's first frame:
  expected 0xff (the flush hid the live caption sprite) while the caption was non-0xff the tick
  before.

## Usage

not visible to me — this environment shows no token, context-window or cost figures.

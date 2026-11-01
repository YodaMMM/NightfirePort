# level-06-start-subtitles — the driver dialogue subtitles at the 07000006 start

Build: ACTION.ELF `e7ffdcb3…` (work/elf), slim ISO `Nightfire-onfoot.iso`. Tags: [M] decompile /
disassembly (function + address), [D] disc bytes, [L] EE snapshot (address + value), [S] suggested.

## What the PS2 shows and where the lines come from

At the 07000006 (Double Cross, castle grounds) start the game subtitles the gate conversation. The
two quoted lines are **USATxt.dat labels 0x0600006e** "Can't I rest a moment?  I drove six hours to
get here." and **0x0600006f** "Then it's six hours back, old man!  So start unloading." [D
USATxt.dat, dumped through `nf_formats::hud::Lang`]. The full conversation set (labels 0x0600006c–
0x72, always-show, plus the guards' chat 0x73–0x77) is the wine-truck arrival.

**The option default.** The fresh-boot Subtitles byte (`DAT_002a37c8`, GameState+0x60) is **0** in
every level-start snapshot [L ee_start_05.bin / ee_start_07000001.bin / ee_start_0700000a.bin /
ee_snapshot.bin @0x2a37c8 = 0]; `bootup_bootup` 0x1c7e78:34 clears it [M], `Boot_LoadPTPData`
0x1c8560:79 loads it from the save, and the def-codename template's subtitle byte is 0 on the disc
[D ACTION.ELF .data 0x2e1b30, both slots] [L ee_snapshot same]. **The port's default (`Profile::
gset.subtitles = false`) is already the game's** — the task's first check passes unchanged. That
makes the symptom the interesting part: the PS2 shows the lines with the option OFF, which the
subtitle post path explains.

**The post path: `Sound_DoSubtitle` (0x1dde58), not the script op.** The level archive carries no
op-0x1e Text commands (all 11 scripts scanned [D 07000006.bin kind-7]) and no briefing FMV exists
(0x07100006 is not in `psiStartBackgroundMovie`'s UsingMovie whitelist [M 0x107eb8 disasm; no
`07100006.PSS` in MOVIES/30_FPS [D]]). The lines ride the **spoken-sound** path instead:

1. `Snd2Lbl` (0x2d25d8, 484 × {u32 label | 0x80000000 always-show, u32 sound_id}) maps **sfx 0xac →
   0x0600006e** and **sfx 0xae → 0x0600006f**, both rows carrying the always-show bit [D table idx
   78/79]. `Sound_DoSubtitle` scans it and gates `DAT_002a37c8 == 0 && (label & 0x80000000) == 0 →
   return` [M 0x1dde58:21-23] — **the bit bypasses the option**, then adds
   `Text_AddMsg(-1, 0xff, 4, Txt_BindLabel(label), 0, fptoui(g_fSubtitleDurationModification ×
   SFXOutputData[id]+0x10 × 60))` [M 0x1dde58:26-34]; the id-0xff type-4 add passes `Text_AddMsg`
   0x1b0c30's flag gate through its `(type == 4 && id == 0xff)` disjunct [M]. The modifier is 0.92
   [L ee_start_05.bin @0x30d084].
2. The sounds are played by the speakers' **start scripts' anim sound events**: `AnimProcessScriptCmds`
   (0x1c1b38) case 1 plays each anim-script op-1 command as `Sound_Play3D(100.0f, −1, −1, id,
   bone-0 pos, rand pitch, 0, 0)` [M]. The disc data: **driver #494** (skin 0x05000035, waits ch
   0xb9 raised by the truck-park TouchOnces at the spawn) start script **0x0600011d op-1 frame 5 =
   sfx 0xac**; **escort #450** (skin 0x050002e) start script **0x0600011b op-1 frame 295 = sfx
   0xae**; the wall guards #487/#488 (no wait channel) carry 0xdc@1 / 0xe1@250 / 0xdd@408 and
   0xe2@696 in scripts 0x06000124/0x06000126 [D 07000006.bin kind-5 entries + the placement census].
   0x0600011d is the 32-byte kind-5 entry the census already knew as the driver's sitting anim —
   the sound event rides inside it.
3. `Sound_ReqestPlaySfx` (0x1dcc68) sets the node's +0x52 flag = `dist < outer × 0.5` (the custom
   outer is 100.0 from the anim call [M 0x1dd548:33]; the spoken ids are streamed,
   SFXOutputData+0x15 = 1 [D]), and `Sound_UpdateSounds` (0x1dda70) fires `Sound_DoSubtitle(id)` on
   the start when the flag is set [M].

So the PS2's behaviour — subtitles during the boot's searchlight sweep, with the option factory-off
— is the always-show rows posting at the anim sound ticks.

## What the port got

* `nf_formats::sfx::snd2lbl` / `sfx_durations` — the two static tables parsed from ACTION.ELF's
  symbols (`Snd2Lbl` 0x2d25d8, `SFXOutputData`+0x10 per id).
* `SubtitleTrack::push_always` (nf-game/src/subtitle.rs) — the id-0xff `Text_AddMsg` add: the queue
  takes it unconditionally (the FIFO chain and the verbatim duration as in `push`).
* `Drones::sound_subtitle` + the hook on the drone anim-sound drain (nf-game/src/drone.rs) — the
  game's `Sound_UpdateSounds` → `Sound_DoSubtitle` chain at the sound start tick: `Snd2Lbl` scan,
  the +0x52 distance gate (dist < 50.0 for the anim call's custom outer 100.0), DoSubtitle's
  option/always-show gate, the 0.92 × f10 × 60 duration, then the `script_msgs` HUD drain + the
  `subtitle_track` push. `Drones::attach_snd2lbl` carries the tables from the host (nf-view lib.rs
  → `DroneContext` → scene.rs's build).
* No change to the option default (already right) and none to the script-op path's gates.

## Verification (all through the real input path, NF_ISO headless)

`crates/nf-view/tests/level06_start_subtitles_live.rs` — 2 tests, **2 passed**:

* `driver_conversation_posts_with_the_option_off`: a fresh 07000006 load (the flag Off as the
  snapshot default pins) with idle input for 700 ticks queues the whole gate conversation in
  script order through the FIFO [M 0x1b11d8]: the wall guard's opener (0x06000070) at tick ≤ 3 (no
  wait channel, frame-1 event), the driver's line (0x0600006e) after the wake chain and before tick
  300 with duration **211** (= 0.92 × 3.8312931 × 60), the escort's reply (0x0600006f) FIFO-chained
  behind the busy head with duration **183**, the guards' hip-flask exchange after; `active_
  subtitle` returns the labels at the right ticks and the head advances at the timer end; the HUD
  drain took the same adds.
* `the_option_on_leaves_the_always_show_lines_unchanged` (sister): the option ON produces the same
  two lines with the same durations — the spoken adds' gate is `Sound_DoSubtitle`'s own, which the
  always-show bit bypasses.

**A pin correction the evidence forced.** The task's pin said "the lines post … with the option on,
none with it off". The decompile + disc say the opposite for THESE lines: the always-show rows
post **whatever the option is** [M 0x1dde58:21-23 + D Snd2Lbl idx 78/79] — and the task's own [L]
capture shows them with the fresh-boot option OFF. The tests pin the game's rule (both tests above)
instead of the pin's wording; nothing was weakened — the option gate still drops every non-always
spoken line, and the script-op (type-4 id-0) gate is untouched (subtitles_live's off/on pair still
passes: 4 passed).

Runs quoted (all `--release --test-threads=1`):

| binary | result |
|---|---|
| `nf-view --test level06_start_subtitles_live` | 2 passed; 0 failed |
| `nf-view --test subtitles_live` | 4 passed; 0 failed |
| `nf-view --test fix_script_commands_live` | 4 passed; 0 failed |
| `nf-game --lib subtitle` | 6 passed; 0 failed |
| full suite (`tools/test/suite.py`) | see the total line at the end |

## What's left

* The exact wake tick of the driver's start script (the ch-0xb9 plates' fire tick) is port-side
  deterministic but has no [L] measurement; the test pins the range, not a hand value. A live
  PCSX2 watch of the first 400 ticks' `SubTitleMsg` head (0x30cd98's +0x08 text pointer) would pin
  the game's tick.
* The 2-D `Sound_Play` / non-streamed force arms of `Sound_ReqestPlaySfx` (+0x52 = 1 verbatim) and
  the `ScriptCam != 0` force are documented but not wired: every other sound source (barks, script
  sounds, sound triggers, map sounds) still posts no subtitle. The wall guards' chat and the driver
  exchange work because they ride the anim-sound path.
* The dressing scene's own subtitled sound (0x119 → 0x06000079, no always-show bit [D Snd2Lbl idx
  89]) will post once the same hook covers the script `Sound` arm.
* The 3 `disc_golden` failures the slim ISO allows.

## Live checks for the main agent

1. PCSX2 07000006 fresh boot, `gdbwatch` on `SubTitleMsg`+0x18 (the head's time remaining, node
   list 0x31997c) for the first 400 ticks, Subtitles option OFF: EXPECT four type-4 posts —
   0x06000070 then 0x0600006e then 0x06000071 then 0x0600006f — at the ticks the port's track
   carries (1/197/408/523 ± the load-tick skew).
2. `fieldprobe` poke `DAT_002a37c8` (0x2a37c8) = 0 and 1 across two boots: EXPECT no change in
   these four lines (always-show) and no briefing-FMV difference (no 07100006.PSS).

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no billing or
usage panel is reachable from the sandbox). No numbers are estimated.

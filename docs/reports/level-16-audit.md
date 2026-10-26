# level-16-audit — the 07000016 behaviour audit

Branch `s/level-16-audit`. The full per-placement audit table is
`docs/levels/07000016-audit.md` (one row per main-map placement — 360 rows over 26 kinds plus the
7 room refs, the scripts section, the mission-record section and the template/library section).
This report is the short form: what was checked, what it found, what is left.

## What's done

- **Setup reuse:** the sandbox carried the level-15 session's repo/ISO/work; the branch was cut
  from origin/master and pushed with its WORKLOG inside the first minutes. The ALSA user-prefix
  was rebuilt from the cached debs (`/tmp/alsa-root` → `/tmp/alsa/prefix`, the patched `alsa.pc`
  prefix — the same layout as level-15's report).
- **The census [D]:** `census_07000016.rs --ignored` re-run — 707 all-map placements, main map
  m63, **360 rows over 25 kinds + 7 room refs** (20 drones, 10 TouchOnce, 8 doors, 1 switch, 15
  script players, 4 muxes, 17 pickups, 4 spawners, 5 dynamic objects, 12 sound triggers, 7
  destroyables, 2 hurt, 2 Apocalypse, 1 ladder, 2 sky-objs, 2 starts, 4 AI points, 64 cover
  nodes, 14 AI volumes, 21 coronas, 106 kind-0x19 CELs, 31 kind-0x31 cels). A new probe
  `dump_07000016_scripts` was added to the same file (both bins).
- **The creator dispatch re-verified [M]:** every case of
  `parsemap_create_dynamic_objects` 0x1d04f0 re-read from work/decomp/ACTION.ELF this session,
  plus the walker gate `parsemap_block_map_data_dynamic` 0x1d1b68 (`(flags & 0xa000) == 0` —
  which proves the dispatch's case 0x31 → `FuseBox_Create` is UNREACHABLE on this level: all 339
  kind-0x31 placements carry 0x2031 and ride the static path as level props, not fuse boxes).
- **The audit table:** `docs/levels/07000016-audit.md` — one row per placement (positions and
  params verbatim from the census), one kind-header per kind carrying game [M] chain / port
  status / kind-level GAP. Per-row notes carry the wiring (out channels, lock channels, wake and
  death channels, script hashes, the no-writer marks).
- **Scripts [D]:** 07000016.bin has 734 entries and **14 kind-7 scripts**; 07800016.bin has 5
  entries and **1** (06000631). Five are the shared blast/debris family (0600004f/52/6f/27/7c4);
  the level's own set: the dust puffs (06000834/83a), the door-graphic anim (06000999), the
  exhaust streams (0600090c), the gantry light (060008bc), the button anim (060008ba), the spawn
  sounds (060009f6), the bay lift (060009e9) and the ShuttleLaunch cinema (0600090a). **The
  level's only camera op is 06000631's stream 0** (`Camera { spline: false, first: 0, last: 114 }`
  + `CameraEnd`) — the pack cinema; events 04/05/06/07 are the Player_Disable /
  Drone_EnableAll freeze pair [M Script_EventHandler 0x1d94c0], modelled in nis.rs. The port's
  camera-op arm (op 0x0d = `Script_CameraStart` 0x1d9140) and scene.rs `script_camera()` are the
  modelled chain — the picture itself is a view-side check (render list below).
- **Mission record [L]:** record **22** of the on-foot MissionData table read byte-for-byte from
  work/ee_start_05.bin (@0x2a4350 + 22·40 = 0x2a46c0; objtable @0x2a42f0, 2×24 bytes): level
  07000016, mission 07000014, part 2, objectives (0x0400004e, fail −1, hint 0x04000055, ch 0x8f,
  enable 0x8e) and (0x0400004f, fail −1, hint 0x04000056, ch 0x90, enable 0x8f), +0x20 = 0x5604,
  +0x24 = 0. The port's `crates/nf-game/src/mission/c07000016.rs::DEF` matches field-for-field.
  **The level doc's §2 said "[L] record 16" — record 16 is 07000012; corrected to 22.**
- **New test:** `worldobj_live::dynobj_07000016_display_pairs_swap_on_their_lock_channels` —
  loads 07000016 through the real path and pins the five kind-0x3b placements: the create states
  (watch-A hidden with 0x30 set [M 0x18dbf8:31 + object.md +0xF0's bit 0x10 = not drawn], the
  watch-B partners drawn) and the channel swaps (0x24 → the unlocked screen restored/locked
  deleted; 0x94 the tower pair; 0xcc → the lift-shaft blockers) [M 0x18d3b8].
- **Doc corrections found by the audit** (applied to docs/levels/07000016.md): §2's record
  16 → 22; §7's kind-0x3b row ("[S] the 0x3b class's handlers are not enumerated" — they are:
  worldobj.rs DynObject on the 0x1d04f0 case-0x3b chain, now live-tested); §8's U-051 — the
  data-side gate writer is settled: **0600090a's Event-0x0d writes ch 243 = 1 then 0, TWICE**
  [D script dump — the two launch-blast windows]; the bay lift's ch-0x50 writer is the same
  script's `[80, 1]` event (and the lift's own 060009e9 writes `[80, 0]` at its end).

## The gaps (each documented in the audit doc; the big ones below)

- **GAP-16-1 — the kind-0x3b per-tick draw swap (new, level-generic):** the Sim side is complete
  and now test-pinned (worldobj.rs computes the game's exact flags/dead/alpha [M 0x18dbf8/
  0x18d3b8]), and the scene's build-time pass already honours the create state (scene.rs:8093
  skips p0 != 0 rows; the fix-striped-boxes2 chain covers the glist-0x10 rows). What is missing
  is the **per-tick half**: the scene's instance list is built once, so a channel rise never
  re-syncs the drawn set — on 07000016 the SecurityStation's ch-0x24 edge leaves the unlocked
  screen undiscovered and the locked screen drawn forever, and ch 0xcc never reveals the
  lift-shaft blockers. **Why not fixed here:** the fix is a per-tick scene hook re-syncing the
  five instances from the DynObject state — view-side work that rule 6 reserves for the main
  agent's render verification. Suggested spots: scene.rs's MISC_KINDS pass builds the instances
  (the placement→DynObject link is the create order in `WorldObjects::create`); the sync hook
  belongs where the scene already per-tick updates Sim-driven visuals.
- **GAP-15-1 — the blast debris particles (inherited, level-generic):** `Explode_Update`'s
  Debris_CreateEx arm stays unported (level-15-audit's report holds the full [M] chain). On
  07000016 it affects the grenades and any scripted blast; the scaffolding (create_debris, the
  blast scripts' Entity streams) is in place.
- **GAP-15-2 — "characters walking through Bond" (inherited, shared):** the drone→player bump
  (msg 0x22, Drone_CollisionHandler) is unported (drone.rs; first documented by level-13-audit
  GAP-13-1). 07000016's 20 room guards + the Kiko chase inherit it.
- **The task book's other symptoms have no 07000016 data-side gap:** no civilians/guests exist on
  the level (nothing to talk — the dtypes are 2/8/0xf/12 [D], none reaches CivilianMission's
  voice branch); the doors DO get their lock channels from the mask-0x04 panel triggers and the
  SecurityStation (live, walkreal/interactables-pinned); the "console" actions are the same
  mask-4 triggers (the only consoles are the door panels — live); the one camera script is
  modelled (the render check below).

## Render list for the main agent (the visual checks)

1. `--bond 2.5,1.0,-118` facing south (the corridor, door[0] locked): the **Screen_door_locked**
   graphic should show at the door lane; **Screen_door_unlocked** must NOT show (the port's
   build-time pass gets this right today — confirm).
2. The same spot after driving ch 0x24 (E at the SecurityStation route): the **unlocked** screen
   should appear and the **locked** one vanish — expected FAIL today (GAP-16-1's per-tick half).
3. `--bond 2.5,1.0,-160` (the bay lift, before the launch): the lift-shaft blockers must NOT
   show; after ch 0xcc they should appear (GAP-16-1 again).
4. The launch cinema (drive ch 0x81): 06000631's camera ride (keys 0..114) should move the view —
   the port's camera chain is modelled end-to-end but never render-verified on this level.

## Live checks for the main agent

1. **U-051 (sharpened, data-side now settled):** watch `switch_channels[243]` (0x26fc90+243)
   through the launch cinema (fresh boot): expect 1 → 0 → 1 → 0 at the two windows where
   0600090a writes it [D script dump], and record Bond's position each rise.
2. Watch `switch_channels[141]` (0x26fd1d) through the mid-corridor touch (#616) — the
   kiko-button gate question (docs/systems/kiko-death-button-gate.md).
3. The Apocalypse channels 0xf4/0xf5 (0x26fcF4/F5): any writer through the launch cinema?
   (The census + script dump found none — the two Apocalypse placements would be dead data.)

## How it was verified

- The census + scripts dumps: `cargo test --offline --release -p nf-view --test census_07000016
  -- --ignored --nocapture` — green (the [D] evidence base; 07000016.bin 734 entries / 14
  scripts, 07800016.bin 5 / 1).
- The new test: `cargo test --offline --release -p nf-view --test worldobj_live --test
  census_07000016` — **worldobj_live 6 passed / 0 failed** (the five pre-existing + the new
  07000016 swap test), census 3 ignored (the assertion-free dumps, run green with --ignored
  above).
- **The per-crate runs** (the sandbox kills background processes between tool calls, and the
  first full nf-view relink hit a 100%-full disk — the linker died on `collect2: ld terminated
  with signal 7`; after clearing the stale executables/dup rlibs the per-crate substitute ran):
  - `nf-game --lib`: **553 passed / 0 failed / 3 ignored**.
  - `nf-game --tests`: all binaries green (25 result lines shown, every one `ok`; 0 failures).
  - `nf-formats`: lib green (89+1+1+1+2 across its binaries) and `disc_golden` **11 passed /
    3 failed — exactly the 3 allowed slim-ISO `disc_golden` failures** (the driving-disc BIGF
    checks: bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest, +1).
  - `nf-audio`: green (23+1+3+1 across its binaries).
  - `nf-view`: worldobj_live + census_07000016 green (above); the remaining ~70 nf-view test
    binaries were NOT re-linked this session (the full relink does not fit the sandbox disk
    alongside the repo/ISO/work — the same constraint level-15 reported; those binaries were
    green on master's last full run and my diff touches only two nf-view test files + docs).
- Every [M] claim in the audit doc cites the decompile file/address re-read this session; every
  port claim cites file:line at this tree; every [D] claim cites the census, the archive bytes or
  the script dump; the [L] record cites the snapshot address. [S] items are marked and excluded
  from pass/fail.

## What's left / for the next message

1. GAP-16-1's per-tick draw half as its own (view-side) task: the scene hook + the main agent's
   render of the four spots above.
2. GAP-15-1 (the debris arm) and GAP-15-2 (the through-Bond bump) — the standing shared tasks.
3. The 06000631 camera ride's render verification (the launch cinema picture).

## Usage

not visible to me (no token/credit/context figures are shown in this environment).

# level-15-audit — the 07000015 behaviour audit

Branch `s/level-15-audit`. The full per-placement audit table is
`docs/levels/07000015-audit.md` (one row per main-map placement — 638 rows over 23 kinds plus the
59 room refs, the template/library section, the scripts section and the mission-record section).
This report is the short form: what was checked, what it found, what is left.

## What's done

- **Setup from a reset sandbox:** fresh clone (master b604242f), branch + WORKLOG pushed first,
  `work-slim.7z` + the on-foot ISO downloaded and verified (369823203 bytes before extraction),
  nf-queue worktree, rustup stable 1.99.1, ALSA headers extracted from `libasound2-dev` into
  `/tmp/alsa-root` with a patched `alsa.pc` prefix (pkg-config suppresses system-path `-L` flags,
  so the unpatched prefix could not link `-lasound`).
- **The census [D]:** `census_07000015.rs --ignored` re-run — 968 all-map placements, **638 on the
  main map m54**, 37 kinds; every main-map interactive kind of the level doc's §7 re-confirmed with
  per-placement params. A new probe `dump_07000015_scripts` was added to the same file.
- **The creator dispatch re-verified [M]:** every case of
  `parsemap_create_dynamic_objects` 0x1d04f0 re-read from work/decomp/ACTION.ELF this session; the
  audit doc's kind headers name each kind's creator, update and use/damage handlers, and the
  port's module for it.
- **The audit table:** `docs/levels/07000015-audit.md` — one row per placement (positions and
  params verbatim from the census), one kind-header per kind carrying game [M] chain / port status
  / kind-level GAP. Per-row notes carry the wiring (out channels, wake channels, lock channels,
  sound ids, pickup ids, the 0xc8/0x42 no-writer marks, the dead-ladder admission).
- **Scripts [D]:** 07000015.bin has 665 entries and **5 kind-7 scripts** (0600004f/52/6f/27/7c4) —
  all five are the shared blast/debris effect scripts the whole disc reuses (07000004-audit lists
  the same family); they are Entity-spawn walks on Wait/Marker keys with SoundEnd, **no camera ops,
  no Anim ops, no hand-backs**. The level has no cut-scene scripts — the broken-cut-scene-camera
  symptom has no data-side cause on 07000015.
- **Mission records [L]:** record 21 of the on-foot MissionData table read byte-for-byte from
  work/ee_start_05.bin (@0x2a4350 + 21·40 = 0x2a4698; objectives @0x2a42d8): level 07000015,
  mission 07000014, part 1, one objective (text 0x0400004d, fail −1, hint 0x04000054, complete ch
  0x8e, init 0, enable 0x4b, flags 0), +0x20 = 0x5604, +0x24 = 0. The port's
  `crates/nf-game/src/mission/c07000015.rs::DEF` matches field-for-field. No fail labels; the
  global 0x60/0x61 hooks apply [M 0x1827d8:128-141] and no placement on this level writes either
  [D channel map] — no civilians exist here [D census].
- **Correction merged:** docs/levels/07000015.md §6 ("Not ported") carried a stale row claiming
  the keypad unlock's channel write is not ported — lock.rs has the burn → out-channel → door-open
  chain (fix-party-mission) and §7's 0x2e row already says "live". The row is replaced with the
  correction note.
- **Doc-level gap fixes:** the audit doc corrects two doc-side inconsistencies it found: the
  "console placement map54 #145" of the walked route is `m54 #721 kind 0x19 "Boxes for CoverBig5"`
  — a static mesh, not a usable console (the only consoles on this level are the three keypads,
  which are live); and the m54 kind counts supersede the level doc's all-map counts.

## The gaps (each documented in the audit doc; the big ones below)

- **GAP-15-1 — the blast debris particles (Explode_Update's Debris_CreateEx arm):** the game's
  explosion objects tick `Explode_Update` 0x17c350, which — while the blast script plays, only for
  script-0x0600052 blasts, inside the first quarter of the lifetime — draws `Rand_Rand(5)` and on
  the `GameState_time % (r+10) == 0` gate spawns metal debris (`Debris_CreateEx` 0x1ac808, the
  MetalHash_132 array) for debris kinds 5/6, and stone debris for 8/10/0xB only on 0x0700008 with
  ch 0x53 on [M 0x17c350:31-67, the decompile re-read this session]. The port has `create_debris`
  (explode.rs:113 — `Debris_CreateEx` 0x1ac808 ported with the exact RNG draws) **but no production
  caller**: the port's explosion is a per-frame queue entry (drone.rs:11321/13098) with no
  per-tick state, so no debris spawns. Affects this level: the 9 mines' blasts, the 8 exploding
  barrels, every grenade/frag blast. **Why not fixed here:** wiring the arm needs a persistent
  per-explosion state (age/kind/script) ticked at the game's object-update position, and the
  `Rand_Rand(5)` draw must land in the frame's RNG stream at the game's order — the ledger's
  `Rand_Rand` recording (`tests/calls/Rand_Rand.calls`, tools/trace/gdbcall.py) is the way to pin
  the order before any test may assert it. That is a task of its own, not a 40-minute tail-end
  fix; the scaffolding (create_debris, the blast scripts' Entity streams via spawn_once_scaled) is
  already in place.
- **GAP-15-2 — "characters walking through Bond" (shared, not level-specific):** the game's
  drone→player bump (msg 0x22, Drone_CollisionHandler) is unported (drone.rs:16544-16551); the
  port's only actor push is Bond-side and airborne-only (player.rs:1796-1840, marked [S] port
  choice). First documented by level-13-audit GAP-13-1; 07000015's 20 room guards inherit it.
  Large gap → its own task (needs the msg-0x22 arm's address + behaviour).
- **The task book's other symptoms have no 07000015 data-side gap:** no guests/civilians exist on
  the level (nothing to talk); the doors that never lock are the six p3=0xc8 doors — ch 0xc8 has
  no writer on the level [D], game and port agree (pinned by the interactables tests); the
  "console" of the walked route is a static mesh (#721); the keypads' console-burn interaction is
  ported and live; the level's scripts carry no cameras.
- **Draw-only note:** the blast scripts' debris glists (0200021a/21d/21e/42d, 020003d7..e8,
  02000499/49a, 020007c9/ca) draw through the port's script player; the mine's exploded beam stays
  solid in the port's static collision world [S-note, pre-existing, documented in the audit doc's
  0xfe header].

## How it was verified

- `cargo test --offline --release -p nf-view --test census_07000015 -- --ignored` — both dumps
  (placements + the new scripts dump) green — the [D] evidence base.
- The touched crate's tests (nf-view): 171 passed / 0 failed / 2 ignored before the sandbox's
  10-minute tool-call limit killed the run mid-binary (the campaign_flow live binary); the census
  binary itself is the file this task touched and it ran green above. nf-game: 896 passed / 0
  failed / 6 ignored. nf-formats: 105 passed / 3 failed — exactly the 3 allowed `disc_golden`
  failures on the slim ISO (the driving-disc `/DRIVING/MIS01.MUS` BIGF checks). nf-audio: 28
  passed / 0 failed. **The single-run four-crate `suite.py` total could not be produced in this
  sandbox: background processes die between tool calls here (two suite starts, one under setsid,
  were killed mid-build), and the foreground per-crate runs above are the reported substitute.**
- Every [M] claim in the audit doc cites the decompile file/address re-read this session; every
  port claim cites file:line at this tree; every [D] claim cites the census or the archive bytes;
  the [L] record cites the snapshot address.

## What's left / for the next message

1. GAP-15-1 (the debris arm) as its own task: the Rand_Rand trace pin, the per-explosion state,
   the tick insertion at the game's update position, the test on a barrel blast.
2. GAP-15-2 (the through-Bond bump) — the shared task from level-13-audit, still open.
3. The nf-view remainder after campaign_flow (the live binaries) — nothing failed; just unfinished
   under the sandbox's tool-call limit.

## Live checks for the main agent

1. **The debris gate (GAP-15-1):** record `Rand_Rand` calls during a barrel blast on 07000015
   (tools/trace/gdbcall.py `Rand_Rand`, level loaded fresh): expect one call with arg 5 per
   exploding-blast tick while `Explode_Update`'s script plays (0x17c350), interleaved with the
   object-update order — the recording tells whether the port can slot its `Rand_Rand(5)` draw at
   the game's stream position.
2. **Explode_Update reachability on this level:** gdbwatch on the `Debris_CreateEx` entry
   0x1ac808 (execute access) while blowing an oil barrel (#757-764, the silo rooms): expect the
   call hit only while the blast's age < lifetime/4 and the hash is 0x6000052 [M 0x17c350:31-47],
   with gfx = MetalHash_132.
3. **Optional:** a `--bond` render at (12, 40.5, 3) facing east (the Start_door #825 corridor,
   p3=0xc8) to confirm the door never opens in the port exactly as the game's dead channel keeps
   it shut — visual parity for the no-writer pin.

## Usage

not visible to me (this environment exposes no token, context-window or cost figures to the
worker).

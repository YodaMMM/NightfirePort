# level-13-audit — the 07000013 behaviour audit: every placed object and character against the game

Branch `s/level-13-audit` off origin/master (b604242f). The deliverable is
**docs/levels/07000013-audit.md**: one row per main-map placement (173 rows — 12 drones 0x0f, 48
statics 0x19, 33 glass 0x20, 1 MP start 0x24, 4 sky 0x2a, 2 param-less fusebox art 0x31
(SCRIPT_SHUTTER_OPEN/CLOSE), 2 hints 0x33, 2 zone cels 0x47, 1 rotor 0x71, 2 ScriptPlayers 0xd9,
2 doors 0xdb, 1 SimpleScript 0xe2, 19+18 cover nodes 0xe5/0xe6, 1 load-level trigger 0xe8, 6 AI
points 0xe9, 8 TouchOnces 0xea, 7 pickups 0xf0, 2 spawners 0xf1, 2 sound triggers 0xf9), each with
the game's chain [M], the port's, and a GAP verdict — plus the level's scripts (every stream's ops
for the two main-map SPs' scripts; a dump of all 24 kind-7 scripts across 07000013/07800013/
07900013/07000500) and the mission records.

## What's done

1. **The [D] evidence base, machine-pinned.** `census_07000013.rs` (master's file, extended with
   `dump_07000013_scripts`) dumps all 439 placements by kind and every kind-7 script; the new
   test `data_web_07000013.rs::the_07000013_main_map_data_web_matches_the_disc` pins the whole
   main-map census (per-kind counts), the writer/reader channel web (TouchOnce out chs
   0x28/0x4a/0x50/0x56/0x64; the elevator SP's out ch 0x23 read by SS Shutters2 + hint #424; the
   alarm chain drone #326 p6=0x57 → sound triggers #410/#411 sound 0x289; the spawner start chs
   0x29/0x2a and the ch-0x2a waiter drone; the exit's level 0x07300013 + gate 0x4e; the door lock
   chs 0xc8/0x0; the glass filters 0x30/0x0) — GREEN.
2. **Per-kind game chains re-verified against the decompile** (work/decomp/ACTION.ELF), with the
   dispatch case lines in `parsemap_create_dynamic_objects` 0x1d04f0 and the `control_funcs`
   (0x26f890) rows: **Break_Create 0x12c230** (class 0x20; p2 is the INVERTED bullet-hit filter
   via Collide_FilterBullets 0x1ece30 — NOT a channel; Break_Kill 0x12b970 sounds 0x2e/0x5f2/0x5f3;
   no E case), **Trigger_LoadLevelCreate 0x1b2950** (type 10; Trigger_Activate 0x1b2318 case 10 =
   Music_Event(3,1) + GS_PauseGame(1) + ResetMap_LevelToLoad(0x7300013,1,0) + GameFlow_PushState;
   E-reachable; never self-destructs), **Drone_AIPoint 0x137550** (no control object; the AIPoints
   array; readers NDrone2_ClosestAIPoint 0x151b18 / FindRunToPoint 0x153168 / FindAlarmPoint
   0x153008), **FuseBox_Create 0x1b0448 + SP_Create 0x1dc7e0** (the SP refuses p0==0 [M 0x1db9a8
   L18] — both 0x31s are art records in BOTH implementations), **Script_EventHandler 0x1d94c0**
   event kinds 3 (spawn-transform bake), 4 (Player_Disable), 5/7 (drone freeze/free), 6 (the
   return transform), 0xb (weapon byte), 0xc (music note).
3. **Port-side sweep with current line numbers** — several of the 0700000d audit's anchors had
   drifted; every citation in the audit doc is re-checked at this tree. The port turned out to
   cover MORE than the 0d audit recorded: kind 0x20 glass is fully ported (breakable.rs:208 +
   drone.rs:13452-13535 + shatter + draw), kind 0xe9 AI points are created and consumed by the
   alarm/run-to AI, the 0x71 rotor exists as an object (worldobj.rs:222-244).
4. **The doc fix:** docs/levels/07000013.md §5's stale "0x20 glass open ch 0x30" line corrected to
   the [M] filter reading (the deep section was already fixed on master; the §5 line was not).
5. **The channel web conclusion** (in the audit doc): ch 0x56 has two writers and NO reader in any
   of the 42 maps (the data's own dead end); ch 0x23's only writer is the elevator SP's completion
   (E-only chain, docs/systems/elevator-button.md is the 07000009 prior art); ch 0x4e (the exit
   gate) has no writer — the exit is E-only; the stairwell door's lock ch 0xc8 has no writer —
   locked forever by the game's own data. All of these match the game by construction, so no port
   change was needed for them.

## The gaps (each documented in the audit doc + "Live checks for the main agent")

- **GAP-13-1 "characters walking through Bond":** the game's drone→player bump (msg 0x22,
  Drone_CollisionHandler) is unported (drone.rs:16544-16551); the port's only actor push is
  Bond-side and airborne-only (player.rs:1796-1840, `!self.grounded` :1821, marked [S] port
  choice). Large gap → its own task (needs the msg-0x22 arm's address + behaviour).
- **GAP-13-2 the hologram never spins (visual):** Rotor ticks (worldobj.rs) but the accumulated
  spin angle is never applied to the draw (no `rotors` reference in scene.rs; MISC_KINDS :8062
  draws the rest pose). Small; needs the main agent's render to verify.
- **GAP-13-3 script events 3/4 unhandled:** Event 3 (the spawn-transform bake, 0x1d94c0:98-116)
  and Event 4 (Player_Disable :118-121) fall to the port's generic-unused arm (nis.rs:1002,
  drone.rs:14727). On 13 the intro 060006d7 fires Event 4 at key 153 and the shutter script
  06000959 fires Event 3 twice — the Entity ops then spawn from the cel transform instead of the
  baked one [S]. The elevator-button mechanism itself (mode-0x40 preload → t=0 Entity → E target)
  IS ported and proven on 07000009.
- **GAP-13-4 the 0x31 shutter art:** param-less fuseboxes draw nothing from load in the port
  (fusebox.rs:106's p0 gate + no cel flag); the game side is [S] — live check 1 (a `--bond` spot
  at the security-room shutter wall) settles whether the retail shows them.
- **GAP-13-5 "guests that never talk":** crosshair bond_talk is ported (drone.rs:4108/:4225) and
  idle/patrol barks are voice-type-3-gated (drone.rs:3909); the skins on 13 (0x5000004/06/08)
  need their voice-type mapping confirmed (live check 3). No civilian E arm exists (the game's
  Player_Activate has none either [M 0x1a5170:126-175]).

## How it was verified

- `cargo test --offline --release -p nf-view --test census_07000013 -- --ignored` — both dumps
  (placements + scripts) — the [D] evidence base (green).
- `cargo test --offline --release -p nf-view --test data_web_07000013` — the new pin — GREEN.
- Every [M] claim in the audit doc cites file+line in work/decomp/ACTION.ELF (re-read this
  session, not from memory); every port claim cites file:line at this tree.

## What's left / for the next message

- The full `tools/test/suite.py` run on the final tree: NOT completed here. This sandbox kills
  background jobs between tool calls, and the suite's build phase alone exceeds the ~10-minute
  foreground window, so no total line was produced. The suite's per-binary isolation would also
  hit the finding below.
- **walk_07000013_live OOMs on this sandbox** (4 GB): the binary reaches 3.5 GB anon RSS and the
  kernel global-OOM kills it (dmesg: `oom-kill ... task=walk_07000013_l anon-rss:3510652kB
  total-vm:4347676kB`), identically under the direct shape and the suite's NF_SILENT=1 shape, and
  after a clean relink — on a tree where this task changed NO sim code. Either the sandbox's 4 GB
  is now below the test's footprint or a recent master change grew it; worth a memory profile
  before the next full-suite window. My own additions (data_web, census) are parser-level and
  light.
- GAP-13-1..5 as future tasks; the live checks listed in the audit doc.

## Usage

not visible to me — this session exposes no token, credit or context-window figures (no usage
readout is available to the agent in this environment; nothing to copy).

# level-05-audit — the 07000005 behaviour audit: report

## What's done

- **The full placement audit is written**: `docs/levels/07000005-audit.md` — one row per placement
  for ALL 363 main-map placements (32 kinds, re-dumped from the disc by the new census probe
  `crates/nf-view/tests/census_07000005.rs --ignored`; the counts match the level doc's §7 census
  exactly), each with the game's [M/D] chain (creator in `parsemap_create_dynamic_objects` 0x1d04f0
  or the static path 0x1d1998, its update, its use/damage/event handlers), the port side, and a GAP
  column. Kinds absent from 05 are listed with the dispatcher check, so nothing is silently skipped.
- **The scripts section** decodes the level's live NIS set from the disc (`dump_sp_script` [D]):
  0x06000A22 (the truck ride-in — stream 9's event 0xD [115,1] writes ch 0x73), 0x060008C8 (the
  gatehouse hand-over — camera 0..68, event 6 [0] hand-back, event 4 disable), 0x06000952 (the plunge
  death NIS — event 0xD [98,1] writes ch 0x62), 0x06000994 (the ledge NIS — camera 0..262, two actor
  anims with the weapon-byte event 0xB [22], event 6 [2] hand-back), 0x06000A23 (the EndLevel —
  event 0x11 [152,152,29,1], the CONDITIONAL write of ch 0x1D), and the two copter flight scripts
  (entity groups via event 0xF). Camera op/hand-back notes cover the camera streams and hand-backs.
- **The mission record** is verified against the snapshot and the port
  (`mission/c07000005.rs` = [L ee_start_05.bin @0x2a3e28 record 0]): one objective "Breach the castle
  walls", complete ch 0x46 (no writer on 05 — the writer is 07000006's OR #498, cross-level),
  reveal ch 0x45 (= TouchOnce #36 [D]); the fail hooks (0x60/0x61/0x63 -> ch 99, ch 0x62 death) with
  05's real writers named (touch #315 and the ledge chain write the alarm ch 164; the plunge chain
  writes 0x62).
- **Corrections** (seven, in the audit doc; the level doc itself untouched per the shared-file rule):
  1. SP #71 is doubly dead — script 0x06000074 is absent from the archive [D the kind-7 hash list]
     and its watch ch 125 is blocked by the game's 0x7D gate [M 0x1b1a78:80-82].
  2. SoundTrigger's STOP channel is param 4, not param 1 [M 0x1b2628 rec+2=+0x3C; 0x1b2790's stop
     arm]; p1 = the end-latch byte, p5 = the global-voice slot, p6 = the difficulty gate, p7 = keep_on.
  3. Pickup p4 is the take-sound id (319; 0xffff = silent on the armour rows), not "the model"
     [M 0x18dda0:123-128].
  4. Searchlight p1 is the ON/track channel (heads #27/#261 track while the alarm/ch-97 channels are
     up), not "the alarm channel"; alarm-out p3 = 0 on all seven heads [D].
  5. TouchOnce/Touch params p2..p8 are inert on types 1/2 [M 0x1b1a78 case 1/2] — #269's p6=51 does
     not gate the heli plate; p10 = the disable-gate channel (no 05 row carries it).
  6. The EndLevel event 0x11 writes ch 0x1D only while ch 152 is off [M 0x1d94c0 case 0x11; D params].
  7. The valley test's header contradicted the who law — fixed (below).
- **Small-gap fix**: `interact_07000005_live.rs`'s valley hurt test header rewritten to cite the who
  law [M 0x17e5d0:51-104] — the ten who-1 Hurt planes are configured-inert on the disc and the
  asserted kill is the game's plunge chain (ch 0xBE -> SP #284 -> NIS 0x06000952 -> event 0xD
  [98,1] -> ch 0x62). No assertion was weakened; the file runs 11/11 green.
- **Large gaps documented for tasking** (each with its [M] chain in the audit doc): the west-ledge
  walk-to-grab for wire #251 (pre-existing open item), the D-004 gold-medal gate for the 007 tokens
  (#279/#326), and the sky +0x4a word (six rows carry p4=1; the renderer consumer unchased, [S]).

## How it was verified

- The census re-dump from `NF_ISO` (363 placements, kind histogram == the merged §7 census).
- The decompile reads cited per row: 0x1b2628/0x1b2790 (sound triggers), 0x1b1720/0x1b1a78/0x1b2b88
  (triggers + the 0x7D gate), 0x17e5d0 (the hurt who law), 0x17d340 (copter), 0x1966c0 (ladder),
  0x1981d8 (wire), 0x1db9a8 (SP record map), 0x1b0448 (fusebox), plus the merged 06/08 per-kind
  research for the level-independent creators.
- `dump_sp_script` over the eight live hashes + the kind-7 hash list (the #71 absence).
- The touched test file: `cargo test --offline --release -p nf-view --test interact_07000005_live`
  -> 11 passed / 0 failed.

## What ran (suite)

The sandbox kills background jobs, so the fast runner ran as repeated `chunk_suite.py` chunks
(build 360 s; 809 test binaries; per-binary limit 200 s in the chunks):

- Recorded 263+ binaries, cumulative **902+ passed, 44+ ignored** when the window closed; 546
  binaries had not been reached.
- Failures seen are the documented pre-existing/environmental set only: `fix_0c_pylon_live` (2) and
  `probe_pylon_render` (1) — the GPU render probes that fail headless (documented on the earlier
  runs of the same tree); `campaign_flow_live` — timed out under my 200 s chunk limit (it needs the
  standard 600 s; not a code failure; it passed in earlier sessions); `net_soak_faults_live` /
  `net_resync_live` / `mp_shot_effects_live` — exit −9, the documented OOM class.
- My changes touch one test comment + docs only; the touched crate's file passes as above. The full
  single-pass suite line is the next session's first item.

## What's left

- The full suite pass on the final tree (the chunked run ran out of window; nothing points at this
  task's changes).
- The three large gaps above as their own tasks.
- Live checks for the main agent (in the audit doc): the SC3 copter lift-off/arrival screenshots,
  the gatehouse hand-back pose, the ledge NIS play + the OR #270 alarm/siren beat, and the alarm
  searchlight track switch.

## Usage

not visible to me

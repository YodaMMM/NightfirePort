# s/g019-sky-3: the sky/thunder work (G-019) and its walk re-tunes on the rewritten history

## Result: nothing left to merge
s/g019-sky-2-walks (a57d60262, which carries s/g019-sky-2's sky and voice-line commits rebased) was
ported onto the new history as s/g019-sky-2-walks-p and merged with origin/master (7562f1e70).
Master already holds all of it: the stream batch 1121b commit (c877244af) joined
"s/g019-sky-2 + the re-rolls of s/g019-sky-2-walks (a57d60262)" (docs/reports/s-stream-batch-1121b.md),
and later master commits re-tuned the same walks again.

The merge conflicts (eleven walkreal files, two replays, docs/GAPS.md, common/tower2.rs) were all
older copies of lines master has since replaced. Each took master's side. The auto-merge left three
leftovers, removed in the second commit:
- walkreal_07000015: the branch's `#[ignore]` on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`
  came back, but master un-ignored that walk with two route changes (s/walks-1121-15-1b). Dropped.
- docs/systems/drone-ai.md: the "State coverage" section appeared twice. Dropped the copy.
- common/tower2.rs: a comment about the per-thread flags; master documents this in the 13 walk. Dropped.

The branch tree now equals origin/master byte for byte (`git diff origin/master` is empty).

## Draws
None changed against master. The sky draw changes from G-019 were already on master through
c877244af: the sky slot table takes the Cloud_Create slots (param 4) and the shared-slot overwrite
with the placement transform; the sky loop and the storm timers/thunder run only while a visible
cel has no no-sky bit; 0x0700001B rewrites the sun vector each frame.

## Tests (NF_SILENT=1, target-o, -j 3 build, 6 binaries at a time)
Every walkreal_* binary (109), plus load_scene_equality, load_scene_digest, replay_parity_07000004/0a/0b,
sky_view_gate_live, sky_flash_swap, sky_gs_live and sky_spin_1b_live: 118 binaries, 298 passed,
0 failed, 31 ignored. No red, so nothing was re-tuned. The 31 ignores are master's.

Per walk (passed/ignored): 01 7/1, 02 5/0, 03 7/3, 04 10/0, 05 17/0, 06 8/1, 07 6/0, 08 6/1, 09 9/0,
0a 15/0, 0b 8/0, 0c 6/0 (+ barrelrow, eastroute, ventexit 1/0 each), 0d 2/2, 11 4/0 (+ route 1/0),
12 4/0, 13 4/1, 14 9/1, 15 9/0, 16 18/1, 1b 17/2, 41 15/0 (+ wall 2/0), 4a 4/0 (+ satchel 2/0).
The probe binaries all pass or are ignored.

## Unproven
Nothing new: this run is a check of master's tree, since the branch equals master.

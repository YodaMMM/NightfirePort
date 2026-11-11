# guard-leaf-press-07 — the leaf-press law grown to thirteen levels; the door side pinned

## What is done

**Setup, fresh sandbox.** The branch `s/guard-leaf-press-07` was created from
`origin/s/guard-leaf-pushout` 3a660d01c; merging `origin/master` was a no-op (master is already
an ancestor). rustup and the ALSA `1.2.8-1+b1` user prefix were rebuilt (the trixie 1.2.16
libasound breaks the test-binary link on this sandbox's older glibc — the prior windows'
finding), `work/` and the disc image re-downloaded (the archive exactly 369823203 bytes, the
ISO 1061754880).

**Item 1 — the press gate now covers thirteen on-foot levels, every walk re-run green.**
The leaf-press arm (`Drone::move_by`'s `leaf_press_rest` branch — the game's order: the move
applies first, then the contact pass's DeltaP push-out, the body resting IN CONTACT at the leaf
face [M Drone_CollisionHandler 0x140968:204-208 + Collide_Intersect 0x1ebaa8:353-373, collide
radius 0.4 = NDrone2_Collision 0x140e90:91-93]) was gated to 07000014 only. This task grew the
set one level per tool call, each level's whole walkreal binary re-run on the tree that carries
its addition:

| level added | walk result on the added tree |
| --- | --- |
| 07000007 (the task's first ask) | walkreal_07000007_live 6 passed / 0 failed / 0 ignored |
| 07000004 | 9/0/1i |
| 07000009 | 9/0/0 |
| 07000041 | 15/0/0 |
| 0700000d | 1/0/2i |
| 07000015 | 6/0/3i |
| 0700000a | 13/0/2i |
| 0700000b | 9/0/0 |
| 07000013 | 3/0/1i |
| 0700001b | 16/0/3i |
| 07000016 | 15/0/3i |
| 07000002 | 4/0/1i |

Every non-ignored test passed everywhere; the ignored entries are each level's pre-existing
ledgers, unchanged. **No walk shifted** — the press arm is the same scan the slide arm already
ran on 07 (the pure-face case the tangent scan leaves), and the walks tolerate it on every level
tried. 07000014's walks were re-run once more on the final tree: 7/0/3i.

**Item 2 — the door side is pinned.** `Doors::contact_centres(i)` was added (a read accessor
over the per-door `obj+0xD0` contact list, `DoorContact { centre, player, valid }` as the
collide step records it — the guard's `centre` is his root pos [M Drones::tick's DoorMover
build]). The 07000014 press test
(`the_countdown_leaf_walker_presses_in_contact_and_reaches_his_goal_once_the_door_opens`,
guard_stuck_sweep_live.rs) now additionally asserts, during the press ticks, that door 0x1A's
(list index 26) contact list carries the pressing guard — a non-player contact whose centre
matches his root pos within 0.05 — which is exactly the list `Door_ReactToHit` (0x135980) reads
at its dispatch head. Green: guard_stuck_sweep_live whole **3 passed / 0 failed / 0 ignored**
[L the run].

**The obligated binaries, on the final tree (all green):** nf-game lib tests **783 passed /
0 failed / 4 ignored**; npc_doors_live 6/0/1i (the pre-existing level-05-complete-2 ledger);
fix_npc_doors_live 1/0/0; fix_guard_misc_live 6/0/0; ai_traces_live 4/0/0; stealth_ai_live
10/0/0.

**The full suite did not fit.** The fast runner is disk-hostile in this 9.9 GB sandbox — in the
previous window the background suite.py filled the disk to 92% mid-build and had to be killed;
the foreground per-binary chunks in the table above (plus the obligated list) are what ran, and
they cover every binary the task names.

## What was run before the push

The gate change is under `crates/*/src/` (drone.rs: the gate line + comment; door.rs: the
accessor), so per the merge lessons every touched-path consumer was run: the whole walkreal
binaries of every gated level (the table), guard_stuck_sweep_live (the pin), npc_doors_live,
fix_npc_doors_live, fix_guard_misc_live, ai_traces_live, stealth_ai_live and the nf-game lib
tests — each with its pass count quoted above. No shared test helper changed
(crates/nf-view/tests/common/ untouched).

## How it was verified

- Every gate addition ships with its level's own real-input walk re-run green on the tree that
  carries it — the door_leaf_slide precedent, one level at a time.
- The door-side pin is white-box on the real press: the same 07000014 walk that proved the
  contact rest now also proves the door's own react path can see the presser.
- No test was weakened; no damage or health path changed; the only src changes are the gate
  set, its comment, and the additive read accessor.
- Not covered this window: the remaining on-foot levels (01, 05, 06, 08, 11, 12, 1b-wall, 4a)
  — same procedure, next window; 05/06/03's long walks did not fit the window's budget, and
  08's slide re-tune makes its press step the natural next check.

## Usage

not visible to me (no token, context-window or cost figures are exposed in this session).

## ntfy failed

All three fallbacks failed; the exact output, in order:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-leaf-press-07: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-leaf-press-07: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --timeout=20 --post-data "guard-leaf-press-07: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → exit code 4 (network failure), no body returned (a direct run, not piped — the piped
   reading would be `head`'s exit code, the earlier window's lesson).

git push to github.com worked throughout; only ntfy.sh is unreachable from this sandbox.

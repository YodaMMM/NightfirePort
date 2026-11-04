# level-01-agent-stairwell-doors — done

`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`
is un-ignored and green: Bond walks the real route from spawn #0 to the exit at Agent
difficulty on the port's own damage, end health **20.9** (lowest 20.9), `level_exit` to
07000002 at tick 10238, the contact on the LoadPointActivator plate, ch 0x10 on. No src
changes — the fix is the walk's own tactics in `crates/nf-view/tests/walkreal_07000001_live.rs`
(commit daf4fd0d).

## The finding: the stairwell ambush is a placed pocket pair, not a door route

The brief's levers (fight the pair from the hall's west side / catch them at door 2) died on
the disc data:

- **[L the trio trace on this tree]** (temporary eprintln at the hall AND, tick 4485): #12
  (skin 0x5000036, SMG 0x12, 12 hp) stands **disabled** (state 0x1, `enabled false`) at
  **(-31.31, 6.34, 11.51)** — the stairwell-head pocket — and #17 (0x5000039, SMG, 12 hp) at
  (-29.08, 6.34, -0.01), the south lower hall. They never move until enabled; they are not
  walking anywhere through door 2.
- **[D the placement dump]** (nf-view load, difficulty 2): the TOUCH slab `out 0xF mask 0x8`
  (player) spans **x -26.13..-24.73, z 13.25..16.84** — crossing it pops the pair (#12, #17
  plus #4 at (-27.23, 10.22, 2.28) upstairs south). The ch-0x26 slab at x -17.85..-16.45
  (crossed earlier, during the escort) enables #3 (0x5000038, SMG, **24 hp**, south room) and
  #2 (0x5000036, the balcony guard at y 10.22).
- **[L the sight probe]**: the corridor's walkable band at the slab's x is **z >= 13.4 only**
  (the z 11.8-13.0 hip rays all hit) — the slab covers the full walkable width, so Bond's
  crossing pops them on ANY line; and **#12's pocket has no clear eye ray from any mouth
  stand** (x -23..-24.5, z 13.5..16.5 all blocked) — while #17's post is exposed from all of
  them. #12 can only be killed when HE steps out (the pre-fix run's mouth fight drew him to
  (-30.6, 13.4) [L the dmg trace]).

Disabled objects are dropped from bullet collision ([M] Collide_PickObj 0x1eae80:82 —
`obj+0xF0 & 0x20`), so the ambushers cannot be thinned before the pop; the pop is by design
unavoidable.

## The fix (test-side only)

In `walk_to_the_exit_at`, Agent-only, after the escort settles and before the pre-stairwell
sweep:

1. **Cross the slab deliberately** — stance at (-24.0, 6.4, 14.8), cross to (-25.7, 6.4,
   14.8), wait for ch 0xF (the pop).
2. **Step back out immediately** to (-22.3, 6.4, 14.8): the poppers' 10-tick reaction [M
   DroneFunc_ReactionTime 0x14ac60:15-24, the 07000001 arm] leaves that window, and the
   back-off leg runs the walker's return fire at 6.5+ u — outside the TooClose doubling [M
   Drone_ModBulletDamage 0x139f50], where the fight machinery is proven. On the fix run this
   one leg dropped **#12, #3, #17 (all `killed true`) for 3.1 health** (42.9 → 39.8).
3. **The pocket rule**: a guard parked west of x -27 is fought SHOOT-ONLY from where Bond
   stands — no nav approach ever walks him back into the 0.4-u corner where the pre-fix run
   died (Bond at (-30.26, 6.37, 13.64), tick 7594, #12/#17 at 0.4-0.9 u [L the pre-fix dmg
   trace]). The same gate guards the pre-stairwell sweep's approach legs.

The pre-fix failure chain, for the record: the sweep's approach to guard #3 crossed the 0xF
slab mid-nav, the pop volley landed at 2.6 u while Bond was still under nav (3×2.1 at t7094-
7106), the backed-off fight left #12 at 6 hp, and the next nav walked Bond back into the
corner where the surviving pair shredded him.

## What's left

Nothing for this task. The walk's margin is real but not fat (20.9 end health; the hall
door-lure phase still bleeds 33.6 to the shotgun captain #5 — the "settle gap" lever from the
brief remains open if a later task wants more headroom, but the walk is deterministic and
green as is).

Live checks for the main agent: **none required** — the fix is walk tactics against the
port's own sim; every fact it relies on is pinned [D]/[L]/[M] above, and no visual output
changes. (Optional, if the main agent wants belt-and-braces: during a real PCSX2 Agent run,
watch switch_channels[0xF] (0x2a38d2+0xF) rise when Bond crosses x ≈ -25.4 at z 13.2-16.8 in
the west corridor, and drone #12's obj waking from state 0x1 the same tick — expected per the
slab + placement dump above.)

## How it was verified (every binary with its count)

- `walkreal_07000001_live` — **7 passed / 0 failed / 0 ignored** (the un-ignored Agent walk
  included; was 6/0/1 on master).
- The full walkreal sweep, 26 walk binaries + celprobe + dump, `--test-threads=1`, 0 failed
  everywhere, only pre-existing ignores:
  walkreal_07000002_live 3/0/1, 03 6/0/1, 04 10/0/0, 05 14/0/0, 06 6/0/0, 07 5/0/0,
  08 5/0/1, 09 8/0/0, 0a 12/0/3, 0b 7/0/0, 0c 3/0/0, 0c_barrelrow 1/0/0, 0c_eastroute 1/0/0,
  0d 1/0/1, 11 2/0/0, 11_route 1/0/0, 12 3/0/0, 13 3/0/0, 14 9/0/0, 15 5/0/1, 16 14/0/0,
  1b 17/0/0, 41 11/0/0, 4a 2/0/0, 4a_satchel 2/0/0, celprobe 2/0/0, dump 1/0/0.
  (The ignored counts are the other tasks' known ignores: 02/03/08/0a/0d/15 — untouched.)
- nf-game: **1047 passed / 0 failed / 8 ignored** (101 integration binaries) + lib
  **656 / 0 / 5 ignored**.
- nf-formats + nf-audio: **136 passed / 3 failed / 0 ignored** — the 3 are exactly the slim-ISO
  allowance (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`).
- nf-view's remaining ~1070 integration binaries (the chunk runner reached 181 passed / 0
  failed / 2 ignored before the window closed) did **not** fit this sandbox window. The task's
  change touches only `walkreal_07000001_live.rs` (a test file; no shared helper under
  `crates/nf-view/tests/common/` changed, no `src/` change), so the affected set is the 01
  binary + the walkreal sweep above, all green.

## Usage

not visible to me

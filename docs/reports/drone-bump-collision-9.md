# drone-bump-collision-9 — the 07 party hall east door: the sister's use zone and the post-E wedge

Branch `s/drone-bump-collision-9` (from `origin/s/drone-bump-collision-8`, merged `origin/master` — one
conflict, `crates/nf-view/src/scene.rs` bot_capsules, resolved for the -8 lineage filter with master's
corpse-gate citation folded into the comment).

## What's done

### 2. party_door_lock_07000007_live's sister — FIXED, 2/0 green

The sister's old stand point (94.5, −47.6) made the "#244 still answers the use key" assert fail because
the game's own swing-door arm swallows that press:

- [M] `Door_Activate` 0x135be0, the kind-0 arm (the `RotPreTransVec` block, LAB_00135e30): an OPEN swing
  door (state & 0xc) E'd by an activator whose closed-frame z (± the 0.3 margin, sign by dir·swing) lands
  |f| < the door's world-sphere radius (obj+0x8C) returns with only the busy latch — no state change.
- [D] door #244's numbers (probe_07_route's d244 dump, the port's placement load): kind 0, closed sphere
  centre world (94.254, 44.125, −47.250), r 1.292, hinge (94.888, −47.25), local z → world −z, opened
  dir = +1 (south swing). At (94.5, −47.6) the closed-frame z is −0.35 → |f| = 0.65 < 1.292: inside the
  band, the press is eaten.
- The re-derived stand point: **the 0x48 trigger #412's box centre (94.27, −45.92)** [D trigger bounds
  x 93.31..95.23, z −46.43..−45.41]. There the closed-frame z is −1.33 → |f| = 1.63 > 1.292 — outside the
  swing band — and the reach holds: [M] `Player_Activate` 0x1a5170:100-108's r-1.0 sphere at head + dir
  vs the door's LIVE world sphere [M Collide_PickObj 0x1eae80's 0x101 arm; the moved pass rebuilds
  obj+0x80, 0x1336f8:209] — the open leaf's live centre (94.888, 44.125, −47.884) sits ~1.8 from the use
  point, inside 1.0 + 1.292. The E toggles the leaf; `moved` goes true.

Verified: `party_door_lock_07000007_live` **2 passed; 0 failed** (8.66 s) on this tree. The push-set
filters (the 0x31 CIVILIAN_DOOR_GUARD exemption, the kind_464/straddle gates) are untouched.

### 1. probe_07_route's post-E hover wedge — DIAGNOSED (the floor ray is exonerated); the route anchors are the fix point

New assertion-free probe `crates/nf-view/tests/probe_07_hover.rs` measures the wedge spot directly:

- The port's scoped feet ray at (93.41, −46.32) reads the REAL passage floor **y 43.000** — scoped ==
  flat, normal (0, 1, 0) — exactly the face the game's cel-scoped ray reads [M Player_FeetOnPoint
  0x194188:89-106]. The ray is NOT the bug.
- The CAPSULE rests on a **walkable rim at y ≈ 43.215** (0.219 above the floor): a synthetic-capsule walk
  of `capsule_push_scoped` shows walkable=true with a grazing push (0.0018, 0.0010, ~0) at
  (93.41, −46.32), walkable=false at z −46.0 / −46.6 / −45.4 / −45.8 / −47.0, and the same hover at the
  drifted spot (92.90, −45.82). That is a level-placement object's top edge beside the route line — the
  game's own collision resolves the same contact the same way [M Player_Collision 0x194df0's walkable
  arm; the band at 0x194188:106 grounds the feet ON the rim]. The -8 dump's grounded=false was this same
  stand mid-walk (the push dither), not a floor-read divergence.
- Consequence: the -8 "hover wedge" is the probe's post-E anchors walking Bond onto that object —
  re-deriving them needs the object's footprint ([L] live render/check below), so probe_07_route stays
  red this round and nothing in the port's collision was changed for it. Its first post-E leg is also
  provably blocked: from the post-E spot the line to (92.6, −47.5) hits at 0.60
  (segment_to_target=Some(0.602), no door in the way — #244 stands OPEN state 0x4 pos01 1.00 in the dump).

## How it was verified

- `party_door_lock_07000007_live` 2/0, `cutscene_timer_pause_live` 1/0, `walkreal_07000007_live` 2/0 —
  all run on the final tree binaries.
- Suite (fast runner, `--only` critical set for the changed areas — see "What's left"): 5 binaries
  completed inside the tool window — party_door_lock_07000007_live (8 s), cutscene_timer_pause_live,
  fix_player4_live (4 s), fix_npc_doors_live (197 s), drake_fight_live (16 s) — tally **11 passed,
  1 failed**: `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact`. That failure is
  on the MERGED tree (master's drone.rs state changes shift the shared-RNG stream — the merge's own
  level-13-guards-7 note records exactly this class of shift); it was not bisected this round.

## What's left

- The full-suite total line: the sandbox lost ~50 minutes to a full-disk outage (a bus-error link
  failure at 100% rootfs — cleared by purging the stale pre-merge build artifacts) and to background
  jobs being killed between tool calls (the runner must run in ≤10-minute foreground chunks). The
  890-entry suite (17+ min) does not fit one chunk; the `--only` run above is the quoted partial. Next
  round: run `tools/test/suite.py` to completion on this tree and quote its total line.
- `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` — bisect against
  `origin/s/drone-bump-collision-8` vs the merge (the RNG-shift suspect above). Do not weaken the test.
- probe_07_route's post-E legs: re-derive the anchors around the object at (93.4, −46.3) once its
  footprint is known; the sister and walkreal_07 need nothing.

## Live checks for the main agent

1. The object under the hover: render/place-inspect (93.4, −46.3) → (92.9, −45.8) on 07000007 (the east
   doorway's north side, passage floor y 43.0). Expected: a static placement whose top face sits at
   y ≈ 43.2 spanning roughly x 92.9..93.5, z −45.8..−46.6. Access type: static-collision read (the
   scoped capsule walkable map already bounds it); the instance name/size settles the route re-derivation.
2. #244's use from the box centre on PCSX2: stand Bond at (94.27, −45.92) facing the open hall door and
   press E. Expected: the leaf moves (Door_Activate's toggle), matching the port's 2/0 sister walk.

## Usage

- Context window: not visible to me (no token/credit figures are exposed in this environment).
- Input/output tokens, cost: not visible to me.

## ntfy failed

All three fallbacks returned HTTP 429 (the ntfy.sh daily publish quota — server-side):

- `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
  `{"code":42908,"http":429,"error":"limit reached: daily message quota reached; increase your limits with a paid plan, see https://ntfy.sh","link":"https://ntfy.sh/docs/publish/#limitations"}`
- `python3 urllib` → `urllib.error.HTTPError: HTTP Error 429: Too Many Requests`
- `wget --post-data` → `ERROR 429: Too Many Requests`

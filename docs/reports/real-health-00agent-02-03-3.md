# real-health-00agent-02-03-3 — report (2026-11-04)

**PART DONE.** The 03 and 02 00 Agent walks are both still red on master and both keeps their
`#[ignore]` — but the 03 death is now **fully diagnosed with a two-ledger diff** and a working
pre-clear that kills the decisive guard cluster at zero fight cost; what remains is one approach
problem and one stall. The 02 sister was not started (the window went to the 03 diagnosis). The
branch is `s/real-health-00agent-02-03-3`, cut from `origin/master` a71f9233.

## The 03 diagnosis (new evidence, all [L] NF_RF_TRACE ledgers on this branch and on
## origin/s/real-health-00agent-02-03-2)

1. **The runs are IDENTICAL to t48500.** The old (pre-door-routes) branch's green walk and
   master's red walk produce the same fights at the same ticks the whole way — including the
   mid-courtyard double-door stalls (the (10.0/11.8,−20.2) doors stalled 2x3000 ticks in the
   GREEN run too; the stalls predate the door routes) and fight#8 at t21426 with health 43.6
   in both.
2. **The divergence is the servant-0 captor flank.** Green: the reactive fights killed
   #19 -> #18 -> #2 for 12.0 total and the walk finished at t61620 with **28.8** (exit loads
   07000004). Master: the guards' door routes moved the captor ~1.4u, the flank spot moved with
   him, and the return-fire pick flipped to **#2 first** — #18 survived into a second fight whose
   **28.8 volley (40.8 -> 12.0)** is the whole green margin. Bond then reached the exit lawn at
   12.0 and the trio there finished him (t59112, (-1.38,5.96,-34.47)).
3. **The cluster**: #19 (skin 0x5000036, hp 12), #18 (0x5000036, hp 12), #2 (0x5000038 shotgun,
   hp 24) at (35.5-37.5, -0.6..-0.8) around door #8 (35.6, 6.5, 2.7). The green run fought them
   from (36.2,5.7) THROUGH the open doorway — parts expose, blocked false.

## What was built (committed, d3+si0-gated in the 03 file only)

- A **pre-clear** before servant 0's captor duel: walk to the green fight spot (the rally,
  (36.5,5.7)), then the cluster hp-ascending with real fire and the resupply retry. **The fights
  resolve at zero cost** (#18/#19/#2 all killed true, health held).
- A **post-rescue continuation walk** to (38.4,0.8) (the green run's chase-end position) so the
  next leg's nav threads the east side like green.

## What's left (the three levers, in the ignore message too)

1. **The rally approach crosses the gun line**: the nav reaches (36.5,5.7) only through door 8's
   threshold (35.07,2.13) — Bond walks INTO the cluster's 3.0-u shotgun band and pays 21-25
   (40.8 -> 15.6-19.2); a straight `walk_to` wedges at the door instead. Need an approach that
   stays north of door 8, or a nav-leg split that anchors the path.
2. **The (35.6,8.7) door stall**: from the rally the post-rescue west-lawn leg detours through
   that door and E-stalls 29x3000 ticks (87000). The continuation walk above is committed but
   unexercised (the walk_to run died before reaching it); the next round must verify it actually
   removes the stall.
3. **The exit-lawn #16 post-kill volley**: in every run, drone #16's fight ends "killed true"
   with health unchanged and Bond dies ~80 ticks later to hits attributed to #16 (six -0.00
   records at the death frame). Either the kill does not stick (a zombie-drone sim smell worth a
   probe) or the attribution is the dying volley. Worth a look before the next walk round.

## Verification

- `walkreal_07000003_live`: **6 passed / 0 failed / 1 ignored** (114.6 s) after the changes —
  the six other tests are untouched by the d3-gated edits.
- `walker.rs` (shared) ended **byte-identical to master** (the diagnostic probes were reverted;
  `git diff --stat` clean). No `crates/*/src/` files changed, no shared test helpers changed, so
  the other walkreal binaries are not affected by this branch's diff; the full sweep did not fit
  the window.
- The 02 walk: untouched on this branch (the same parked state as master: 3 passed / 1 ignored).

## Live checks for the main agent

1. The exit-lawn #16 (shotgun captain at (2.5,5.8,-33.1)): after Bond's return fire reports
   "killed true" on him, do his shots keep landing? A gdbwatch on his Drone record (+0x1c
   health / corpse flag / state) for 100 ticks after a killing volley would settle whether the
   port leaves a live shooter at hp<=0 (zombie) or the records are the dying volley's in-flight
   pellets.
2. The (35.6,8.7) east door: from the rally side the walk's E never opens it (29 attempts,
   3000 ticks apart). A fieldprobe E at that door on a real PCSX2 run from (36.6,4.7) — does
   the game's own Bond open it, and if not, what state does the door show (the swing-path busy
   marker vs a locked-channel rejection)?

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).

## ntfy failed

All three fallbacks failed in order (2026-11-04 ~01:58 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "real-health-00agent-02-03-3: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. python3 urllib (fallback 2)
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → killed after 30 s with no output (connection never established)

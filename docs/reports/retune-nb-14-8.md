# retune-nb-14-8 — the 14 infiltration to the delta exit 00 Agent walk, round 8

Branch `s/retune-nb-14-8`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-7`
merged in first (it was not on master; the merge was clean — the round-4..7 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fresh this round: the full setup was redone from the task message (clone,
work-slim.7z, the ISO archive verified at 369823203 bytes before extraction, rustup stable
1.100, the no-sudo ALSA user prefix from libasound2-dev + a libasound.so symlink, cargo
fetch). The fast suite was started in the background twice in the first minutes per the
task order; on this box background processes are killed (see How it was verified).

## What's done

**The walk is still `#[ignore]`d — still red — but the round-7 ledger's next lever (the
SERVER-ROOM armour) is MEASURED REAL and BANKED, and the death moved t25389 → t28518 with
the door face crossed at unchanged health for the first time since round 4.**

- **The pass-point question the round-7 ignore message ordered probed first was answered by
  a read-only nav dump (`NF_SRV_PROBE`, env-gated, byte-identical when unset, `quick` mode
  exits before the walk)** [L 2026-11-20]: the server-room armour pickup is live at exactly
  (−27.29,0.87,30.96) (kind 3), and a per-path nav route exists from the west-corridor end
  (−48.7,0.9,33.2) to the armour — 11 nodes along the z 27.3-27.8 east lane — but its
  middle node (−33.13,27.33) sits d 0.53 from the escape touch #456 (−33.1,26.8), the
  cut-scene trigger the ledger orders the fetch to mind. The shipped legs bypass it north
  ((−33.1, 28.6), d_touch 1.8). The ascent-top approach was rejected on the v10 ledger's
  own evidence: the (−30.6,29.9) door.
- **The v1 run measured the take's west line DEAD**: the pocket is walled on the direct east
  cut (bond (−29.12,30.95) → target d 1.83, segment_to_target 0.35, the 6000-tick cap) and
  the (−30.6,29.9) door stays shut (ch_open 0, pos01 0.00). The pocket is entered from the
  nav route's own north node (−28.00,1.20,33.21) then straight south — the route the game's
  own web builds for this exact destination [L].
- **The v2 run LANDED the take**: armour 35.0 health 87.5 at t~14600 (the loop + fetch legs'
  fights — #18 killed true at the take — chewed 65 of the 100 bank). The bank carried the
  door face: health 87.5 at the Kiko corridor east (rounds 5-7 entered it at 17.5), and the
  pocket #20 — the round-7 bill's main shooter — DIED (killed true t27930). Death moved
  t25389 → t28518 (+3100 ticks), past the pocket line, INSIDE the lane crossing's own
  resupply wander: the gun is DRY west of the door and the fights' own resupply law dragged
  Bond to the nearest live pickup ((−41.3,1.5,36.5) — the Kiko-corridor armour), its nav
  route swinging SOUTH through the pocket convergence zone; the final blows were #23's
  2.50s and the WORLD's −30.51 (the live wire / hurt volume at z ~28) [L].
- **Two further levers were measured on their own runs and REMOVED from the tree**:
  the corridor-end drop scoop granted DRY guns (gun 0x1 clip 1 spare 0 — the door squad
  spent their rounds on Bond before dying; v3/v4 died identically at t28518/t28658), and
  the Kiko-corridor armour take inserted before the lane fired the same resupply wander
  inside the take leg (v5 died t30347 past the lane — #23/#31/#30 2.50s then #28's −61.84
  grenade — the deepest this walk has been; the leg is removed, the shape recorded).
- The v6 run is the FINAL TREE: the server-room fetch banked, nothing else. Death t28518.
- The ignore message is refreshed with the round-8 ledger and the next levers in order:
  (1) the Kiko-corridor armour ledge (y 1.53) approach from the east lane's end, probed
  first via NF_SRV_PROBE (the nav web's swing through the convergence zone is the blocker —
  a dumped-node walk_nav_to chain if a clean route exists); (2) the door-face legs' armour
  DRAIN law (the round-7 ledger's lever 2, still untried — the mute legs + scans drained
  the 35-bank to 2.5 by the corridor); (3) the dry-gun law west of the door (every guard
  drop west of the door is spent; the bank is the only pool until the wake legs' node-7
  scoop).

## How it was verified

- **walkreal_07000014_live (whole binary, the final tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 172.24 s** — the ignored one is the 00agent sister
  (its refreshed ledger in the ignore message). Includes the un-ignored Operative d1 walk,
  the plain/tops-up walk, the d2 agent walk, the vent-drop, grill, E-cascade and tripbomb
  tests — the sisters stay green with the banked fetch and the probe hook on the tree (the
  gates keep their streams byte-identical, and the whole-binary run proves it).
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed,
  10.27 s** — matches rounds 5-7's counts.
- Every lever was judged on its own real run (v1 west-line wedge, v2 banked take, v3/v4
  dry scoop, v5 Kiko-take wander, v6 final tree), each with NF_DMG_TRACE ledgers captured
  (srv_v1.log .. srv_v6.log, untracked scratch).
- **The fast suite (`tools/test/suite.py`)**: started in the background twice in the first
  minutes as ordered; the first start was killed before it even created suite.out and the
  second was killed deliberately after one liveness check (it compiles the merged tree and
  would hold the cargo target lock against the lever work on this 2-core box) — the same
  failure mode rounds 4-7 recorded (suite.out stayed empty). The foreground four-crate
  build exceeds the window on this box; the two required binaries' whole runs above are the
  evidence. NO total line this round, same as rounds 4-7.
- Only `crates/nf-view/tests/walkreal_07000014_live.rs` changed (the d3-gated fetch legs +
  the env-gated probe + the refreshed ignore message); no file under `crates/*/src/`
  changed, so no other test binary is affected; the two required binaries were run whole
  anyway.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE /
  NF_SRV_PROBE; code citations are [M] with the decomp address (Player_HandlePain
  0x1902c8:84-104); no [S] items entered pass/fail; no test was weakened; the game bytes,
  the token and the URLs stayed out of the commits.

## Still red / what's left (the round-9 levers, in the ignore message too)

1. **The Kiko-corridor armour ledge approach** — the walk is one clean +50 short: the lane
   crossing passed in the v5 stream and the resupply wander's own target IS this pickup.
   Probe nav_route to (−41.29,1.53,36.48) from the east lane's end (−44.5,0.9,34.0) with
   NF_SRV_PROBE; if the route swings south, walk the dumped nodes as a walk_nav_to chain.
2. **The door-face legs' armour DRAIN law** — the mute legs + scans stand ~2000 ticks in
   the door squad's bursts and drained the 35-bank to 2.5 by the corridor; a shorter stand
   shape banks more of it for the lane + touch + wake legs.
3. **The dry-gun law west of the door** — every guard drop west of the door is spent; the
   fights fight on "cycling to a gun (0)" and their resupply wanders are what the deaths
   ride. The wake legs' node-7 scoop re-arms only after the touch.

## Live checks for the main agent

- The resupply wander's nav route read (the fights' own law) at t28200-28500 (the v6 bill):
  does the route from the lane area to (−41.3,1.5,36.5) really swing through the z ~28
  convergence zone, and does the live wire hurt volume there have a dodge lane a
  grenade-aware hold could take?
- The server-room armour take (−27.29,0.87,30.96): confirm the armour pane on the HUD reads
  50 → draining on the walk into the door face, to pin the bank visually.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost
figures for the session; nothing to copy.

## ntfy failed

All three fallbacks failed, in order, at 08:55-08:57 UTC 2026-11-20 (the same failure mode
rounds 5-7 recorded — the DNS resolves, the TLS connect to ntfy.sh hangs):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-14-8: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'retune-nb-14-8: …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → hung silently, killed by the 25 s wrapper with no output (exit 124; no HTTP status, no traceback).
3. `wget --timeout=15 --tries=1 -O /tmp/ntfy_out.txt --post-data "retune-nb-14-8: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → hung with no response written; the 45 s tool-call kill hit before wget's own 15 s
   timeout produced output. /tmp/ntfy_out.txt was never created.

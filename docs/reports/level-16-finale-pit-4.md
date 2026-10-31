# level-16-finale-pit-4 — 07000016: the finale window, round 4

Branch `s/level-16-finale-pit-4` (master b44ac857 + the pit-3 merge). **Status: blocked** — the
walk does not complete yet, but this round closed the mechanism to a single decidable question
and produced the live checks that answer it. No `nf-game`/`nf-formats`/`nf-audio` source
changed; the only code edit is the ignored walk's citation (now carrying the full chain).

## What this round proved (every fact tagged)

### 1. The volume really deals the damage — all three stages read [M]

- **The who-mask is a START gate only.** `Hurt_Update` 0x17e5d0:51-104: in state 0 the gated
  arm takes `uVar5 = (byte)switch_channels[gate]` (line 98) and `uVar5 != 0 → obj+0xF4 = 1`
  (state 1). The mask is used nowhere else — it is not stored, not ANDed per victim, and the
  function applies no damage at all. ch 0xf3's pulse (value 1) starts the volume.
- **The victim pass is ungated.** `Player_DealWithObjHit` 0x194458:95-102: class 0x27 →
  `Player_HandlePain(Hurt_GetDamage + hitdata+8, …)` with no state/flag check;
  `Player_CollisionHandler` 0x1949a8:24 returns early only when the player is DISABLED
  (`+0x94a == 0`) or `debug_scanmode` is on.
- **The collide candidate law accepts the volume while the gate is high.**
  `Collide_PickObj` 0x1eae80:82-90 rejects a candidate when `obj+0xF0 & 0x20` (the gate-low
  inactive bit Hurt_Update's maintenance sets/clears) — cleared once 0xf3 rises — and for the
  player's sphere test (mode 0x100) the final clause short-circuits TRUE, so `Hurt_Create`'s
  `+0xFA |= 0x44` (no bit 8, no solid bit) excludes nothing. `Collide_Update` 0x1ea398 then
  runs `Collide_Intersect` on the volume's glist (216 tris [D]).
- Conclusion: **an enabled player touching the volume's box while ch 0xf3 = 1 takes 1000
  damage in the shipped engine.** The port's geometric contact matches the decompile.

### 2. The hand-back chain is [M]-complete — and it lands in the volume box

- **event 6 snapshots record(i−1).** `Script_EventHandler` 0x1d94c0:73-84 looks up
  `info + (byte)info[0xab3]·0x28 + 0x80` and takes that record's entity when its kind ∈ {1,2},
  else copies `glb_players+0x90` (the player's live matrix) into `SCRIPTINFO+0x40`
  (`Mat_Copy(src, dest)` — the const/mutable mangling settles the direction);
  `Script_Update` 0x1da998:29-31 increments `+0xab3` **after** each `Script_Run`, and
  `Script_Run` 0x1d8548:14 processes the stream at `+0xab3` with records at `0xa8+i·0x28` — so
  during stream i's processing the handler reads record(i−1).
- **For 06000631's event 6 (stream 9, t747) that record is stream 8** — the Entity stream
  `[D nis dump: Wait(2eb), Entity { hash: 0, parent: None, spline: false, obj_type: 31 }]`.
  `obj_type 31` is not in `Script_CreateEntity` 0x1d8968's class map → the generic tail
  `control_create_object(0,0,0,0)` — a **model-less dummy**; hash 0 attaches no gfx. The
  command sets record+0x10 = the dummy and record+0x24 = **kind 1** (0x1d8d78:101).
- **The dummy is posed AT CREATE:** `Script_EntityStart` 0x1d8d78:111 calls
  `Script_Interp(info, stream)` immediately after the create — stream 8's key track is the
  single key 204, t747, **(3.57, −22.76, −179.55)** [D key dump] — so the snapshot the next
  stream's event 6 takes is the dummy's matrix.
- **`Script_Free` 0x1da880:44-48 → `Player_Enable(glb_players, SCRIPTINFO+0x40, +0xab6)`;
  `Player_Enable` 0x195498:54-68 teleports unconditionally** (`obj+0x30 = matrix[3]`, look
  re-derived, unlink + `build_LinkToRoom(1)`). **The shipped Bond re-enables at the pit-floor
  key, inside the volume box (x −30.7..35.3, y −24.08..−21.32, z −197.3..−133.8 [D]), 1611
  ticks before the pulse.**
- The port's own reading (`streams[i−1]`, the create-time key application) matches all of
  this — the port is faithful here; no code change was made.

### 3. The escape map is closed [D]

- Door ch 0xf0 @2180 — **9 ticks after the pulse drops** (1611..2171); the lift ch 0x50 @2240.
- The button's inhibit ch 0x8d has **no other disc writer**: 07900016.bin holds ONLY 06000632
  (its `[8d,0]` @t420 is the single 0x8d-clear in the packs I can dump), and 06000632's gate is
  ch 0x82 = mux #286 (0xbf·0xc0) [D trig dump]. **0xc0's writer is new this round: the launch
  script 0600090a's own stream 20, `[c0,1]` @t7840 [D]**; 0xbf needs all four pit-squad deaths
  (mux #264, chs 0x17..0x1a [D]). The button-first route (pit-3's list item) is dead: the gate
  stays latched until the pit squad dies AND the launch script nears its end.
- The launch script fires **no event 4/6/7** (no Bond disable/re-place inside it) [D dump].

### 4. The one open link — the live checks decide it

`Script_Interp` 0x1d9a08's **mode-0** behaviour for a one-key track. The Entity record's spline
byte is FALSE → `Script_EntityStart` 0x1d8d78:102-104 sets record+0x22 = 0. If the create-time
interp does not apply key 204, the shipped dummy (and the hand-back) is `control_create_object`'s
**origin** pose, not the pit floor — the whole finale question moves. The port applies the key
(its phase-A assert pins the port's own reading — a port assertion, not game evidence).

## Live checks for the main agent (PCSX2, FRESH boot of 07000016 — a save-state resume skips the boot path)

1. **The hand-back position**: watch Bond's `obj+0x30` (0x26fc90-family player object base —
   the player object's +0x30 vector) across the defeat cinema 06000631's hand-back.
   Expected per the [M] chain: (3.57, −22.76, −179.55). **If the real game shows any other
   pose (the bridge, the origin), THAT difference is the port gap** — record the value.
2. **The pulse's effect**: watch `switch_channels[243]` (0x26fc90 + 243·4) and Bond's health
   through the launch. Expected per the [M] chain: 243 rises ~2359 ticks after the ch-0x81
   touch and the player DIES at the pit floor. If the player instead SURVIVES at the pit
   floor, the damage delivery differs from `Collide_PickObj`/`Player_DealWithObjHit` as read —
   watch `Hurt_GetDamage` 0x17ea18's caller during the window (the long-open live check from
   docs/systems/hurt-volumes.md) and record who calls it and with what.
3. **If Bond is moved**: `tools/trace/gdbwatch.py` on `obj+0x30` from the hand-back to the
   pulse records any writer that relocates him — that write is the shipped survival mechanism.

## What's left

- The three live checks above (one boot covers all three).
- Whichever way they land: (a) pose ≠ pit floor → fix the port's create-time interp/hand-back
  to the observed source and the walk proceeds on the proven route legs; (b) pose == pit floor
  but the player survives → the damage delivery needs the observed gate; (c) the player dies →
  the shipped finale really is gated behind something outside this surface (the packs I cannot
  dump: 07a00016+ need the nf-extract rebuild that the sandbox's disk pressure blocked).
- The chaser-fight resupply leg (pit-3's +61 ammo-out) for the kill, or the pit-squad + button
  completion ride through 06000632's t420 re-arm.

## Verification (the touched binaries, quoted)

- `walkreal_07000016_live` — **8 passed / 0 failed / 1 ignored (42.10 s)** (the ignored tail
  now carries this round's citation).
- `interactables_07000016_live` — **8 passed / 0 failed (42.98 s)**.
- `nf-game` lib tests — **588 passed / 0 failed / 5 ignored (0.05 s)**.
- The fast runner (`tools/test/suite.py`) was started on the final tree; the sandbox's disk
  filled twice this session (ENOSPC killed the first two build attempts — `cargo clean` +
  rebuild recovered ~6.8 GB), so the full-suite total may not land before the window closes;
  the partial totals are in `suite.out` at the repo root (uncommitted, per the rules).

## Usage

not visible to me — this sandbox exposes no token/credit/context-window figures in any
readable surface (no environment variable, log or UI text was found).

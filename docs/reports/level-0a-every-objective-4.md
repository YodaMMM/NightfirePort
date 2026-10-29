# level-0a-every-objective-4 — the ring's mouth is the keypad door #724; the keypad E-path's sim side ported (the audit's GAP-1 machine); the mission walk GREEN end-to-end and un-ignored

## What was asked

Continue `level-0a-every-objective-3`: the walk still wedged at the ring's mouth (three walked
wedge points in the ignore reason). Find the route the game's player takes through the mouth
[D the collision around the mouth; M the player's capsule resolve; L if needed], fix any port
collision gap, walk the ring legs and the hatch E on real input, un-ignore
`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch`; FULL suite with its total line.

## What the probes found (scratch probes, not committed)

**1. The wall seals the pocket — there is no open crossing.** `nf-extract coll-scan` over the
mouth + a 4-ray occupancy map + a 3-ray-corridor BFS over the port's collision (x −24..4,
z −36..−6, step 0.5) all agree: the wall at x ≈ −10.45..−10.70 (`Room-L-atrium_top` /
`corridors_b_6`) runs CONTINUOUSLY from z −15.38 down to z −23.70 (the west face x −10.45 spans
z −17.96..−15.38, −21.62..−19.30, −23.70..−21.62 [D coll-scan]), joining the diagonal walls
south. The -3 round's three walked routes failed because all three crossed a wall that never
ends. The BFS from the pocket, the corner approach AND the Q-Worm band reached NONE of the
east-side goals (node 130/132/0, the south corridor) [D sim rays].

**2. The only opening is door #724's doorway — and the game's own ai graph crosses exactly
there.** The wall's one gap is the doorway (the jamb faces z −18.04/−19.36, 1.32 wide [D
coll-scan]) filled by `room-L_door2` #724, locked on ch 0x30 (p3 = 48 [D]). `nav_route` from the
pocket to node 0 routes pocket → **(−11.01,−18.60) → (−9.88,−18.62)** → (−8.70,−21.55) — straight
through the doorway; the same nodes carry the band→lobby route [D 0700000a ai_network]. The
door's lock channel 0x30 has no trigger/script writer — its writers are the **keypads**: the 0x2E
Lock placements #648 (−10.695, 1.524, −17.456) and #732 (−10.450, 1.524, −17.456), kind 0, code
index 1, p5 = 48 [D placements; rotations ±π/2 — #648 faces the pocket, #732 faces the wall].
They stand on the doorway's north wall face, one on each side of the seam.

**3. The keypad chain is the game's route, and the port had the machine but not the wires.**
[M Player_Activate 0x1a5170 case 0x3A]: the use pick (the sphere at head + 1×look) hits the lock,
the facing gate `Mat_GetDir(lock+0x90) · (user+0xC0 − lock+0xC0) < 0` refuses, then
`Lock_Activate` 0x1aed00 (disasm): class-3 user, no pause manager, lock state 0/5 → **kind 0
with an empty key-ring slot → state := 3** (arm). [M Lock_Update 0x1ae3d8 case 3] copies
`KeyCodes[p1·5]` (5 bytes) and creates the keypad menu; [M C_KEYPAD_Handler 0x216810 msg 0x4b]
fills the typed buffer; [M case 4 :262-296] the full match fires `Lock_Unlock` 0x1b0650 (state 1,
the key equipped, NextLOD) and the **next case-1 tick holds `switch_channels[p5]` = 1** — the
door's rising-edge unlock + auto-activate [M 0x1363c8:101-107]. The port already had
`Lock::decrypt_tick`, `Keypad` (arm/digit/tick/close, lock.rs) and the unlocked-lock channel hold
loop — what was missing was the E path, the arming, the keypad tick and the codes.

**4. The codes are [L]-readable.** `KeyCodes` @0x2beff8 (symtab) in the level-start snapshot:
`work/ee_start_0700000a.bin @0x2beff8` carries the boot's own table — code index 1 reads
**"3078"**, shown flag 0 [L]. (The game generates the table during boot with the shared stream
[M Locks_Init 0x1b0548] — those draws predate the snapshot's capture point, so the port
generates on a local stream [S, deterministic] and the walk test overrides the field with the
snapshot bytes [L].) Note the Q-Worm install does NOT reveal the code on 0a: PCQWorm_Update's
reveal arm runs only for kind ≠ 0 PCs [M 0x1aee20] and #688 is kind 0 [D placements] — how a
real player learns "3078" is a live check below.

**5. The port's door leaf is NOT a ghost.** A fresh-load probe showed the doorway clear —
an artifact: `set_dynamic` runs per tick, so a zero-tick sim has NO door collision. After 5
ticks the closed leaf blocks every crossing ray (hits at x ≈ −10.6, z −18.1..−19.0 [D sim]);
with the door open (state 4) every eastward ray z −18.3..−19.15 is clear [D sim] — the open
leaf parks clear of the walked line.

## What changed

- `crates/nf-game/src/lock.rs`: `Lock::facing` (the placement rotation's matrix row 2 — the
  0x3A facing gate's direction [M 0x1a5170:158-166]), computed in `Lock::create`.
- `crates/nf-game/src/drone.rs`:
  - `Drones::key_codes` (`Locks_Init` 0x1b0548 over a local `Rng::new(0x1b0548)`, warmup 0 [S —
    the game's GameState+0x34 warmup is session state the fresh load does not model]) and
    `Drones::keypad: Option<(usize, Keypad)>` — new fields, own lines at the struct/constructor
    ends.
  - The class-0x3A use arm (next to `useobjs.player_activate`): the same use sphere, the facing
    gate, the Lock_Activate gates (not unlocked/burned/scanning, no keypad already armed), kind 0
    → arm with the table's code [M 0x1aed00 disasm]. The key-ring consult stays unported (the
    recorded lock-side gap, lock.rs header) — the slot reads empty [S, documented].
  - The keypad tick after the channel-hold loop: `Keypad::tick()` on solve → the lock
    `unlocked = true` (+ the existing hold loop writes ch 0x30 the next tick, the game's
    case-1-next-tick order) [M 0x1ae3d8 case 4 → case 1].
  - `Drones::keypad_digit` / `keypad_display` — the C_KEYPAD_Handler msg-0x4b port [M 0x216810].
- `crates/nf-view/tests/walkreal_0700000a_live.rs`:
  - `load_carry` overrides `key_codes` from the snapshot [L @0x2beff8].
  - The ring: the walked legs to the pocket, the keypad stand (−11.9,−17.5), the REAL E (aim +
    hold-use through `step_bond`, the keyboard-E pattern), the 4 digits from the [L] code,
    asserts: the keypad armed [M 0x1aed00], ch 0x30 + door #724 state 4 [M 0x1ae3d8 case 4 →
    case 1 + 0x1363c8:101-107], the used lock unlocked [M 0x1b0650].
  - The crossing legs (−11.0/−9.6, z −18.7) + the lane south + the ring legs to the hatch mouth;
    the LONG runs walk in segments with the watch between them (`walk_leg_watched`, arrive 1.3)
    after a boundary-only watch let an alarm runner land his press mid-leg at +446.
  - **`#[ignore]` REMOVED.**
- Test-initializer fixes for the new field: lock.rs's test `lock()`, gadget.rs's two, lock_sounds_live.rs, sound_coverage_live.rs (facing [0,0,1]).

## How it was verified (what I ran)

- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live -- --test-threads=1`:
  **7 passed / 0 failed / 2 ignored** (25.7 s) — the mission walk
  `walk_office_floor_qworm_keyboard_hatch` is UN-IGNORED and green: "walked to the hatch in
  15247 ticks; stuck 0; falls 0", `level_exit == Some(0x0700_000b)`, the voice plate and the
  laser-switch inertness asserts hold. The 2 remaining ignores are the two glide pins
  (unchanged, their own mechanisms).
- `cargo test --offline --release -p nf-game -- --test-threads=1`: **947 passed / 0 failed /
  9 ignored** (the lib unit tests + every live test binary).
- `interactables_0700000a_live` 7/0/0 and `keyboard_0700000a_live` 1/0/0 (the 0a use-path
  regressions).
- The keypad flow probed end-to-end in a scratch first: E arms lock 1, the digits 3-0-7-8
  solve, ch 0x30 rises, the door auto-opens, and the ray map at state 4 is clear.

## The FULL suite — honest status: NOT run to a total line this window

The driving message's background start (`nohup … &`) died instantly again (empty output, no
process — the same sandbox behaviour the -3/-2 reports documented). The chunked foreground
sweep (`chunk_suite.py`) did not fit the remaining window after the route/port/test work; the
affected crates were run directly instead (the totals above). **The next message should run
`chunk_suite.py` to ALL DONE on this tree and quote the total line** (state file fresh, e.g.
`/home/z/suite0a4_state.json`); the known environmental fails stay the 3 `disc_golden`, the 2
OOM binaries (mp_stress_explosives_live, mp_online_failures) and probe_pylon_render (no GPU).

## What's left

1. The FULL suite's chunked sweep + its total line (above).
2. The keypad MENU UI (GAP-1's presentation half) — the sim-side machine is ported and the walk
   drives the real handler; the page/cursor render is still open (the port never shows the
   keypad page).
3. The render checks (below) — the keypads' 0x2E placements are not in the port's dynamic draw
   list (the recorded lock.rs note), so the wall shows no keypad art.

## Live checks for the main agent

1. PCSX2, fresh 0700000a boot: stand at (−11.6, −17.5) facing the wall and press E — does the
   keypad page appear (MenuManager action 0x40000021)? Enter 3078 (this boot's code may differ —
   `Locks_Init` draws per boot): does door #724 (−10.6, −19.3) swing open? Watch `Lock_Activate`
   0x1aed00 (access: read; the lock obj from placement #648), the lock +0xF4 rise 0→3→4→1, and
   `switch_channels[48]` @0x26fc90+48 rise one tick after the match.
2. How does a REAL player learn the code? The Q-Worm install's reveal arm is kind-gated and #688
   is kind 0 [M 0x1aee20 + D placements], and the boot snapshot's shown flag is 0 [L @0x2beff8+6].
   Check the keypad page for a hint line (Txt_BindLabel 0x2000014 — the game may bind it
   unconditionally), and whether any level object (a monitor/placard) displays 4 digits.
3. Render checks (`--bond x,y,z,yaw,pitch`):
   - `--bond -11.9,1.3,-17.5,-1.57,0.3` — the keypad wall at (−10.69, 1.52, −17.46): today no
     keypad art draws (0x2E not in the dynamic draw list); the merge should confirm whether that
     is acceptable for this route or the draw needs its own task.
   - `--bond -11.0,1.3,-18.7,-0.0,0.0` — door #724's doorway open (state 4): the leaf parked
     clear, the atrium lane visible beyond.
   - `--bond 21.5,1.3,19.4,0.0,0.0` — the hatch mouth at the end of the walk.

## Usage

not visible to me (this environment exposes no token, cost or context-window figures to the agent).

# los-mask-join — report

**This branch moves the shared random stream** (guard sight changes which guards engage and
when, on every level). Keep it on its own batch and re-pin once.

Branch `s/los-mask-join` = master 3efb98f19 + `s/los-mask-envelope` (merge c98567732) + one
world.rs fix + walk repairs (test-only) + one exact re-pin.

## What a player sees

Guards now look with the game's own sight ray (`DroneVision_CanSeeObjectFrom` 0x176f18:69/95,
mask `param_6 | 0xa27`; `Collide_LineOfSight` 0x1edf50 ORs 4 at 0x1edf54 `ori t3,t1,4`). The
mask decides what a guard sees through:

| surface | guard sight (0xa27) | rule |
|---|---|---|
| plain walls, floors, crates | blocked | |
| glass (surface class 0x10), static or an intact pane | **sees through** | bit 0x2 [M `Collide_Intersect` 0x1ebaa8:314-316] |
| surfaces whose surface byte carries 0x40 (e.g. the 0b roof's parapet band at z 22.3, a 07000011 spawn-floor face) | **sees through** | bit 0x800 [M 0x1ebaa8:301-303] |
| surface classes 0xd / 0xe | sees through | bit 0x1 [M :311-313] |
| bullet-pass (node-flagged) geometry | blocked | 0xa27 has no bit 0x8 [M :317-319] |
| the 0x1000 player-missile cage | sees through | bit 0x20 [M 0x1eae80:72-75] |
| an intact non-glass pane (class 0xF) | blocked | picked like any object [M 0x1eae80:72-90] |
| a hidden object (its word carries 0x10) or one whose word carries 0x20 | sees through | [M 0x1eae80:72-82] |

So a guard behind a window, a glass door or a see-through parapet now spots Bond and opens
fire; guard rounds already pass glass by the player's rule (master), and Bond's pass the same
way. The same table drives the guard's re-check before firing and the other 0xa27 callers
(the hostage/peer checks, the surrender check).

Other masked rays on the same walk:
- Laser sensor cone and auto-aim (0xe): glass drops; a plain pane blocks.
- Sensor beam reach, searchlight, grapple, homing (0xc): a pane blocks unless its own word
  carries 0x40 (fixed here — see below).
- Crouch/jump headroom (0xd): bullet-pass geometry and 0x40/0x80 surfaces drop; glass blocks.

## Decompile check (done before the merge)

- `Collide_PickObj` 0x1eae80:57-90 read in full: the subject excludes (:57-59), the skip word
  (base 0x20; mask 0x20 adds 0x1000, 0x8 adds 0x40, 0x4 adds 0x10, :72-81), the type arms
  (:83-87). The port's `pick_admits` and the envelope's per-object admission match.
- `Collide_Intersect` 0x1ebaa8:93-99 and :300-319 read: the obj word's 0x40 sets HITDATA+0x51
  bit 2; 0x800 drops surface 0x40; 0x1 drops 0xd/0xe; 0x2 drops 0x10; 0x8 drops HITDATA+0x51
  bit 2 (surface 0x40/0x80 or the word). `Intersect_RayGeom` 0x10a8c8:326 stores the
  triangle's surface byte into HITDATA+0x50, the byte the port keeps in `mats`.
- `Collide_LineOfSight` 0x1edf50 disasm: the `| 4` is the instruction at 0x1edf54 (not a
  delay-slot artefact).
- Every guard sight/aim call site was checked against its game mask: `seek_opponent_los`
  (0x176f18, 0xa27), the surrender check (0x14a168), peer sight (0x1760b0), the explosives and
  vicinity callers (0x1435f0, 0x13a438), cover (0xa27), the turret track (0x72f, GT_Track
  0x17f9c8:258), the bullet-flash and auto-aim rays (mask 10 at 0x12d890:343-345 → 0xe).

## Fixed in this join (port bug)

`world.rs`: the envelope's masked walks OR'ed the dynamic set's baked `pass` bytes into the
mask-0x8 drop. Those bytes are set on every pane part (the bullet walks' stand-in), so every
mask-0x8 ray (the 0xc beam reach, 0xe auto-aim and sensor cone, 0xd headroom) passed through
every intact pane. The game picks a pane like any object and drops the hit only on the word's
0x40 or the surface's 0x40/0x80. The masked walks now read the pane's word (master's rule
before the envelope). `move_probe_clear` (the port's guard/route step probe, no game query)
keeps leaving pane parts out, as `segment_clear_dynamic` does, so guards still walk up to and
press the glass. Unit test `masked_rays_read_a_panes_glist_word_not_its_baked_pass` fails on
the envelope code (the 0xc beam assert) and passes here.

## The reds and what was done

Battery on the joined tree, then each red compared against the same tree with the masks off
(an env toggle used only while diagnosing, not committed). Every red below was green with the
masks off, so each is the masked vision's doing; the per-site masks are the game's, so these
are lost margin, re-routed as a player would. No ignores, no pins loosened, no damage or
health changed.

| walk | what changed under the masks | repair |
|---|---|---|
| 02 00 Agent | the stair leg out of the west-wing armour room wedged 4 × 3000 ticks (it does on the unmasked tree too) and a civilian died while Bond stood there | back out the corridor the detour came in by |
| 11 00 Agent | a drone sees Bond through a surface-0x40 face; a few ticks of drift lost the armour-room stand on a 1-round clip, then a loot take climbed back up a stair | reload before the armour-room door; the 27.1 strip scoops drops at Bond's feet (walker_b2 gains walker.rs's `reload_full`) |
| 15 00 Agent | the mine shots left the view at pitch -0.38, the E sphere hit the floor short of the exit plane | level the view before E |
| 13 Agent | the lobby guards engage more and earlier; the walk ran dry and died at the entrance | the 00 Agent pre-clears (spawn pocket, lobby, doorway) run at Agent too (ends 90.3 hp) |
| 12 00 Agent | the lobby drops were not reached and a drop behind the east desks wedged the resupply | the warm chain's 12 tactics: back-off 3.0 + lane fights |
| 08 Operative | the secure-room guard #4 hit Bond 28 times while he held at the closed door and the plain sight rule never answered; then in the Hind fight one rocket died 2 ticks after launch and the last one's blast finished Bond | the glass-fight rule, the secure-room armour, the Agent rocket sight gates (ends 71.2 + 40.4 armour) |
| 03 Agent | courtyard guards engage from the lane doors on; the walk ran dry and wedged at the door-8 mouth | the glass-fight rule (ends 38.0) |
| ai_live two-torso-shots | the second round lands on part 32, not 4 | exact pin `[(100,4,4.75),(130,32,-0.5)]` (the envelope had dropped the part from the pin; it is exact again) |

## Still red

`walkreal_0700000b_live` `..._real_damage_agent` and `..._real_damage_00agent`: the roof-deck
sprint to the parachute. The deck guards see the run across the parapet band (surface 0x40,
the losdbg trace: 5 guards, the hit plane z 22.3, x -4.6..7.8, y 58.4) and Bond dies at the
take slot (Agent: 21 hits for 100, the last 3 × 11.2 from #4 at 8 u). With the masks off the
same Agent run already ended at 27.2 hp, so the margin was thin. Tried: darting the idle deck
guards from the deck key before the sprint (no shot ever opened), and a sprint bent west via
(-12, 36) (#4 still reached Bond, death earlier at tick 15738). Not repaired.

## Battery (final tree)

All release, NF_SILENT, one binary per cargo call, on the final tree:

- walkreal: 01 7/0/1, 02 5/0, 03 6/0/2, 04 9/0/1, 05 16/0/1, 06 6/0/2, 07 5/0/1, 08 5/0/1,
  09 9/0, 0a 13/0/2, **0b 5/2** (the Agent and 00 Agent real-damage walks, above), 0c_barrelrow
  1/0, 0c_eastroute 1/0, 0c 3/0/1, 0d 1/0/2, 11 5/0, 11_route 1/0, 12 4/0, 13 3/0/1,
  14 7/0/3, 15 7/0/2, 16 15/0/3, 1b 16/0/3, 41 15/0, 41_wall 2/0, 4a 4/0, 4a_satchel 2/0
  (passed/failed/ignored; no ignore added).
- nf-game --lib 756/0/4 (includes the new pane test and the envelope's un-ignored mask test).
- ai_live 37/0, ai_missing_states_live 10/0, ai_points_live 4/0, cover_selection_live 2/0,
  dtype_sniper_alert_live 1/0, cameras_turrets_census_live 1/0, glass_pass_every_level_live
  3/0/3, gun_turrets_live 3/0, turret_hits_live 3/0.
- Not run: the full suite.

## Unproven

- Whether the 0x40-surface faces a guard now sees through (the 0b parapet band, the 07000011
  face) look see-through in the game. The rule is the decompile's; a live check of a 0b roof
  guard spotting Bond across the parapet would settle the player-visible part.
- Which sight call moves the 05 sniper's pose so the second round lands on part 32.

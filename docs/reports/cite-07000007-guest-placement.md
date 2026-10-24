# cite-07000007-guest-placement

A [D] citation for the 07000007 guest on the mission path. `civilian_mission_wait_live.rs`'s
`find_walker` asserted "07000007 places a guest on a kind-8 mission path (paths 2/4/5 node 0)"
with no evidence behind the parenthetical. I read the placement records and the path/node records
straight out of the disc image, cited them, replaced the hedge with the data, and added a sister
test that fails if the placement or the node moves.

## What the disc says

Archive and entry ([D], read through `nf_formats::disc::files_bin_entries` +
`levelbin::LevelBin`):

- `07000007.bin` = **FileList entry 34**, offset **0x8384800** into FILES.BIN, size **0xf55000**.
- Main map = **LevelBin entry 526, name `01000097`, kind 1**, data **0x722120..0xf3ef00** in the
  archive.

The AI-path block ([D]; block at entry-data offset 0x701940, size 0xc5c4, payload
**0xe23a64..0xe30024**; version 8 at 0xe23a64, path count **9** at 0xe23a68):

| AiNetwork idx | name (record abs) | kind | nodes | the port's `Network.paths` |
|---|---|---|---|---|
| 0 | Boundry (0xe23a6c) | 0x1 boundary | 748 | — (walls) |
| 1 | Main Path (0xe2d314) | 0x2 | 64 | [0] |
| 2 | Main Path (2) (0xe2e7f4) | 0x2 | 10 | [1] |
| 3 | **Party Patrol Path_1** (0xe2ebd4) | **0x8** | 5 | **[2]** |
| 4 | Rook Patrol Path_3 (0xe2ee04) | 0x8 | 15 | [3] |
| 5 | Party Patrol path_3 (0xe2f354) | 0x8 | 6 | [4] |
| 6 | Party Patrol path_4 (0xe2f5d4) | 0x8 | 6 | [5] |
| 7 | Party Patrol Path_3 (0xe2f854) | 0x8 | 11 | [6] |
| 8 | Party Patrol path_5 (0xe2fc64) | 0x8 | 10 | [7] |

Six kind-8 mission paths at AiNetwork 3..8 = `Network.paths[2..7]`, as the `assign_path` doc in
drone.rs already claimed. The single boundary (path 0) is what shifts AiNetwork 3 → the port's
`Network.paths[2]`.

Path 3 "Party Patrol Path_1": kind word **0x8 at 0xe2ec54**, node count **5 at 0xe2ec78**, extra
length 0 at 0xe2ec7c, link count 4 at 0xe2edc0. Node 0 record **0xe2ec80**, position bytes at
+0x10 = **0xe2ec90**: x `d3 e6 2b 42` (0x422be6d3), y `03 00 2c 42` (0x422c0003), z
`e9 7f 6d c2` (0xc26d7fe9) — (42.97, 43.00, −59.37) as on disc; the loader adds 0.4 to y
[M AIPath_Parse 0x1d2598: node+0x14 += 0.4].

The placement block ([D]; block at entry-data offset 0x80c87c, size 0xeee0, payload
**0xf2e9a0**; **513** placements; **35** carry param 0xf = 0xa — the dtype-10 party guests the
test header counts). The walker:

- **Placement 402, record 0xf382e0** (0x11c bytes = 0x4c + 26 params): object index field 272,
  hashcode 0xffffffff (by-index object), flags **0x0f** (dynamic drone), translation bits
  **0x422bda27 / 0x42305681 / 0xc26d8c1d** = (42.96, 44.08, −59.39), params **(2,1)** difficulty,
  **(3,0)** switch channel, **(0xf,0xa)** dtype 10.
- It stands **0.017 (2D)** from path-3 node 0 — inside the game's acceptance
  [M `NDrone2_AssignAIPath` 0x154190: fVar11 = 0.3].

## What was wrong, what I changed

The test's substance was right — a dtype-10, switch-0 guest does sit on a kind-8 node — but the
message's "paths 2/4/5" was a loose hedge. The path-4 and path-5 node-0 guests (placements 425 at
0.062 and 424 at 0.075) carry **switch channel 6**, so `find_walker`'s `switch_channel == 0` gate
excludes them: the walker is **path 2, node 0, placement 402**, and nothing else. The doc-name
citation ("[D, docs/WORKLOG fix-drone4]") is replaced by the offsets above.

Changes in `crates/nf-view/tests/civilian_mission_wait_live.rs`:

1. `find_walker`'s expect message now names the citation: placement 402 on
   `Network.paths[2]` (Party Patrol Path_1) node 0, with the record offsets.
2. Header and `find_walker` doc comments carry the [D] offsets (FileList 34, LevelBin 526, block
   payloads, the 35-guest count).
3. New sister `the_walker_is_placement_402_on_party_patrol_path_1_node_0` — the [D] pin, reading
   the disc itself through `nf_formats` (not through the port's sim):
   - FileList index/offset/size, main-map entry name/offset;
   - path block: 9 paths, path 0 is the boundary, AiNetwork path 3 name/kind/node-count;
   - node 0 x/y/z **bit-exact** (raw disc y asserted directly — `AiNetwork::parse` keeps the disc
     value; the +0.4 is a load step);
   - placement block: 513 records, placement 402's hashcode/flags/translation bits/params
     (difficulty 1, switch 0, dtype 10);
   - the runtime side: the port's `Network.paths[2]` is that path, its node 0 y carries the
     loader's +0.4 [M AIPath_Parse], `dr.placement == 402` spawns with dtype 10 / switch 0 and
     stands within the 0.3 acceptance [M AssignAIPath 0x154190];
   - uniqueness: no other dtype-10 switch-0 guest is inside the acceptance of any kind-8 node
     (next closest: placement 349 at 3.06 [D]).

It fails if the node record moves (bit asserts), if the placement record moves (translation-bit
assert), if the pairing drifts (acceptance assert), or if a different node index became the
closest.

## Verification

- `cargo test --offline --release -p nf-view --test civilian_mission_wait_live --
  --test-threads=1`: **3 passed** (the two pre-existing wait tests green on the updated file, the
  new sister green).
- Full four-crate suite (`python3 tools/test/suite.py`, per-binary time limit) on this tree; the
  total line is quoted below once it finishes.

## Live checks for the main agent

None needed — the disc bytes and the port's sim agree, and the pin is byte-exact on both sides.
Optional sanity render if you want eyes on it: `--bond 40,45,-45,1.2,0.05` looking north-west
should show the guest walking the corridor between the ballroom and the bar (Party Patrol Path_1
nodes 0→4 run (42.98,−59.37) → (42.91,−77.29)).

## Usage

- Input tokens / output tokens / context window used / cost: not visible to me (this sandbox
  exposes no token, credit or context figures to copy).

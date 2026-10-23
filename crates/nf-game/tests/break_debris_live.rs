//! The kill pieces through the real level data (a breakable shatters into smaller
//! pieces). A shot vase spawns the game's debris — one object per
//! stride-3 collision record of every leaf box [M Debris_Create_Loop 0x1ed548 + disasm
//! 0x1ed638], dressed by EffectInfo[hit material & 0x3f] [D ACTION.ELF 0x3037c0] with the
//! game's launch numbers (velocity = spread·0.5 · record plane normal · REC_FRAME_RATE, the
//! hit-velocity term zeroed by Break_Kill [M 0x12b970:140, Debris_Create 0x1aaa00:38-49]); a
//! shot window spawns one shard per small source triangle (psiCreateBreakableData's draw-model
//! faces [M 0x117418], ShatterPolyRecurse's leaf rule [M 0x12b270:156-162]); the sister: an
//! unshot breakable (and a missed shot) spawns nothing. Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::breakable::Breaks;
use nf_game::drone::Rng;
/// `REC_FRAME_RATE` [L grenade.rs:148 = 1/60 in the lib].
const REC_FRAME_RATE: f32 = 1.0 / 60.0;
use nf_game::shatter::EFFECT_INFO;

fn level(id: u32) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let name = format!("0700{:04}.bin", id);
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))?;
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).ok()
}

/// The game's stride-3 record count over a collision block, computed here from the raw parsed
/// data (independent of the port's `Source` build): per leaf box, `i` from `tris.start` while
/// `i < tris.end` step 3 [M disasm 0x1ed5f8-0x1ed838].
fn stride3_count(level: &Level, object: (usize, usize)) -> usize {
    let c = level.maps[object.0].objects[object.1].collision.as_ref().unwrap();
    c.boxes.iter().filter(|b| b.children.is_none())
        .map(|b| (b.tris.end - b.tris.start).div_ceil(3) as usize)
        .sum()
}

#[test]
fn a_shot_vase_spawns_the_games_debris_count_and_launch_numbers() {
    let Some(level) = level(1) else { return };
    let mut br = Breaks::create(&level);
    if br.list.is_empty() { panic!("07000001 carries breakables [D census]"); }
    assert!(br.shatter.debris.is_empty() && br.shatter.shards.is_empty(),
        "sister: nothing is spawned before any shot");
    // The vases: kind 0 with a collision block (the game's kill needs cel+8 [M 0x12b970:59-63]).
    let Some((i, _)) = br.list.iter().enumerate()
        .find(|(_, b)| b.kind == 0 && !b.src.records.is_empty()) else { panic!("a kind-0 vase with collision on 07000001 [D]") };
    let (lo, hi) = (br.list[i].lo, br.list[i].hi);
    let object = if br.list[i].placement != usize::MAX {
        let p = &level.maps[level.main_map().unwrap()].placements[br.list[i].placement];
        if p.by_hash() { level.by_hash[&p.hashcode] } else { (level.main_map().unwrap(), p.index as usize) }
    } else { unreachable!() };
    let count = stride3_count(&level, object);
    assert!(count >= 1, "the vase's leaves carry records");
    // One bullet through the bounds centre kills it.
    let a = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, hi[2] + 2.0];
    let b = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2] - 2.0];
    let mut rng = Rng::new(0x5eed);
    let kills = br.bullet_hit(a, b, 0x1e, 25.0, &mut rng);
    assert_eq!(kills.len(), 1, "health 0.0 [D placements] — the first bullet kills");
    assert_eq!(br.shatter.debris.len(), count,
        "one debris object per stride-3 record [M 0x1ed548; count recomputed from the disc data]");
    assert!(br.shatter.shards.is_empty(), "kind 0 takes the debris arm, not the shatter [M 0x12b970:78]");
    // The launch numbers of every piece [M Debris_Create 0x1aaa00:30-50]. The row follows the
    // hit's material; rows share models (0x0200012d at 15 AND 16), so the speed may match any
    // row with the piece's model — the model hash itself is the [D] pin.
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    for (k, d) in br.shatter.debris.iter().enumerate() {
        let speed = (d.vel[0] * d.vel[0] + d.vel[1] * d.vel[1] + d.vel[2] * d.vel[2]).sqrt();
        let rows: Vec<_> = EFFECT_INFO.iter().filter(|r| r.model == d.model).collect();
        assert!(!rows.is_empty(), "piece {k} model 0x{:08x} is a table row [D 0x3037c0]", d.model);
        assert!(rows.iter().any(|r| (speed - r.spread * 0.5 * REC_FRAME_RATE).abs() <= r.spread * 0.5 * REC_FRAME_RATE * 0.05),
            "piece {k}: |vel| = {speed} ≈ spread·0.5·REC (the hit-velocity term is zeroed [M 0x12b970:140])");
        let row = rows.iter().find(|r| (speed - r.spread * 0.5 * REC_FRAME_RATE).abs() <= r.spread * 0.5 * REC_FRAME_RATE * 0.05).unwrap();
        let half = (row.life_base >> 1) as i32;
        assert!(d.life >= half && d.life < half * 2, "piece {k}: life = (base>>1) + Rand_Rand(base>>1) [M 0x1aaa00:34-37]");
        assert_eq!(d.fade, 0xff, "obj+0x106 = 0xff at create [M 0x1aaa00:31]");
        assert!(d.spin.iter().all(|s| s.abs() <= row.spin / 2.0 + 1.0e-4),
            "piece {k}: spin within ±EffectInfo+0x7c/2 [M 0x1dfef0]");
        // The centre sits on the object (the leaf verts + obj position [M 0x1ed548:126-131]).
        assert!(d.pos.iter().zip(centre).all(|(p, c)| (p - c).abs() <= 3.0),
            "piece {k} at {:?} near the object centre {center:?}", d.pos, center = centre);
    }
    // The eight shared draws per piece [M 0x1aaa00:30-50]: a fresh kill on a twin vase draws 8×count.
    let mut br2 = Breaks::create(&level);
    let j = br2.list.iter().enumerate().find(|(_, b)| b.kind == 0 && !b.src.records.is_empty() && !b.dead).map(|(j, _)| j).unwrap();
    let (lo, hi) = (br2.list[j].lo, br2.list[j].hi);
    let a = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, hi[2] + 2.0];
    let b = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2] - 2.0];
    let mut rng = Rng::new(0x5eed);
    let before = rng.draws;
    br2.bullet_hit(a, b, 0x1e, 25.0, &mut rng);
    let n = br2.shatter.debris.len();
    assert_eq!(rng.draws - before, 8 * n as u64, "eight draws per debris object [M 0x1aaa00:30-50]");
}

#[test]
fn a_shot_window_spawns_a_shard_per_source_triangle_and_a_miss_spawns_none() {
    let Some(level) = level(1) else { return };
    let mut br = Breaks::create(&level);
    // The disc's panes carry few large triangles (max edge² 3.9–11 [D probe 2026-10-20]), so a
    // real shot SPLITS them (ShatterPolyRecurse recurses while the longest edge² exceeds
    // clamp(half·distSq, 0.5, 100) [M 0x12b270:156-162]) — every source triangle yields at
    // least one shard, and the pieces tile the pane.
    let Some(i) = br.list.iter().enumerate().find(|(_, b)| {
        b.kind == 2 && !b.src.records.is_empty() && b.src.glass.len() >= 2
    }).map(|(i, _)| i) else { panic!("a kind-2 pane on 07000001 [D — 259 kind-2 placements on the disc]") };
    let src_count = br.list[i].src.glass.len();
    let (lo, hi) = (br.list[i].lo, br.list[i].hi);
    let a = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, hi[2] + 2.0];
    let b = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2] - 2.0];
    let mut rng = Rng::new(0x5eed);
    let kills = br.bullet_hit(a, b, 0x1e, 25.0, &mut rng);
    assert_eq!(kills.len(), 1);
    assert_eq!(kills[0].sound, 0x2E, "glass → 0x2E [M 0x12b970:65]");
    assert!(br.shatter.shards.len() >= src_count,
        "every source triangle yields at least one shard (splits only add) [M 0x12b970:79-131]");
    assert!(br.shatter.debris.is_empty(), "the glass arm spawns no debris [M 0x12b970:78-133]");
    let g = 1.0e-3;
    for s in &br.shatter.shards {
        assert!((s.lifetime - REC_FRAME_RATE * 25.0).abs() < 1.0e-6, "lifetime = REC·25 [M 0x12b270:205-209]");
        assert!(s.delay >= 0.0, "the peel delay = dist·REC·15 [M 0x12b270:190-192]");
        assert_eq!(s.rot, [0.0, 0.0, 0.0], "rot starts 0 [M 0x12b270:186-191]");
        for v in &s.verts {
            assert!((lo[0] - g..=hi[0] + g).contains(&(v.pos[0] + s.pos[0]))
                && (lo[1] - g..=hi[1] + g).contains(&(v.pos[1] + s.pos[1]))
                && (lo[2] - g..=hi[2] + g).contains(&(v.pos[2] + s.pos[2])),
                "the shards tile the pane's bounds [M 0x12b270]");
        }
    }
    // The pieces tick away on the game's timer: delay (dist·15 ticks) + lifetime (25 ticks).
    let mut ticks = 0;
    while !br.shatter.shards.is_empty() {
        br.shatter.update();
        ticks += 1;
        assert!(ticks < 300, "the shards expire on the game's timer [M 0x12b808]");
    }
    assert!(ticks >= 25, "each shard lives its 25 active ticks after its peel delay [M 0x12b808:52-83]");
    // Sister: a missed shot spawns nothing.
    let mut br = Breaks::create(&level);
    let mut rng = Rng::new(0x5eed);
    let kills = br.bullet_hit([1.0e5, 1.0e5, 1.0e5], [1.0e5, 1.0e5, 2.0e5], 0x1e, 25.0, &mut rng);
    assert!(kills.is_empty());
    assert!(br.shatter.shards.is_empty() && br.shatter.debris.is_empty(),
        "an unshot/missed breakable spawns nothing");
}

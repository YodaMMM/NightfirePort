//! 07000005: BFS over the game's own nav network from the wire landing to the zip door —
//! the AI's walkable-space graph is the map of the route. Scratch print, not committed.

use nf_formats::iso9660::IsoImage;

fn main() {
    let img = IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
    let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0005);

    // Flatten: (path, node) -> global id.
    let mut ids = Vec::new();
    let mut adj: Vec<Vec<usize>> = Vec::new();
    let mut pos: Vec<[f32; 3]> = Vec::new();
    for p in nav.paths.iter() {
        let base = pos.len();
        for n in p.nodes.iter() { pos.push(n.pos); adj.push(Vec::new()); }
        for l in p.links.iter() {
            let (a, b) = (l.a as usize, l.b as usize);
            if a < p.nodes.len() && b < p.nodes.len() {
                adj[base + a].push(base + b);
                adj[base + b].push(base + a);
            }
        }
        ids.push(p.kind);
    }
    eprintln!("paths {} total nodes {}", nav.paths.len(), pos.len());

    let near = |q: [f32; 3], r: f32| -> Vec<usize> {
        (0..pos.len()).filter(|&i| (pos[i][0]-q[0]).powi(2) + (pos[i][1]-q[1]).powi(2) + (pos[i][2]-q[2]).powi(2) < r*r).collect()
    };
    let starts = near([-23.5, 30.0, 4.0], 25.0);
    let goals = near([37.25, 49.0, -69.0], 30.0);
    eprintln!("start candidates {:?} goal candidates {:?}", starts.iter().map(|&i| (i, pos[i])).collect::<Vec<_>>(),
        goals.iter().map(|&i| (i, pos[i])).collect::<Vec<_>>());

    // BFS from all starts.
    let mut prev = vec![usize::MAX; pos.len()];
    let mut q = std::collections::VecDeque::new();
    for &s in &starts { prev[s] = s; q.push_back(s); }
    while let Some(u) = q.pop_front() {
        for &v in &adj[u] { if prev[v] == usize::MAX { prev[v] = u; q.push_back(v); } }
    }
    for &g in &goals {
        if prev[g] == usize::MAX { eprintln!("node {g} {:?} UNREACHABLE from the landing", pos[g]); continue; }
        let mut chain = vec![g];
        let mut u = g;
        while prev[u] != u { u = prev[u]; chain.push(u); }
        chain.reverse();
        eprintln!("== route to node {g} ({} hops) ==", chain.len() - 1);
        for (k, &i) in chain.iter().enumerate() {
            eprintln!("  {k:3} n{i} ({:.2},{:.2},{:.2})", pos[i][0], pos[i][1], pos[i][2]);
        }
    }
    // The component containing the goal nodes — how far does the door's chain reach?
    let mut comp = std::collections::BTreeSet::new();
    let mut stack: Vec<usize> = goals.clone();
    for &g in &goals { comp.insert(g); }
    while let Some(u) = stack.pop() {
        for &v in &adj[u] { if comp.insert(v) { stack.push(v); } }
    }
    eprintln!("== the door component: {} nodes ==", comp.len());
    let mut v: Vec<_> = comp.iter().copied().collect();
    v.sort_by_key(|&i| (pos[i][2] * 10.0) as i64);
    for i in v {
        eprintln!("  n{:3} ({:7.2},{:6.2},{:8.2}) deg {}", i, pos[i][0], pos[i][1], pos[i][2], adj[i].len());
    }
}

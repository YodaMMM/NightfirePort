"""calltree.py — what a feature's game code reaches that the port never mentions.

Usage: calltree.py <decomp dir> <ACTION.ELF> <repo dir> <entry name or 0xaddr> [...]
       calltree.py ... --all-mentioned-dstates   (every DSTATE handler the port mentions)

Edges: direct calls (Name__F...( in the decompiled C) plus state changes — a
Drone_SM_SetState / SendMsg-to-state call whose 2nd argument is a constant maps through
NDrone2_StateFuncs (0x29b300) to that state's handler. 'Mentioned' = the function's name or
address appears anywhere in crates/*/src (not tests). Mentioned is NOT ported-complete.
"""
import os, re, sys, struct
dec, elfp, repo = sys.argv[1], sys.argv[2], sys.argv[3]
args = sys.argv[4:]
funcs = {}; byname = {}; bodies = {}
for f in os.listdir(dec):
    m = re.match(r'([0-9a-f]{8})_(.+)\.c$', f)
    if not m: continue
    a = int(m.group(1), 16); full = m.group(2); base = full.split('__')[0]
    funcs[a] = base; byname.setdefault(base, a); byname.setdefault(full, a)
    bodies[a] = open(os.path.join(dec, f), encoding='utf-8', errors='ignore').read()
elf = open(elfp, 'rb').read()
phoff = struct.unpack_from('<I', elf, 0x1c)[0]; phn = struct.unpack_from('<H', elf, 0x2c)[0]
segs = []
for i in range(phn):
    t, off, va, pa, fs, ms = struct.unpack_from('<6I', elf, phoff + i * 32)
    if t == 1: segs.append((va, off, fs))
def rd(va):
    for s, off, fs in segs:
        if s <= va < s + fs: return struct.unpack_from('<I', elf, off + va - s)[0]
    return None
STATE_TABLE = 0x29b300
states = {}
for i in range(0x100):
    p = rd(STATE_TABLE + 4 * i)
    if p is None or p not in funcs: continue
    states[i] = p
text = ''
for root, _, fs in os.walk(os.path.join(repo, 'crates')):
    if 'tests' in root.split(os.sep): continue
    for f in fs:
        if f.endswith('.rs'): text += open(os.path.join(root, f), encoding='utf-8', errors='ignore').read()
addrs = set(int(x, 16) for x in re.findall(r'0x([0-9a-fA-F]{5,6})\b', text))
words = set(re.findall(r'[A-Za-z_][A-Za-z0-9_]{5,}', text))
def mentioned(a): return a in addrs or funcs[a] in words
call_re = re.compile(r'\b([A-Za-z_][A-Za-z0-9_]*__F[A-Za-z0-9_]*|FUN_[0-9a-f]{8})\s*\(')
state_re = re.compile(r'Drone_SM_SetState__[A-Za-z0-9_]*\s*\([^,()]*(?:\([^()]*\))?[^,()]*,\s*(0x[0-9a-fA-F]+|\d+)')
def edges(a):
    b = bodies[a]; out = set()
    for m in call_re.finditer(b):
        n = m.group(1)
        t = byname.get(n) or byname.get(n.split('__')[0])
        if n.startswith('FUN_'): t = int(n[4:], 16) if int(n[4:], 16) in funcs else None
        if t is not None and t != a: out.add(t)
    for m in state_re.finditer(b):
        sid = int(m.group(1), 0)
        if sid in states: out.add(states[sid])
    return out
SKIP = re.compile(r'^(gcc2|sce|Sif|Dma|DMA|Ps2|PS2|ps2|psi|Mem_|MemCard|Iop|IOP|Pad|Gs|libc|str|mem|printf|sprintf|malloc|free|Rand_|Vec_|Mat_|Quat_|Vu|VU|Spu|SPU|Kernel|Thread|Sema|View_|Sound_Play|Text_|Txt_|Font_|hashtable_)')
def walk(entry):
    seen = set([entry]); stack = [entry]; missing = []
    while stack:
        a = stack.pop()
        for t in edges(a):
            if t in seen: continue
            seen.add(t)
            if SKIP.match(funcs[t]): continue
            if not mentioned(t): missing.append(t)
            stack.append(t)
    return seen, missing
def resolve(x):
    if x.startswith('0x'): return int(x, 16)
    return byname[x]
entries = []
if '--all-mentioned-dstates' in args:
    entries = sorted(a for a in funcs if 'DSTATE' in funcs[a] and mentioned(a))
else:
    entries = [resolve(x) for x in args]
total_missing = set()
for e in entries:
    seen, missing = walk(e)
    total_missing |= set(missing)
    if '--all-mentioned-dstates' in args and not missing: continue
    print('%s 0x%06x: reaches %d, never mentioned %d' % (funcs[e], e, len(seen), len(missing)))
    if '--all-mentioned-dstates' not in args or len(missing) <= 12:
        for m in sorted(missing): print('    0x%06x %s' % (m, funcs[m]))
print('TOTAL distinct never-mentioned reachable: %d' % len(total_missing))
if '--list' in args:
    for m in sorted(total_missing): print('MISSING 0x%06x %s' % (m, funcs[m]))

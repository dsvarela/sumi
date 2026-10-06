"""Quick-and-dirty human-style grader using sumi's v1 technique list.
Reads 81-char puzzles, applies easiest available step repeatedly, reports hardest technique.
Experimental only; not reference code."""
import sys
from itertools import combinations
from multiprocessing import Pool
from collections import Counter

ROWS = [[r * 9 + c for c in range(9)] for r in range(9)]
COLS = [[r * 9 + c for r in range(9)] for c in range(9)]
BOXES = [[(br * 3 + i) * 9 + bc * 3 + j for i in range(3) for j in range(3)] for br in range(3) for bc in range(3)]
UNITS = ROWS + COLS + BOXES
ROW = [i // 9 for i in range(81)]
COL = [i % 9 for i in range(81)]
BOX = [(i // 27) * 3 + (i % 9) // 3 for i in range(81)]
PEERS = [set() for _ in range(81)]
for u in UNITS:
    for a in u:
        PEERS[a].update(x for x in u if x != a)
BIT = [1 << d for d in range(9)]
def pc(m): return bin(m).count("1")
def digits(m): return [d for d in range(9) if m >> d & 1]

class G:
    def __init__(self, s):
        self.v = [0] * 81
        self.c = [0x1FF] * 81
        for i, ch in enumerate(s[:81]):
            if ch in "123456789":
                self.place(i, int(ch) - 1)
    def place(self, i, d):
        self.v[i] = d + 1
        self.c[i] = 0
        for p in PEERS[i]:
            self.c[p] &= ~BIT[d]
    def elim(self, cells, d):
        ch = False
        for i in cells:
            if self.c[i] & BIT[d]:
                self.c[i] &= ~BIT[d]; ch = True
        return ch
    def pos(self, unit, d):
        return [i for i in unit if self.c[i] & BIT[d]]

def full_house(g):
    for u in UNITS:
        e = [i for i in u if not g.v[i]]
        if len(e) == 1 and g.c[e[0]]:
            g.place(e[0], digits(g.c[e[0]])[0]); return True
    return False

def naked_single(g):
    for i in range(81):
        if not g.v[i] and pc(g.c[i]) == 1:
            g.place(i, digits(g.c[i])[0]); return True
    return False

def hidden_single(g):
    for u in UNITS:
        for d in range(9):
            p = g.pos(u, d)
            if len(p) == 1:
                g.place(p[0], d); return True
    return False

def locked(g):
    for b in range(9):
        for d in range(9):
            p = g.pos(BOXES[b], d)
            if len(p) < 2: continue
            rs = {ROW[i] for i in p}; cs = {COL[i] for i in p}
            if len(rs) == 1 and g.elim([i for i in ROWS[rs.pop()] if BOX[i] != b], d): return True
            if len(cs) == 1 and g.elim([i for i in COLS[cs.pop()] if BOX[i] != b], d): return True
    for u in ROWS + COLS:
        for d in range(9):
            p = g.pos(u, d)
            if len(p) < 2: continue
            bs = {BOX[i] for i in p}
            if len(bs) == 1 and g.elim([i for i in BOXES[bs.pop()] if i not in u], d): return True
    return False

def naked(n):
    def f(g):
        for u in UNITS:
            e = [i for i in u if g.c[i] and pc(g.c[i]) <= n]
            for combo in combinations(e, n):
                m = 0
                for i in combo: m |= g.c[i]
                if pc(m) == n:
                    others = [i for i in u if i not in combo]
                    if any(g.elim(others, d) for d in digits(m)): return True
        return False
    return f

def hidden(n):
    def f(g):
        for u in UNITS:
            ds = [d for d in range(9) if 2 <= len(g.pos(u, d)) <= n]
            for combo in combinations(ds, n):
                cells = set()
                for d in combo: cells.update(g.pos(u, d))
                if len(cells) == n:
                    keep = sum(BIT[d] for d in combo)
                    ch = False
                    for i in cells:
                        if g.c[i] & ~keep:
                            g.c[i] &= keep; ch = True
                    if ch: return True
        return False
    return f

def fish(n):
    def f(g):
        for base, cover, idx in ((ROWS, COLS, COL), (COLS, ROWS, ROW)):
            for d in range(9):
                lines = [(k, g.pos(base[k], d)) for k in range(9)]
                lines = [(k, p) for k, p in lines if 2 <= len(p) <= n]
                for combo in combinations(lines, n):
                    cov = set()
                    for _, p in combo: cov.update(idx[i] for i in p)
                    if len(cov) == n:
                        bset = {set_k for set_k, _ in combo}
                        cells = [i for c in cov for i in cover[c] if (ROW[i] if base is ROWS else COL[i]) not in bset]
                        if g.elim(cells, d): return True
        return False
    return f

def conj_pairs(g, d, units):
    out = []
    for u in units:
        p = g.pos(u, d)
        if len(p) == 2: out.append(tuple(p))
    return out

def sees(a, b): return b in PEERS[a]

def skyscraper(g):
    for units, idx in ((ROWS, COL), (COLS, ROW)):
        for d in range(9):
            pr = conj_pairs(g, d, units)
            for (a1, a2), (b1, b2) in combinations(pr, 2):
                for (x, rx), (y, ry) in (((a1, a2), (b1, b2)), ((a1, a2), (b2, b1)), ((a2, a1), (b1, b2)), ((a2, a1), (b2, b1))):
                    if idx[x] == idx[y] and idx[rx] != idx[ry]:
                        cells = [i for i in PEERS[rx] & PEERS[ry] if i not in (x, y)]
                        if g.elim(cells, d): return True
    return False

def kite(g):
    for d in range(9):
        rp = conj_pairs(g, d, ROWS); cp = conj_pairs(g, d, COLS)
        for r in rp:
            for c in cp:
                for rb, re in ((r[0], r[1]), (r[1], r[0])):
                    for cb, ce in ((c[0], c[1]), (c[1], c[0])):
                        if rb != cb and BOX[rb] == BOX[cb] and len({rb, re, cb, ce}) == 4 and BOX[re] != BOX[rb] and BOX[ce] != BOX[cb]:
                            t = ROW[ce] * 9 + COL[re]
                            if g.elim([t], d): return True
    return False

def bivalues(g): return [i for i in range(81) if pc(g.c[i]) == 2]

def xywing(g):
    bv = bivalues(g)
    for p in bv:
        x, y = digits(g.c[p])
        wings = [w for w in bv if w != p and sees(p, w)]
        for a in wings:
            if g.c[a] & BIT[x] and not g.c[a] & BIT[y]:
                z = digits(g.c[a] & ~BIT[x])[0]
                for b in wings:
                    if b != a and g.c[b] == (BIT[y] | BIT[z]):
                        if g.elim([i for i in PEERS[a] & PEERS[b] if i != p], z): return True
    return False

def xyzwing(g):
    bv = bivalues(g)
    for p in range(81):
        if pc(g.c[p]) != 3: continue
        wings = [w for w in bv if sees(p, w) and g.c[w] & ~g.c[p] == 0]
        for a, b in combinations(wings, 2):
            if g.c[a] | g.c[b] == g.c[p] and g.c[a] != g.c[b]:
                z = digits(g.c[a] & g.c[b])[0]
                if g.elim([i for i in PEERS[a] & PEERS[b] & PEERS[p]], z): return True
    return False

def wwing(g):
    bv = bivalues(g)
    for a, b in combinations(bv, 2):
        if g.c[a] != g.c[b] or sees(a, b): continue
        for x in digits(g.c[a]):
            y = digits(g.c[a] & ~BIT[x])[0]
            for e1, e2 in conj_pairs(g, x, UNITS):
                if {e1, e2} & {a, b}: continue
                if (sees(e1, a) and sees(e2, b)) or (sees(e1, b) and sees(e2, a)):
                    if g.elim(list(PEERS[a] & PEERS[b]), y): return True
    return False

def colouring(g):
    for d in range(9):
        adj = {}
        for a, b in conj_pairs(g, d, UNITS):
            adj.setdefault(a, set()).add(b); adj.setdefault(b, set()).add(a)
        seen = {}
        for s in adj:
            if s in seen: continue
            comp = {s: 0}; st = [s]
            while st:
                x = st.pop()
                for y in adj[x]:
                    if y not in comp:
                        comp[y] = 1 - comp[x]; st.append(y)
            seen.update(comp)
            if len(comp) < 3: continue
            cls = [[i for i, k in comp.items() if k == col] for col in (0, 1)]
            for col in (0, 1):  # wrap
                if any(sees(a, b) for a, b in combinations(cls[col], 2)):
                    if g.elim(cls[col], d): return True
            # trap
            cells = [i for i in range(81) if g.c[i] & BIT[d] and i not in comp
                     and any(sees(i, a) for a in cls[0]) and any(sees(i, b) for b in cls[1])]
            if g.elim(cells, d): return True
    return False

def ur1(g):
    for r1, r2 in combinations(range(9), 2):
        for c1, c2 in combinations(range(9), 2):
            cells = [r1 * 9 + c1, r1 * 9 + c2, r2 * 9 + c1, r2 * 9 + c2]
            if len({BOX[i] for i in cells}) != 2: continue
            if any(g.v[i] for i in cells): continue
            bvs = [i for i in cells if pc(g.c[i]) == 2]
            if len(bvs) != 3 or len({g.c[i] for i in bvs}) != 1: continue
            m = g.c[bvs[0]]
            t = [i for i in cells if i not in bvs][0]
            if g.c[t] & m == m:
                g.c[t] &= ~m; return True
    return False

ORDER = [
    ("Full House", 0, full_house), ("Naked Single", 0, naked_single), ("Hidden Single", 0, hidden_single),
    ("Locked Candidates", 1, locked), ("Naked Pair", 1, naked(2)), ("Hidden Pair", 1, hidden(2)),
    ("Naked Triple", 2, naked(3)), ("Hidden Triple", 2, hidden(3)), ("Naked Quad", 2, naked(4)), ("Hidden Quad", 2, hidden(4)),
    ("Skyscraper", 3, skyscraper), ("2-String Kite", 3, kite), ("X-Wing", 3, fish(2)), ("Simple Colouring", 3, colouring),
    ("XY-Wing", 3, xywing), ("XYZ-Wing", 3, xyzwing), ("W-Wing", 3, wwing), ("Swordfish", 3, fish(3)), ("UR Type 1", 3, ur1),
]
IDX = {n: k for k, (n, _, _) in enumerate(ORDER)}
LEVELS = ["Easy", "Medium", "Hard", "Expert", "Unsolved"]

def grade(s):
    g = G(s)
    used = Counter(); lvl = 0; hardest = "Full House"
    while not all(g.v):
        for name, L, f in ORDER:
            if f(g):
                used[name] += 1
                if L > lvl or (L == lvl and IDX[name] > IDX[hardest]): lvl, hardest = L, name
                break
        else:
            return s, 4, "stuck", used
    return s, lvl, hardest, used

if __name__ == "__main__":
    puzzles = [l.strip() for l in open(sys.argv[1]) if len(l.strip()) == 81]
    with Pool() as pool:
        res = pool.map(grade, puzzles, chunksize=20)
    lv = Counter(r[1] for r in res); hd = Counter(r[2] for r in res)
    n = len(res)
    print("N =", n)
    for k in range(5): print(f"{LEVELS[k]:9s} {lv[k]:6d} {100*lv[k]/n:6.2f}%")
    print("hardest technique:")
    for k, v in hd.most_common(): print(f"  {k:18s} {v:6d} {100*v/n:6.2f}%")
    exp = [r for r in res if r[1] == 3]
    cnt = Counter(sum(v for k, v in r[3].items() if dict((a, b) for a, b, _ in ORDER)[k] == 3) for r in exp)
    print("Expert puzzles: number of expert steps:", sorted(cnt.items()))
    if len(sys.argv) > 2:
        with open(sys.argv[2], "w") as fo:
            for s, L, h, _ in res: fo.write(f"{s} {LEVELS[L]} {h}\n")

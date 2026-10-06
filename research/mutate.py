import random, sys, time
from multiprocessing import Pool
from collections import Counter
from grader import G, grade, pc, digits, LEVELS
def count(s, limit=2):
    g=G(s); n=0
    if any(not g.v[i] and g.c[i]==0 for i in range(81)): return 0
    def dfs(g):
        nonlocal n
        best=None
        for i in range(81):
            if not g.v[i]:
                if g.c[i]==0: return
                if best is None or pc(g.c[i])<pc(g.c[best]): best=i
        if best is None: n+=1; return
        for d in digits(g.c[best]):
            h=G.__new__(G); h.v=g.v[:]; h.c=g.c[:]; h.place(best,d); dfs(h)
            if n>=limit: return
    dfs(g); return n
def solution(s):
    g=G(s)
    def dfs(g):
        best=None
        for i in range(81):
            if not g.v[i]:
                if g.c[i]==0: return None
                if best is None or pc(g.c[i])<pc(g.c[best]): best=i
        if best is None: return g.v[:]
        for d in digits(g.c[best]):
            h=G.__new__(G); h.v=g.v[:]; h.c=g.c[:]; h.place(best,d); r=dfs(h)
            if r: return r
    return dfs(g)
def neighbour(args):
    s, seed = args
    rnd=random.Random(seed); sol=solution(s)
    for _ in range(50):
        p=list(s)
        clues=[i for i in range(81) if p[i]!='.']; holes=[i for i in range(81) if p[i]=='.']
        i=rnd.choice(clues); j=rnd.choice(holes)
        p[i]='.'; p[j]=str(sol[j])   # shift a clue
        # minimise: try removing each clue once in random order
        q=''.join(p)
        if count(q)!=1: continue
        order=[k for k in range(81) if q[k]!='.']; rnd.shuffle(order)
        q=list(q)
        for k in order:
            if k==j: continue
            c=q[k]; q[k]='.'
            if count(''.join(q))!=1: q[k]=c
        q=''.join(q)
        return grade(q)[1:3]
    return None
if __name__=='__main__':
    target=sys.argv[1]
    seeds=[l.split()[0] for l in open('qq20k.graded') if l.split()[1]==target][:int(sys.argv[2])]
    t=time.time()
    with Pool() as pool:
        res=pool.map(neighbour, [(s,k) for s in seeds for k in range(int(sys.argv[3]))])
    res=[r for r in res if r]
    c=Counter(LEVELS[r[0]] for r in res)
    print(target, 'seeds',len(seeds),'neighbours',len(res), dict(c), f'{time.time()-t:.1f}s')
    print(Counter(r[1] for r in res if LEVELS[r[0]]==target).most_common())

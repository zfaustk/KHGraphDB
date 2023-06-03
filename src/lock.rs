//! Entity lock table. Shared / exclusive. Wait-for
//! is a graph. A cycle is deadlock; the requester
//! aborts. There is no latch on a page. The key is
//! a Khid. One writer still owns the arena; this
//! table is how two logical transactions refuse to
//! step on the same vertex.
//!
//! Textbook 2PL. Not a promise of multi-writer
//! mutation of the live Graph. The engine takes
//! these locks, then the store.

use std::collections::{HashMap, HashSet, VecDeque};

use super::khid::Khid;

pub type TxId = u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    S,
    X,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Acquire {
    Ok,
    Wait,
    Deadlock,
}

struct Grant {
    x: Option<TxId>,
    s: HashSet<TxId>,
    wait: VecDeque<(TxId, Mode)>,
}

impl Grant {
    fn empty() -> Grant {
        Grant {
            x: None,
            s: HashSet::new(),
            wait: VecDeque::new(),
        }
    }

    fn holders(&self) -> HashSet<TxId> {
        let mut h = self.s.clone();
        if let Some(x) = self.x {
            h.insert(x);
        }
        h
    }

    fn compatible(&self, tx: TxId, m: Mode) -> bool {
        match m {
            Mode::S => {
                if let Some(x) = self.x {
                    x == tx
                } else {
                    true
                }
            }
            Mode::X => {
                if let Some(x) = self.x {
                    x == tx
                } else {
                    self.s.iter().all(|t| *t == tx)
                }
            }
        }
    }
}

/// Lock manager. Keys are KHID. No timeout: the
/// caller retries or aborts on Deadlock.
pub struct LockMgr {
    next: TxId,
    tab: HashMap<Khid, Grant>,
    held: HashMap<TxId, Vec<(Khid, Mode)>>,
    waiting: HashMap<TxId, Khid>,
}

impl LockMgr {
    pub fn new() -> LockMgr {
        LockMgr {
            next: 1,
            tab: HashMap::new(),
            held: HashMap::new(),
            waiting: HashMap::new(),
        }
    }

    pub fn begin(&mut self) -> TxId {
        let id = self.next;
        self.next += 1;
        self.held.insert(id, Vec::new());
        id
    }

    pub fn acquire(&mut self, tx: TxId, k: Khid, m: Mode) -> Acquire {
        if self.has(tx, k, m) {
            return Acquire::Ok;
        }
        {
            let g = self.tab.entry(k).or_insert_with(Grant::empty);
            if g.compatible(tx, m) {
                match m {
                    Mode::S => {
                        g.s.insert(tx);
                    }
                    Mode::X => {
                        g.s.remove(&tx);
                        g.x = Some(tx);
                    }
                }
                self.held.entry(tx).or_insert(Vec::new()).push((k, m));
                return Acquire::Ok;
            }
            g.wait.push_back((tx, m));
        }
        self.waiting.insert(tx, k);
        if self.cycle(tx) {
            self.cancel_wait(tx, k);
            Acquire::Deadlock
        } else {
            Acquire::Wait
        }
    }

    fn has(&self, tx: TxId, k: Khid, m: Mode) -> bool {
        match self.held.get(&tx) {
            Some(v) => v.iter().any(|&(id, mode)| {
                id == k && (mode == m || mode == Mode::X)
            }),
            None => false,
        }
    }

    fn cancel_wait(&mut self, tx: TxId, k: Khid) {
        self.waiting.remove(&tx);
        if let Some(g) = self.tab.get_mut(&k) {
            g.wait.retain(|&(t, _)| t != tx);
        }
    }

    /// Wait-for: requester waits on every holder of
    /// that key. DFS from tx. A cycle means deadlock.
    fn cycle(&self, tx: TxId) -> bool {
        let mut seen = HashSet::new();
        let mut stack = vec![tx];
        while let Some(n) = stack.pop() {
            if !seen.insert(n) {
                continue;
            }
            let k = match self.waiting.get(&n) {
                Some(k) => *k,
                None => continue,
            };
            let holders = match self.tab.get(&k) {
                Some(g) => g.holders(),
                None => continue,
            };
            for h in holders.iter() {
                if *h == n {
                    continue;
                }
                if *h == tx {
                    return true;
                }
                stack.push(*h);
            }
        }
        false
    }

    /// Wake waiters that now fit. Returns txs that
    /// got the lock.
    pub fn release(&mut self, tx: TxId) -> Vec<TxId> {
        let keys = self.held.remove(&tx).unwrap_or(Vec::new());
        self.waiting.remove(&tx);
        for &(k, _) in keys.iter() {
            if let Some(g) = self.tab.get_mut(&k) {
                g.s.remove(&tx);
                if g.x == Some(tx) {
                    g.x = None;
                }
                g.wait.retain(|&(t, _)| t != tx);
            }
        }
        let mut woke = Vec::new();
        for &(k, _) in keys.iter() {
            loop {
                let next = match self.tab.get(&k).and_then(|g| g.wait.front().cloned()) {
                    Some(p) => p,
                    None => break,
                };
                let ok = self.tab.get(&k).map(|g| g.compatible(next.0, next.1)).unwrap_or(false);
                if !ok {
                    break;
                }
                if let Some(g) = self.tab.get_mut(&k) {
                    g.wait.pop_front();
                    match next.1 {
                        Mode::S => {
                            g.s.insert(next.0);
                        }
                        Mode::X => {
                            g.x = Some(next.0);
                        }
                    }
                }
                self.waiting.remove(&next.0);
                self.held.entry(next.0).or_insert(Vec::new()).push((k, next.1));
                woke.push(next.0);
            }
            let empty = self.tab.get(&k).map(|g| {
                g.x.is_none() && g.s.is_empty() && g.wait.is_empty()
            }).unwrap_or(false);
            if empty {
                self.tab.remove(&k);
            }
        }
        woke
    }

    pub fn holding(&self, tx: TxId) -> usize {
        self.held.get(&tx).map(|v| v.len()).unwrap_or(0)
    }

    pub fn is_waiting(&self, tx: TxId) -> bool {
        self.waiting.contains_key(&tx)
    }
}

impl Default for LockMgr {
    fn default() -> LockMgr {
        LockMgr::new()
    }
}

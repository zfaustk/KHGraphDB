//! 2PL table. Cycle is deadlock.

use crate::{Khid, LockMgr, Mode, Acquire};

#[test]
fn exclusive_conflicts() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let b = m.begin();
    let k = Khid::from_raw(3);
    assert_eq!(m.acquire(a, k, Mode::X), Acquire::Ok);
    assert_eq!(m.acquire(b, k, Mode::X), Acquire::Wait);
    assert!(m.is_waiting(b));
    m.release(a);
    assert!(!m.is_waiting(b));
    assert_eq!(m.holding(b), 1);
}

#[test]
fn shared_share() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let b = m.begin();
    let k = Khid::from_raw(4);
    assert_eq!(m.acquire(a, k, Mode::S), Acquire::Ok);
    assert_eq!(m.acquire(b, k, Mode::S), Acquire::Ok);
}

#[test]
fn upgrade_waits_the_other_share() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let b = m.begin();
    let k = Khid::from_raw(5);
    assert_eq!(m.acquire(a, k, Mode::S), Acquire::Ok);
    assert_eq!(m.acquire(b, k, Mode::S), Acquire::Ok);
    assert_eq!(m.acquire(a, k, Mode::X), Acquire::Wait);
}

#[test]
fn cycle_is_deadlock() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let b = m.begin();
    let k1 = Khid::from_raw(1);
    let k2 = Khid::from_raw(2);
    assert_eq!(m.acquire(a, k1, Mode::X), Acquire::Ok);
    assert_eq!(m.acquire(b, k2, Mode::X), Acquire::Ok);
    assert_eq!(m.acquire(a, k2, Mode::X), Acquire::Wait);
    assert_eq!(m.acquire(b, k1, Mode::X), Acquire::Deadlock);
}

#[test]
fn reacquire_x_is_ok() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let k = Khid::from_raw(9);
    assert_eq!(m.acquire(a, k, Mode::X), Acquire::Ok);
    assert_eq!(m.acquire(a, k, Mode::X), Acquire::Ok);
}

#[test]
fn release_wakes_waiter() {
    let mut m = LockMgr::new();
    let a = m.begin();
    let b = m.begin();
    let k = Khid::from_raw(8);
    assert_eq!(m.acquire(a, k, Mode::X), Acquire::Ok);
    assert_eq!(m.acquire(b, k, Mode::S), Acquire::Wait);
    let woke = m.release(a);
    assert_eq!(woke, vec![b]);
    assert_eq!(m.holding(b), 1);
}

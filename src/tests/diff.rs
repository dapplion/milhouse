use crate::{List, Vector};
use typenum::{U64, U1024};

type L = List<u64, U1024>;

fn changed(new: &L, base: &L) -> Vec<(usize, Option<u64>, u64)> {
    let mut out = vec![];
    new.for_each_changed(base, &mut |i, b, n| out.push((i, b.copied(), *n)))
        .unwrap();
    out
}

fn brute_force(new: &L, base: &L) -> Vec<(usize, Option<u64>, u64)> {
    let base_vec = base.to_vec();
    new.iter()
        .enumerate()
        .filter_map(|(i, n)| {
            let b = base_vec.get(i).copied();
            (b != Some(*n)).then_some((i, b, *n))
        })
        .collect()
}

#[test]
fn for_each_changed_shared_lineage() {
    let base = L::try_from_iter(0..600).unwrap();
    let mut new = base.clone();
    for i in [0usize, 5, 63, 64, 255, 599] {
        *new.get_mut(i).unwrap() += 1000;
    }
    new.push(600).unwrap();
    new.push(601).unwrap();
    new.apply_updates().unwrap();

    assert_eq!(changed(&new, &base), brute_force(&new, &base));
}

#[test]
fn for_each_changed_unrelated_trees() {
    let base = L::try_from_iter(0..500).unwrap();
    let new = L::try_from_iter((0..500).map(|i| if i % 7 == 0 { i + 1_000 } else { i })).unwrap();
    assert_eq!(changed(&new, &base), brute_force(&new, &base));
}

#[test]
fn for_each_changed_empty_base() {
    let base = L::try_from_iter(std::iter::empty()).unwrap();
    let new = L::try_from_iter(10..20).unwrap();
    let out = changed(&new, &base);
    assert_eq!(out.len(), 10);
    assert!(out.iter().all(|(_, b, _)| b.is_none()));
}

#[test]
fn for_each_changed_pending_updates_rejected() {
    let base = L::try_from_iter(0..10).unwrap();
    let mut new = base.clone();
    *new.get_mut(0).unwrap() = 42;
    assert!(new.for_each_changed(&base, &mut |_, _, _| {}).is_err());
}

#[test]
fn to_vec_leaves_matches_to_vec() {
    let list = L::try_from_iter(0..777).unwrap();
    assert_eq!(list.to_vec_leaves(), list.to_vec());

    let unpacked = List::<tree_hash::Hash256, U64>::try_from_iter(
        (0..33u8).map(tree_hash::Hash256::repeat_byte),
    )
    .unwrap();
    assert_eq!(unpacked.to_vec_leaves(), unpacked.to_vec());

    let mut with_updates = list.clone();
    *with_updates.get_mut(3).unwrap() = 999;
    assert_eq!(with_updates.to_vec_leaves(), with_updates.to_vec());
}

#[test]
fn vector_unaffected_smoke() {
    let v = Vector::<u64, U64>::try_from_iter(0..64).unwrap();
    assert_eq!(v.to_vec().len(), 64);
}

use criterion::{criterion_group, criterion_main, Criterion};
use milhouse::List;

pub fn updates(c: &mut Criterion) {
    for n in [100, 100_000, 100_000_000] {
        let mut list = List::<u64, typenum::U1099511627776>::default();
        for i in 0..n {
            list.push(i as u64).unwrap();
        }
        c.bench_function(&format!("apply_recursive_updates {n}"), |b| {
            b.iter(|| list.apply_recursive_updates())
        });
    }
}

criterion_group!(benches, updates);
criterion_main!(benches);

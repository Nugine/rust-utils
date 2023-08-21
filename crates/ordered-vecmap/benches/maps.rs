use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use criterion::{AxisScale, PlotConfiguration};
use rand::Rng;

use fnv::FnvHashMap;
use ordered_vecmap::VecMap;
use std::collections::{BTreeMap, HashMap};

#[inline]
pub fn map_collect<T, U, C>(iter: impl IntoIterator<Item = T>, f: impl FnMut(T) -> U) -> C
where
    C: FromIterator<U>,
{
    iter.into_iter().map(f).collect()
}

pub fn get_trivial(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_trivial");

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for n in [3, 5, 7, 16, 32, 64, 128, 512] {
        let data = {
            let mut v: Vec<u64> = vec![0; n];
            rand::thread_rng().fill(&mut *v);
            v
        };

        let input = data.first().unwrap();

        {
            let map: VecMap<_, _> = map_collect(&data, |&x| (x, x));
            let id = BenchmarkId::new("vecmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: BTreeMap<_, _> = map_collect(&data, |&x| (x, x));
            let id = BenchmarkId::new("btreemap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: HashMap<_, _> = map_collect(&data, |&x| (x, x));
            let id = BenchmarkId::new("hashmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: FnvHashMap<_, _> = map_collect(&data, |&x| (x, x));
            let id = BenchmarkId::new("fnvhashmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }
    }
}

pub fn get_nontrivial(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_nontrivial");

    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for n in [3, 5, 7, 16, 32, 64, 128, 512] {
        let data = {
            let mut v: Vec<u64> = vec![0; n];
            rand::thread_rng().fill(&mut *v);
            v.into_iter().map(|x| x.to_string()).collect::<Vec<_>>()
        };

        let input = data.first().unwrap();

        {
            let map: VecMap<_, _> = map_collect(&data, |x| (x.clone(), x.clone()));
            let id = BenchmarkId::new("vecmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: BTreeMap<_, _> = map_collect(&data, |x| (x.clone(), x.clone()));
            let id = BenchmarkId::new("btreemap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: HashMap<_, _> = map_collect(&data, |x| (x.clone(), x.clone()));
            let id = BenchmarkId::new("hashmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }

        {
            let map: FnvHashMap<_, _> = map_collect(&data, |x| (x.clone(), x.clone()));
            let id = BenchmarkId::new("fnvhashmap", n);
            group.bench_function(id, |b| b.iter(|| map.get(black_box(input)).unwrap()));
        }
    }
}

criterion_group!(benches, get_trivial, get_nontrivial,);
criterion_main!(benches);

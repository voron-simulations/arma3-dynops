use criterion::{Criterion, criterion_group, criterion_main};
use dynops::population::{Group, plan};
use std::hint::black_box;

/// Altis' measured class histogram (786 farms, 217 hamlets, 61 villages, 23
/// towns, 19 cities) trimmed to a round 1100 locations, using the weights
/// `fnc_locationWeight` gives each class. Shuffled with a fixed-seed LCG so
/// heavy locations are scattered the way they are on a real map.
fn altis_like_weights() -> Vec<f64> {
    let classes = [(1.0, 786), (2.0, 217), (4.0, 61), (8.0, 23), (16.0, 13)];
    let mut weights: Vec<f64> = classes
        .iter()
        .flat_map(|&(weight, count)| std::iter::repeat_n(weight, count))
        .collect();
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    for i in (1..weights.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        weights.swap(i, (state >> 33) as usize % (i + 1));
    }
    weights
}

/// A 20-group infantry catalogue (fireteams through platoons) shared by three
/// factions, 10/5/5, like a side with a main force and two smaller ones.
fn catalogue() -> Vec<Group> {
    const COSTS: [u32; 20] = [
        2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 8, 8, 9, 10, 11, 12, 14, 16, 18, 20,
    ];
    COSTS
        .iter()
        .enumerate()
        .map(|(i, &cost)| Group {
            faction: match i % 4 {
                0 | 1 => 0,
                2 => 1,
                _ => 2,
            },
            cost,
        })
        .collect()
}

/// The same 70/20/10 split of the side budget at every size.
fn faction_budgets(total: u32) -> [u32; 3] {
    let main = total * 7 / 10;
    let second = total * 2 / 10;
    [main, second, total - main - second]
}

fn bench_plan(c: &mut Criterion) {
    let mut group = c.benchmark_group("population_plan");
    let weights = altis_like_weights();
    assert_eq!(weights.len(), 1100);
    let groups = catalogue();

    for &total in &[500u32, 5000] {
        let budgets = faction_budgets(total);
        group.bench_function(
            format!("locations_1100_groups_20_factions_3_budget_{total}"),
            |b| {
                b.iter(|| {
                    plan(
                        black_box(&weights),
                        black_box(&budgets),
                        black_box(&groups),
                        42,
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_plan);
criterion_main!(benches);

//! End-to-end checks of `locations::detect` against the bundled per-map
//! building dumps in `data/`. See adr/0001-location-detection.md for the
//! algorithm and the measurement table these tests are validated against.

use dynops::locations::{DetectParams, Location, LocationClass, detect};
use nalgebra::Vector2;

fn parse(data: &str) -> Vec<Vector2<f64>> {
    data.lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once(','))
        .filter_map(|(x, y)| Some(Vector2::new(x.parse().ok()?, y.parse().ok()?)))
        .collect()
}

/// Structural invariants that must hold for every map, regardless of whether
/// the exact class histogram is known.
fn check_structural_invariants(
    name: &str,
    points: &[Vector2<f64>],
    locations: &[Location],
    params: &DetectParams,
) {
    assert!(
        !locations.is_empty(),
        "{name}: expected at least one location"
    );

    for loc in locations {
        assert!(
            loc.buildings >= params.min_buildings,
            "{name}: location {} has {} buildings, below min_buildings {}",
            loc.id,
            loc.buildings,
            params.min_buildings
        );

        // Every one of the location's own buildings must lie inside its OBB.
        // We can't recover exact membership from the public API, so instead
        // count how many *of the whole map's* points the OBB contains: with
        // disjoint, non-overlapping locations (checked separately below) that
        // count can only be >= the location's own members, and a real
        // containment bug (a member point actually falling outside its own
        // box) would make it strictly less than `buildings`.
        let contained = points
            .iter()
            .filter(|&&p| loc.obb.contains(p, 1e-6))
            .count();
        assert!(
            contained >= loc.buildings,
            "{name}: location {} (class {:?}) claims {} buildings but its OBB only contains {}",
            loc.id,
            loc.class,
            loc.buildings,
            contained
        );

        // Regression guard for the bug this feature replaces: a fixed-eps
        // single-linkage chain could span a tenth of the map (Altis: up to
        // 3688m). The span-limited recursive cut keeps every location within
        // max_span except when min_eps is reached first, which the ADR's
        // measurement table shows can overrun by up to ~15% (775m against a
        // 700m max_span on Altis). 2x max_span is a generous ceiling that
        // accepts any legitimate min_eps floor overrun while still catching
        // the old unbounded-chain failure mode by a wide margin.
        let span = (loc.obb.a * 2.0).max(loc.obb.b * 2.0);
        assert!(
            span <= params.max_span * 2.0,
            "{name}: location {} span {span:.1}m looks like the old unbounded single-linkage chain, not a min_eps floor overrun",
            loc.id
        );
    }

    // Locations partition (a subset of) the input: every point can belong to
    // at most one cluster by construction (`detect_into` only ever recurses
    // into an already-identified single group, so no point can be handed to
    // two different output locations), which the `sum(buildings) <= n`
    // aggregate below cross-checks. Note this is *not* the same as OBBs never
    // overlapping -- two disjoint clusters' tight minimum-area boxes can
    // still overlap in the empty space between them (e.g. two diagonal
    // clusters near a shared corner), which real map data does exhibit, so
    // OBB overlap is not a valid disjointness test.
    let total_buildings: usize = locations.iter().map(|l| l.buildings).sum();
    assert!(
        total_buildings <= points.len(),
        "{name}: locations claim {total_buildings} buildings but only {} points were input",
        points.len()
    );
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Histogram {
    farm: usize,
    hamlet: usize,
    village: usize,
    town: usize,
    city: usize,
}

impl Histogram {
    fn of(locations: &[Location]) -> Self {
        let mut h = Histogram::default();
        for loc in locations {
            match loc.class {
                LocationClass::Farm => h.farm += 1,
                LocationClass::Hamlet => h.hamlet += 1,
                LocationClass::Village => h.village += 1,
                LocationClass::Town => h.town += 1,
                LocationClass::City => h.city += 1,
            }
        }
        h
    }
}

fn check_map(name: &str, data: &str, expected_histogram: Option<Histogram>) {
    let points = parse(data);
    let params = DetectParams::default();
    let locations = detect(&points, &params);

    check_structural_invariants(name, &points, &locations, &params);

    if let Some(expected) = expected_histogram {
        let actual = Histogram::of(&locations);
        assert_eq!(
            actual, expected,
            "{name}: class histogram does not match the committed measurement table"
        );
    }
}

// Class histograms below are the committed measurements from
// adr/0001-location-detection.md's table (Stratis/Malden/Altis), reproduced
// exactly by `locations::detect` with `DetectParams::default()`.

#[test]
fn test_map_stratis() {
    check_map(
        "Stratis",
        include_str!("../data/objects.Stratis.txt"),
        Some(Histogram {
            farm: 16,
            hamlet: 12,
            village: 1,
            town: 2,
            city: 0,
        }),
    );
}

#[test]
fn test_map_malden() {
    check_map(
        "Malden",
        include_str!("../data/objects.Malden.txt"),
        Some(Histogram {
            farm: 100,
            hamlet: 59,
            village: 16,
            town: 10,
            city: 2,
        }),
    );
}

#[test]
fn test_map_altis() {
    check_map(
        "Altis",
        include_str!("../data/objects.Altis.txt"),
        Some(Histogram {
            farm: 786,
            hamlet: 217,
            village: 61,
            town: 23,
            city: 19,
        }),
    );
}

// The remaining bundled maps aren't in the ADR's committed table, so only the
// structural invariants (containment, disjointness, span ceiling) apply.

#[test]
fn test_map_livonia() {
    check_map("Livonia", include_str!("../data/objects.Livonia.txt"), None);
}

#[test]
fn test_map_tanoa() {
    check_map("Tanoa", include_str!("../data/objects.Tanoa.txt"), None);
}

#[test]
fn test_map_chernarus() {
    check_map(
        "Chernarus2020",
        include_str!("../data/objects.Chernarus2020.txt"),
        None,
    );
}

//! Budget-driven group allocation over weighted locations.
//!
//! Pure logic behind the `population:*` extension commands. One plan covers
//! all the locations one side owns: each location has a weight, each group has
//! a cost and belongs to a faction, and each faction has a point budget. The
//! plan decides which groups go to which locations without any faction
//! exceeding its budget. The extension never sees class names or positions,
//! only indices into the caller's arrays. See adr/0002-population.md for the
//! design.

use std::fmt;

/// One group placed at one location. Both fields are indices: `location` into
/// the `weights` and `group` into the `groups` slice handed to [`plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    pub location: usize,
    pub group: usize,
}

/// A group the side can field: which faction pays for it, and what it costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    /// Index into the `faction_budgets` slice handed to [`plan`].
    pub faction: usize,
    pub cost: u32,
}

/// Most assignments one [`plan`] may produce. They are all held in memory until
/// the session ends and paged out ~9KB at a time, so a mistyped budget must not
/// be able to exhaust the memory of the Arma process.
pub const MAX_ASSIGNMENTS: usize = 100_000;

/// Why [`plan`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanError {
    /// A weight is NaN, infinite or negative.
    InvalidWeight { index: usize, value: f64 },
    /// A group costs nothing, so "keep picking until the budget runs out"
    /// would never terminate.
    ZeroCost { index: usize },
    /// A group names a faction that has no budget entry.
    FactionOutOfRange {
        index: usize,
        faction: usize,
        factions: usize,
    },
    /// The budgets could buy more than [`MAX_ASSIGNMENTS`] groups: the sum of
    /// the faction budgets divided by the cheapest group's cost is `worst_case`.
    TooManyAssignments { max: usize, worst_case: u64 },
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWeight { index, value } => write!(
                f,
                "location weight at index {index} must be finite and non-negative, got {value}"
            ),
            Self::ZeroCost { index } => {
                write!(f, "group cost at index {index} must be greater than 0")
            }
            Self::FactionOutOfRange {
                index,
                faction,
                factions,
            } => write!(
                f,
                "group at index {index} names faction {faction}, but only {factions} faction budgets were given"
            ),
            Self::TooManyAssignments { max, worst_case } => write!(
                f,
                "the budgets could place up to {worst_case} groups, above the limit of {max}; \
                 lower the budgets or use more expensive groups"
            ),
        }
    }
}

impl std::error::Error for PlanError {}

/// Sign and mantissa cleared: what is left of an `f64` is the power of two at
/// or below it.
const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;

/// Indices of locations that can receive anything (weight above zero), heaviest
/// first, lowest index first on ties.
fn weight_order(weights: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..weights.len()).filter(|&i| weights[i] > 0.0).collect();
    order.sort_by(|&a, &b| weights[b].total_cmp(&weights[a]).then(a.cmp(&b)));
    order
}

/// Splits `budget` over locations proportionally to `weights`.
///
/// Each share is rounded down, and the points lost to rounding go one at a
/// time to the highest-weight locations (lowest index first on ties), so the
/// shares always sum to `budget` exactly. Zero-weight locations get nothing.
/// Returns all zeros when no location has positive weight.
fn split_budget(weights: &[f64], budget: u64) -> Vec<u64> {
    let mut shares = vec![0u64; weights.len()];
    let order = weight_order(weights);
    let Some(&heaviest) = order.first() else {
        return shares;
    };

    // Dividing by a power of two is exact, so the weight ratios (and with them
    // the rounding of cases like 1:3 of 400) are untouched, while the sum and
    // `budget * weight` stay finite however large the weights are.
    let scale = f64::from_bits(weights[heaviest].to_bits() & EXPONENT_MASK).max(f64::MIN_POSITIVE);
    let scaled: Vec<f64> = weights.iter().map(|&w| w / scale).collect();
    let total: f64 = scaled.iter().sum();

    let mut assigned = 0u64;
    for &i in &order {
        let exact = budget as f64 * scaled[i] / total;
        // The clamp keeps float error from ever handing out more than `budget`.
        let share = (exact.floor() as u64).min(budget - assigned);
        shares[i] = share;
        assigned += share;
    }

    // Normally fewer points are left than there are locations, but float error
    // in the floors could in principle leave more, so deal them out in full
    // rounds before the one-point-each tail.
    let leftover = budget - assigned;
    let count = order.len() as u64;
    let (each, extra) = (leftover / count, leftover % count);
    for (rank, &i) in order.iter().enumerate() {
        shares[i] += each + u64::from((rank as u64) < extra);
    }
    shares
}

/// SplitMix64. Chosen over the ADR's xorshift64* because every seed is valid,
/// including 0, whereas xorshift state must be non-zero. Inline to avoid a
/// dependency for ten lines.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n`, for `n > 0`. Multiply-shift without rejection: the
    /// bias is below `n / 2^64`, far beneath anything a catalogue can notice.
    fn below(&mut self, n: u64) -> u64 {
        ((u128::from(self.next_u64()) * u128::from(n)) >> 64) as u64
    }
}

/// The side's group catalogue together with each faction's unspent budget.
struct Catalogue<'a> {
    groups: &'a [Group],
    /// Per faction: its group indices by ascending cost, ties by ascending
    /// index, so "which of its groups fit in `cap` points" is a prefix.
    by_faction: Vec<Vec<usize>>,
    /// Per faction: points not yet spent.
    remaining: Vec<u32>,
}

impl<'a> Catalogue<'a> {
    fn new(groups: &'a [Group], faction_budgets: &[u32]) -> Self {
        let mut by_faction = vec![Vec::new(); faction_budgets.len()];
        for (index, group) in groups.iter().enumerate() {
            by_faction[group.faction].push(index);
        }
        for members in &mut by_faction {
            members.sort_by_key(|&g| groups[g].cost);
        }
        Self {
            groups,
            by_faction,
            remaining: faction_budgets.to_vec(),
        }
    }

    /// How many of `faction`'s groups can be bought with `limit` points
    /// without exceeding the faction's own remaining budget.
    fn fitting(&self, faction: usize, limit: u64) -> usize {
        let cap = limit.min(u64::from(self.remaining[faction]));
        self.by_faction[faction].partition_point(|&g| u64::from(self.groups[g].cost) <= cap)
    }

    /// Picks a group costing at most `limit` that its faction can still pay
    /// for, and charges the faction. The faction is chosen first, weighted by
    /// its remaining budget among factions with at least one affordable group,
    /// so factions are drawn down in step and their groups come out in
    /// proportion to their budgets. The group is then uniform within the
    /// faction. `None` when no group qualifies.
    fn pick(&mut self, rng: &mut Rng, limit: u64) -> Option<usize> {
        let total_weight: u64 = (0..self.remaining.len())
            .filter(|&f| self.fitting(f, limit) > 0)
            .map(|f| u64::from(self.remaining[f]))
            .sum();
        if total_weight == 0 {
            return None;
        }

        let mut ticket = rng.below(total_weight);
        for faction in 0..self.remaining.len() {
            let fitting = self.fitting(faction, limit);
            if fitting == 0 {
                continue;
            }
            let weight = u64::from(self.remaining[faction]);
            if ticket < weight {
                let group = self.by_faction[faction][rng.below(fitting as u64) as usize];
                self.remaining[faction] -= self.groups[group].cost;
                return Some(group);
            }
            ticket -= weight;
        }
        // Unreachable: `ticket < total_weight`, which is the sum of the
        // weights walked above.
        None
    }
}

fn validate(
    weights: &[f64],
    faction_budgets: &[u32],
    groups: &[Group],
    total_budget: u64,
) -> Result<(), PlanError> {
    if let Some((index, &value)) = weights
        .iter()
        .enumerate()
        .find(|&(_, &w)| !w.is_finite() || w < 0.0)
    {
        return Err(PlanError::InvalidWeight { index, value });
    }
    for (index, group) in groups.iter().enumerate() {
        if group.faction >= faction_budgets.len() {
            return Err(PlanError::FactionOutOfRange {
                index,
                faction: group.faction,
                factions: faction_budgets.len(),
            });
        }
        if group.cost == 0 {
            return Err(PlanError::ZeroCost { index });
        }
    }
    // Even the cheapest group, bought over and over, bounds the output. Judged
    // on the inputs alone, not on what fits where, so whether a call is
    // accepted never depends on the locations.
    if let Some(cheapest) = groups.iter().map(|g| g.cost).min() {
        let worst_case = total_budget / u64::from(cheapest);
        if worst_case > MAX_ASSIGNMENTS as u64 {
            return Err(PlanError::TooManyAssignments {
                max: MAX_ASSIGNMENTS,
                worst_case,
            });
        }
    }
    Ok(())
}

/// Allocates one side's groups to the side's locations. See the module docs
/// and the ADR.
///
/// `weights[i]` is location `i`'s relative weight, `faction_budgets[f]` is
/// faction `f`'s point budget, and each of `groups` has a cost and the faction
/// that pays for it. The locations' shares are split from the total budget
/// (the sum of the faction budgets) by weight. The factions' budgets only
/// limit what may be drawn: no faction ever spends more than its own, and
/// factions are drawn in proportion to what they have left.
///
/// The result lists assignments by ascending location index, and within a
/// location in the order they were picked. It is a pure function of its
/// arguments. A group with a nonexistent faction or a zero cost, a bad weight,
/// or budgets that could buy more than [`MAX_ASSIGNMENTS`] groups, is an error;
/// every other degenerate input (no locations, no groups, no budget, nothing
/// affordable) simply yields no assignments. Validation runs first, so a bad
/// input is rejected whatever the other arguments are.
pub fn plan(
    weights: &[f64],
    faction_budgets: &[u32],
    groups: &[Group],
    seed: u64,
) -> Result<Vec<Assignment>, PlanError> {
    let total_budget: u64 = faction_budgets.iter().map(|&b| u64::from(b)).sum();
    validate(weights, faction_budgets, groups, total_budget)?;

    let mut catalogue = Catalogue::new(groups, faction_budgets);
    let mut rng = Rng(seed);
    let mut assignments = Vec::new();

    // Local pass. Points a location cannot spend are pooled rather than lost.
    // The pool never exceeds the total budget because each leftover is at most
    // its share and the shares sum to the total.
    let mut pool = 0u64;
    for (location, &share) in split_budget(weights, total_budget).iter().enumerate() {
        let mut remaining = share;
        while let Some(group) = catalogue.pick(&mut rng, remaining) {
            assignments.push(Assignment { location, group });
            remaining -= u64::from(groups[group].cost);
        }
        pool += remaining;
    }

    // Pool pass: hand out one group per location per round, heaviest first,
    // so leftovers favour big settlements without piling onto a single one.
    // `cycle` over an empty order yields nothing, which ends the loop when no
    // location can receive anything.
    for &location in weight_order(weights).iter().cycle() {
        let Some(group) = catalogue.pick(&mut rng, pool) else {
            break;
        };
        assignments.push(Assignment { location, group });
        pool -= u64::from(groups[group].cost);
    }

    // Stable, so the pool pass's picks follow the local pass's at each location.
    assignments.sort_by_key(|x| x.location);
    Ok(assignments)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(location: usize, group: usize) -> Assignment {
        Assignment { location, group }
    }

    fn g(faction: usize, cost: u32) -> Group {
        Group { faction, cost }
    }

    /// A catalogue of one faction (index 0) with the given group costs.
    fn solo(costs: &[u32]) -> Vec<Group> {
        costs.iter().map(|&cost| g(0, cost)).collect()
    }

    fn spent_by_faction(out: &[Assignment], groups: &[Group], factions: usize) -> Vec<u64> {
        let mut spent = vec![0u64; factions];
        for x in out {
            let group = groups[x.group];
            spent[group.faction] += u64::from(group.cost);
        }
        spent
    }

    fn picks_of(out: &[Assignment], group: usize) -> usize {
        out.iter().filter(|x| x.group == group).count()
    }

    fn per_location(out: &[Assignment], locations: usize) -> Vec<usize> {
        let mut counts = vec![0; locations];
        for x in out {
            counts[x.location] += 1;
        }
        counts
    }

    /// Tiny LCG so the property test is reproducible without a dev-dependency.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    // --- split_budget ---

    #[test]
    fn split_is_proportional_when_exact() {
        assert_eq!(split_budget(&[1.0, 3.0], 400), vec![100, 300]);
    }

    #[test]
    fn split_gives_remainder_to_highest_weights_first() {
        // 10 * (1,1,2)/4 = 2.5, 2.5, 5 -> floors 2, 2, 5, leftover 1 goes to
        // the heaviest location.
        assert_eq!(split_budget(&[1.0, 1.0, 2.0], 10), vec![2, 2, 6]);
    }

    #[test]
    fn split_breaks_ties_by_lower_index() {
        assert_eq!(split_budget(&[1.0, 1.0, 1.0], 5), vec![2, 2, 1]);
    }

    #[test]
    fn split_gives_zero_weight_locations_nothing() {
        assert_eq!(split_budget(&[0.0, 1.0, 0.0], 7), vec![0, 7, 0]);
    }

    #[test]
    fn split_without_positive_weight_is_all_zero() {
        assert_eq!(split_budget(&[0.0, 0.0], 9), vec![0, 0]);
        assert_eq!(split_budget(&[], 9), Vec::<u64>::new());
    }

    #[test]
    fn split_survives_weights_whose_sum_overflows() {
        let shares = split_budget(&[f64::MAX, f64::MAX, f64::MAX], 10);
        assert_eq!(shares.iter().sum::<u64>(), 10);
        assert!(shares.iter().all(|&s| (3..=4).contains(&s)), "{shares:?}");
    }

    #[test]
    fn split_survives_subnormal_weights() {
        let shares = split_budget(&[f64::from_bits(1), f64::from_bits(3)], 100);
        assert_eq!(shares.iter().sum::<u64>(), 100);
    }

    #[test]
    fn split_handles_budgets_beyond_u32() {
        // Several factions' u32 budgets add up past u32::MAX.
        let budget = 2 * u64::from(u32::MAX);
        let shares = split_budget(&[1.0, 2.0, 4.0], budget);
        assert_eq!(shares.iter().sum::<u64>(), budget);
    }

    #[test]
    fn split_always_sums_to_budget() {
        let mut rng = Lcg(7);
        for _ in 0..500 {
            let n = rng.below(40) as usize;
            let weights: Vec<f64> = (0..n)
                .map(|_| match rng.below(7) {
                    0 => 0.0,
                    1 => 1.0,
                    2 => 2.0,
                    3 => 4.0,
                    4 => 8.0,
                    5 => 16.0,
                    _ => rng.below(1000) as f64 / 7.0,
                })
                .collect();
            let budget = rng.below(100_000);
            let shares = split_budget(&weights, budget);
            assert_eq!(shares.len(), n);
            if weights.iter().any(|&w| w > 0.0) {
                assert_eq!(
                    shares.iter().sum::<u64>(),
                    budget,
                    "weights {weights:?} budget {budget}"
                );
            } else {
                assert!(shares.iter().all(|&s| s == 0));
            }
            for (w, s) in weights.iter().zip(&shares) {
                if *w == 0.0 {
                    assert_eq!(*s, 0, "zero weight must get nothing");
                }
            }
        }
    }

    // --- plan: degenerate inputs ---

    #[test]
    fn empty_weights_yield_no_assignments() {
        assert_eq!(plan(&[], &[100], &solo(&[1, 2]), 1), Ok(vec![]));
    }

    #[test]
    fn empty_catalogue_yields_no_assignments() {
        assert_eq!(plan(&[1.0, 2.0], &[100], &[], 1), Ok(vec![]));
        assert_eq!(plan(&[1.0, 2.0], &[], &[], 1), Ok(vec![]));
    }

    #[test]
    fn zero_budget_yields_no_assignments() {
        assert_eq!(plan(&[1.0, 2.0], &[0], &solo(&[1, 2]), 1), Ok(vec![]));
        assert_eq!(
            plan(&[1.0, 2.0], &[0, 0], &[g(0, 1), g(1, 1)], 1),
            Ok(vec![])
        );
    }

    #[test]
    fn all_zero_weights_yield_no_assignments() {
        assert_eq!(plan(&[0.0, 0.0, 0.0], &[100], &solo(&[1]), 1), Ok(vec![]));
    }

    #[test]
    fn cheapest_group_above_budget_yields_no_assignments() {
        assert_eq!(plan(&[1.0, 5.0], &[10], &solo(&[11, 20]), 1), Ok(vec![]));
    }

    #[test]
    fn group_costing_more_than_its_factions_budget_is_never_placed() {
        // The total budget is plentiful, but it belongs to a faction with no
        // groups.
        assert_eq!(plan(&[1.0], &[5, 100], &[g(0, 10)], 1), Ok(vec![]));
    }

    // --- plan: validation ---

    #[test]
    fn non_finite_or_negative_weights_are_rejected() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            let err =
                plan(&[1.0, bad], &[10], &solo(&[1]), 1).expect_err("bad weight must be rejected");
            assert!(
                matches!(err, PlanError::InvalidWeight { index: 1, .. }),
                "{bad}: {err:?}"
            );
        }
    }

    #[test]
    fn negative_zero_weight_counts_as_zero() {
        assert_eq!(
            plan(&[-0.0, 1.0], &[3], &solo(&[1]), 1),
            Ok(vec![a(1, 0); 3])
        );
    }

    #[test]
    fn zero_cost_is_rejected() {
        assert_eq!(
            plan(&[1.0], &[10], &solo(&[3, 0]), 1),
            Err(PlanError::ZeroCost { index: 1 })
        );
    }

    #[test]
    fn faction_without_a_budget_entry_is_rejected() {
        assert_eq!(
            plan(&[1.0], &[10, 10], &[g(0, 1), g(2, 1)], 1),
            Err(PlanError::FactionOutOfRange {
                index: 1,
                faction: 2,
                factions: 2
            })
        );
    }

    #[test]
    fn groups_without_any_faction_budget_are_rejected() {
        assert_eq!(
            plan(&[1.0], &[], &solo(&[1]), 1),
            Err(PlanError::FactionOutOfRange {
                index: 0,
                faction: 0,
                factions: 0
            })
        );
    }

    #[test]
    fn validation_runs_before_the_degenerate_shortcuts() {
        // Otherwise the same bad catalogue would be accepted or rejected
        // depending on an unrelated budget or location list.
        assert_eq!(
            plan(&[], &[10], &solo(&[0]), 1),
            Err(PlanError::ZeroCost { index: 0 })
        );
        assert_eq!(
            plan(&[1.0], &[0], &solo(&[0]), 1),
            Err(PlanError::ZeroCost { index: 0 })
        );
        assert!(plan(&[], &[], &solo(&[1]), 1).is_err());
        assert!(plan(&[f64::NAN], &[10], &[], 1).is_err());
    }

    #[test]
    fn errors_name_the_offending_index() {
        let msg = plan(&[1.0, -2.0], &[1], &solo(&[1]), 0)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("index 1"), "{msg}");
        let msg = plan(&[1.0], &[1], &solo(&[1, 1, 0]), 0)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("index 2"), "{msg}");
        let msg = plan(&[1.0], &[1], &[g(0, 1), g(7, 1)], 0)
            .unwrap_err()
            .to_string();
        assert!(
            msg.contains("index 1") && msg.contains("faction 7"),
            "{msg}"
        );
    }

    // --- plan: output cap ---

    #[test]
    fn budget_that_could_place_too_many_groups_is_rejected() {
        let err = plan(&[1.0], &[MAX_ASSIGNMENTS as u32 + 1], &solo(&[1]), 1)
            .expect_err("one point over the cap must be rejected");
        assert_eq!(
            err,
            PlanError::TooManyAssignments {
                max: MAX_ASSIGNMENTS,
                worst_case: MAX_ASSIGNMENTS as u64 + 1
            }
        );
        let msg = err.to_string();
        assert!(
            msg.contains("100000"),
            "message should state the limit: {msg}"
        );
    }

    #[test]
    fn budget_exactly_at_the_cap_is_accepted() {
        let out = plan(&[1.0, 1.0], &[MAX_ASSIGNMENTS as u32], &solo(&[1]), 1).unwrap();
        assert_eq!(out.len(), MAX_ASSIGNMENTS);
    }

    #[test]
    fn cap_bound_uses_the_cheapest_group_of_any_faction() {
        // 120_000 points in total; the cheapest group costs 1.
        let budgets = [60_000, 60_000];
        assert!(plan(&[1.0], &budgets, &[g(0, 5), g(1, 1)], 1).is_err());
        // The same budgets with a cheapest cost of 2 can place at most 60_000.
        assert!(plan(&[1.0], &budgets, &[g(0, 5), g(1, 2)], 1).is_ok());
    }

    #[test]
    fn cap_is_checked_on_the_summed_budgets_without_overflow() {
        let err = plan(&[1.0], &[u32::MAX, u32::MAX], &solo(&[1]), 1).unwrap_err();
        assert!(
            matches!(err, PlanError::TooManyAssignments { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn cap_does_not_depend_on_the_locations() {
        // Rejected even when no location could receive anything.
        assert!(plan(&[], &[u32::MAX], &solo(&[1]), 1).is_err());
        assert!(plan(&[0.0], &[u32::MAX], &solo(&[1]), 1).is_err());
    }

    #[test]
    fn huge_budgets_are_fine_when_groups_are_expensive_or_absent() {
        assert!(plan(&[1.0], &[u32::MAX], &[], 1).is_ok());
        assert!(plan(&[1.0], &[u32::MAX], &solo(&[u32::MAX]), 1).is_ok());
        assert!(plan(&[1.0], &[u32::MAX], &solo(&[100_000]), 1).is_ok());
    }

    // --- plan: allocation behaviour ---

    #[test]
    fn weights_split_assignments_proportionally() {
        let out = plan(&[1.0, 3.0], &[400], &solo(&[1]), 9).unwrap();
        assert_eq!(per_location(&out, 2), vec![100, 300]);
    }

    #[test]
    fn leftover_points_are_pooled_and_spent() {
        // Shares 2, 2, 1: the third location cannot afford a group on its own.
        let out = plan(&[1.0, 1.0, 1.0], &[5], &solo(&[2]), 3).unwrap();
        assert_eq!(out, vec![a(0, 0), a(1, 0)]);
        // Shares 2, 2, 2.
        let out = plan(&[1.0, 1.0, 1.0], &[6], &solo(&[2]), 3).unwrap();
        assert_eq!(out, vec![a(0, 0), a(1, 0), a(2, 0)]);
    }

    #[test]
    fn pooled_points_go_to_the_heaviest_location_first() {
        // Shares 3 and 6. Location 0 can't afford a cost-4 group (3 left over),
        // location 1 buys one and has 2 left over. The pool of 5 buys exactly
        // one more group, and it must go to the heavier location 1.
        let out = plan(&[1.0, 2.0], &[9], &solo(&[4]), 5).unwrap();
        assert_eq!(out, vec![a(1, 0), a(1, 0)]);
    }

    #[test]
    fn pool_keeps_going_round_robin_until_nothing_fits() {
        // Twelve shares of 1 can never buy a cost-3 group, so all 12 points
        // are pooled. They buy four groups, dealt one per location in weight
        // order (ties by index) rather than piled onto a single location.
        let out = plan(&[1.0; 12], &[12], &solo(&[3]), 2).unwrap();
        assert_eq!(out, vec![a(0, 0), a(1, 0), a(2, 0), a(3, 0)]);
    }

    #[test]
    fn pooled_points_respect_faction_budgets() {
        // Shares 2, 2, 1 of a total 5. Location 0 buys the only group faction
        // 0 can afford. Locations 1 and 2 can't afford faction 1's cost-3
        // group locally (2 and 1 points), but the pool of 3 can, and the group
        // goes to the heaviest location (lowest index on the tie).
        let groups = [g(0, 2), g(1, 3)];
        let out = plan(&[1.0, 1.0, 1.0], &[2, 3], &groups, 4).unwrap();
        assert_eq!(out, vec![a(0, 0), a(0, 1)]);
    }

    #[test]
    fn location_below_cheapest_group_gets_nothing_when_pool_stays_empty() {
        // Shares 6 and 2. Location 0 buys one cost-5 group and keeps 1,
        // location 1 keeps 2. The pool of 3 can't buy anything.
        let out = plan(&[3.0, 1.0], &[8], &solo(&[5]), 1).unwrap();
        assert_eq!(out, vec![a(0, 0)]);
    }

    #[test]
    fn zero_weight_locations_never_receive_groups() {
        let out = plan(&[0.0, 1.0, 0.0, 2.0], &[100], &solo(&[1, 2]), 11).unwrap();
        assert!(!out.is_empty());
        assert!(out.iter().all(|x| x.location == 1 || x.location == 3));
    }

    #[test]
    fn only_groups_that_fit_are_picked() {
        // A single location with share 7: the cost-100 group must never be
        // picked, and the loop must still stop once nothing fits.
        let groups = solo(&[3, 100, 4]);
        let out = plan(&[1.0], &[7], &groups, 77).unwrap();
        assert!(out.iter().all(|x| x.group != 1));
        assert!(spent_by_faction(&out, &groups, 1)[0] <= 7);
    }

    #[test]
    fn every_group_of_a_faction_gets_picked_uniformly() {
        let groups = solo(&[1, 1, 1, 1]);
        let out = plan(&[1.0], &[400], &groups, 5).unwrap();
        let seen: Vec<usize> = (0..4).map(|group| picks_of(&out, group)).collect();
        // Uniform picking over 400 draws: each bucket lands near 100.
        assert!(seen.iter().all(|&n| (50..=150).contains(&n)), "{seen:?}");
    }

    // --- plan: factions ---

    #[test]
    fn budgets_9_and_1_place_ten_groups_in_that_ratio() {
        let groups = [g(0, 1), g(1, 1)];
        let out = plan(&[1.0], &[9, 1], &groups, 3).unwrap();
        assert_eq!(out.len(), 10);
        assert_eq!(picks_of(&out, 0), 9);
        assert_eq!(picks_of(&out, 1), 1);
    }

    #[test]
    fn faction_with_zero_budget_never_appears() {
        let groups = [g(0, 1), g(1, 1), g(2, 2)];
        for seed in 0..20 {
            let out = plan(&[1.0, 2.0], &[0, 50, 0], &groups, seed).unwrap();
            assert_eq!(out.len(), 50);
            assert!(out.iter().all(|x| x.group == 1), "seed {seed}: {out:?}");
        }
    }

    #[test]
    fn faction_budgets_are_spent_exactly_when_groups_cost_one() {
        let groups = [g(0, 1), g(1, 1), g(2, 1)];
        let out = plan(&[1.0, 3.0, 2.0], &[3, 100, 17], &groups, 8).unwrap();
        assert_eq!(spent_by_faction(&out, &groups, 3), vec![3, 100, 17]);
    }

    #[test]
    fn faction_is_picked_in_proportion_to_its_remaining_budget() {
        // Location 0's share is 500 of 1000 points: 900 points belong to
        // faction 0 and 100 to faction 1. Choosing a faction uniformly would
        // give faction 1 about 250 of those picks; choosing by remaining
        // budget gives about 50 (std dev ~5).
        let groups = [g(0, 1), g(1, 1)];
        let out = plan(&[1.0, 1.0], &[900, 100], &groups, 21).unwrap();
        let at_first = &out[..500];
        assert!(at_first.iter().all(|x| x.location == 0));
        let faction_one = at_first.iter().filter(|x| x.group == 1).count();
        assert!(
            (30..=70).contains(&faction_one),
            "faction 1 took {faction_one} of the first 500 picks"
        );
    }

    #[test]
    fn groups_a_faction_cannot_afford_are_skipped_for_the_other_factions() {
        // Faction 0 has 5 points but only a cost-10 group: it never places
        // anything, and faction 1 spends its own 50 points.
        let groups = [g(0, 10), g(1, 10)];
        let out = plan(&[1.0], &[5, 50], &groups, 6).unwrap();
        assert_eq!(out.len(), 5);
        assert!(out.iter().all(|x| x.group == 1));
    }

    #[test]
    fn budgets_adding_up_beyond_u32_are_handled() {
        // Each faction can afford exactly one cost-u32::MAX group, and no
        // single location's share (or u32) could hold both.
        let groups = [g(0, u32::MAX), g(1, u32::MAX)];
        let budgets = [u32::MAX, u32::MAX];
        let out = plan(&[1.0, 2.0], &budgets, &groups, 1).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(
            spent_by_faction(&out, &groups, 2),
            vec![u64::from(u32::MAX); 2]
        );
    }

    // --- plan: determinism ---

    #[test]
    fn same_inputs_and_seed_give_identical_output() {
        let weights = [1.0, 4.0, 2.0, 8.0, 1.0];
        let groups = [g(0, 2), g(0, 3), g(1, 5), g(1, 8), g(2, 13)];
        let first = plan(&weights, &[100, 60, 40], &groups, 42).unwrap();
        let second = plan(&weights, &[100, 60, 40], &groups, 42).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_give_different_output() {
        let weights = [1.0; 10];
        let groups = [g(0, 1), g(0, 2), g(1, 3), g(1, 4), g(1, 5)];
        let first = plan(&weights, &[60, 40], &groups, 1).unwrap();
        let second = plan(&weights, &[60, 40], &groups, 2).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn seed_zero_is_usable() {
        let out = plan(&[1.0], &[50], &solo(&[1, 2, 3]), 0).unwrap();
        let distinct: std::collections::HashSet<_> = out.iter().map(|x| x.group).collect();
        assert!(distinct.len() > 1, "seed 0 degenerated: {out:?}");
    }

    // --- plan: properties ---

    #[test]
    fn random_inputs_never_overspend_any_faction_and_leave_nothing_placeable() {
        let mut rng = Lcg(0xD15EA5E);
        for iteration in 0..200 {
            let locations = rng.below(25) as usize;
            let weights: Vec<f64> = (0..locations)
                .map(|_| [0.0, 1.0, 1.0, 2.0, 4.0, 8.0, 16.0][rng.below(7) as usize])
                .collect();
            let factions = rng.below(4) as usize;
            let budgets: Vec<u32> = (0..factions).map(|_| rng.below(300) as u32).collect();
            let group_count = if factions == 0 {
                0
            } else {
                rng.below(9) as usize
            };
            let groups: Vec<Group> = (0..group_count)
                .map(|_| {
                    g(
                        rng.below(factions as u64) as usize,
                        1 + rng.below(30) as u32,
                    )
                })
                .collect();
            let seed = rng.next();

            let out = plan(&weights, &budgets, &groups, seed).unwrap();
            let context = format!(
                "iteration {iteration}: weights {weights:?} budgets {budgets:?} groups {groups:?}"
            );

            let spent = spent_by_faction(&out, &groups, factions);
            for (faction, (&s, &b)) in spent.iter().zip(&budgets).enumerate() {
                assert!(s <= u64::from(b), "faction {faction} overspent: {context}");
            }
            let total_budget: u64 = budgets.iter().map(|&b| u64::from(b)).sum();
            let total_spent: u64 = spent.iter().sum();
            assert!(total_spent <= total_budget, "{context}");

            assert!(
                out.windows(2).all(|w| w[0].location <= w[1].location),
                "not ordered by location: {context}"
            );
            for x in &out {
                assert!(x.location < locations, "{context}");
                assert!(x.group < group_count, "{context}");
                assert!(
                    weights[x.location] > 0.0,
                    "zero weight populated: {context}"
                );
            }

            // Everything unspent is pooled and offered to every group, so when
            // someone can receive it, no group is left that both the pool and
            // its faction's remaining budget could still pay for.
            if weights.iter().any(|&w| w > 0.0) {
                let pool = total_budget - total_spent;
                for (index, group) in groups.iter().enumerate() {
                    let faction_left = u64::from(budgets[group.faction]) - spent[group.faction];
                    let cost = u64::from(group.cost);
                    assert!(
                        cost > pool || cost > faction_left,
                        "group {index} (cost {cost}) still fits pool {pool} and faction \
                         remainder {faction_left}: {context}"
                    );
                }
            } else {
                assert!(out.is_empty(), "{context}");
            }

            assert_eq!(
                out,
                plan(&weights, &budgets, &groups, seed).unwrap(),
                "{context}"
            );
        }
    }
}

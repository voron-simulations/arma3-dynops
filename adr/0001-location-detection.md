# 1. Span-limited recursive single-linkage clustering for location detection

## Status

Accepted

## Context

DynOps needs to find every settlement on a map -- from a 2-3 house farm up to
a city -- so downstream systems (population, intel, dynsim) can target them.

The existing `cluster::entrypoint` path (`EPSILON = 100`, `MIN_POINTS = 6`,
DBSCAN) fails this on two counts, measured against the bundled building dumps
in `data/objects.*.txt`:

- `MIN_POINTS = 6` structurally excludes 2-3 house farms: a farm can never
  have 6 mutual neighbours within `eps` if it only has 2-5 buildings.
- A single fixed `EPSILON` chains settlements together once the map is dense
  enough. Single-linkage clustering (which is what DBSCAN's core/edge
  reachability reduces to once every point is within `eps` of *some* other
  point) has no notion of "this component got big enough, stop merging" --
  it only stops at gaps wider than `eps`. On Altis the largest such
  component is 980x1350m at eps=40, 1540x1721m at eps=60, and 2468x3688m at
  eps=100 -- a tenth of the map, chained through the sparse buildings that
  connect otherwise-distinct towns.

Lowering `eps` shrinks the worst-case chain but starts fragmenting genuine
settlements (a village's outskirts drop below the connectivity threshold
before its unrelated neighbour's chain breaks), and there is no single `eps`
that works for every settlement on every map: Stratis's map-average building
density is far lower than Altis's.

## Decision

Cut the single-linkage dendrogram at a *settlement-sized span* instead of a
fixed radius, and use `min_points = 2` (not 6) so farms survive.

1. Run `cluster::cluster(eps, 2, points)` (`src/cluster.rs`). With
   `min_points = 2`, `KdTree::within` always includes the query point itself,
   so a point with one neighbour is `Core` and an isolated point is `Noise` --
   this *is* connected-components at radius `eps`, and singletons drop out
   for free with no new clustering code.
2. For each resulting component, compute its axis-aligned span
   (`max(width, height)`). If `span > max_span` **and** `eps > min_eps`,
   recurse on that subset at `eps * split_factor`. Otherwise emit it if
   `len >= min_buildings`.
3. `geometry::min_area_obb` per emitted component (convex hull + rotating
   calipers -- see below), classified by building count.

Recursing shrinks `eps` only where a component is actually oversized, so a
dense, compact village is emitted after step 1 with no recursion, while a
long sparse chain accreted through empty countryside keeps getting re-cut
until either it fragments into settlement-sized pieces or `eps` bottoms out
at `min_eps` (in which case it is emitted as-is, oversized -- see the "unless
min_eps was reached" caveat in `tests/integration.rs`).

Defaults, tuned against all six bundled maps and locked in as
`DetectParams::default()`: `eps = 100`, `max_span = 700`, `split_factor =
0.7`, `min_eps = 35`, `min_buildings = 2`.

### Measured results

Recursive cut, defaults above:

| Map | locations | farms 2-4 | hamlets 5-12 | villages 13-40 | towns 41-120 | cities 120+ | max span |
|---|---|---|---|---|---|---|---|
| Stratis | 31 | 16 | 12 | 1 | 2 | 0 | 570 m |
| Malden | 187 | 100 | 59 | 16 | 10 | 2 | 645 m |
| Altis | 1106 | 786 | 217 | 61 | 23 | 19 | 775 m |

(Reproduced exactly by `tests/integration.rs`'s `test_map_stratis` /
`test_map_malden` / `test_map_altis`, which assert this histogram.)

### Bounding shape: convex hull + rotating calipers, not MVEE

The existing `bounding::bounding_ellipse` (minimum-volume enclosing ellipse,
via an iterative Khachiyan-style fit) is unsuitable for per-settlement boxes:
it is outlier-sensitive, and its inner matrix inversion is singular below 3
points -- exactly the farm case this feature exists to handle, and it
silently returns a garbage ellipse rather than failing loudly. `geometry::
min_area_obb` (Andrew's monotone chain hull, O(n log n), then a rotating
calipers pass over the hull edges) is exact, deterministic, and well-defined
for n = 0 (returns `None`, making the empty case unrepresentable downstream),
n = 1, n = 2, and collinear/duplicate input. It also maps directly onto an
Arma `RECTANGLE` marker/location (`setMarkerSize [a, b]` + `setMarkerDir`),
where an ellipse would need translating.

## Alternatives considered

- **Fixed-eps DBSCAN** (the status quo `cluster::entrypoint` path, `eps =
  100`, `min_points = 6`): rejected per Context above -- fails both on small
  farms and on eps-driven chaining.
- **HDBSCAN**: the principled fix for exactly this failure mode (it derives
  a stability-based cut of the single-linkage dendrogram per-branch, instead
  of a single global `eps` or a single global `max_span`), and is the
  natural next step if per-map tuning of `max_span`/`split_factor`/`min_eps`
  ever turns out to be necessary. Not implemented here: the span-limited
  recursive cut is measured (table above) to already produce a clean,
  plausible settlement histogram across all six bundled maps with one shared
  set of defaults, and HDBSCAN is a materially larger dependency and
  implementation surface for a problem that, so far, doesn't need it.

## Consequences

- A location's span can exceed `max_span` in the rare case where recursion
  bottoms out at `min_eps` without fragmenting (a dense, sparse-gapped ribbon
  of buildings, e.g. along a coastal road). This is expected and bounded --
  see `tests/integration.rs`'s span-ceiling check -- not a bug.
- `min_buildings = 2` means single isolated buildings are never their own
  location. If a lone building genuinely needs tracking, it needs a
  different mechanism (this component only detects clusters).
- Detection is recomputed from scratch each run (no persistence across
  missions); `onMissionEnded` clears the extension's session state so a
  restarted mission can't page into stale data.

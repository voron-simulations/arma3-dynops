# 2. Budget-driven population of faction-owned locations

## Status

Accepted

## Context

Location detection (ADR 0001) yields every settlement on the map, annotated
with a size class and flags (`isMilitary`, `isAirport`, `hasHospital`). The
next consumer is population: each faction should field a force whose total
size is controlled by one number (a budget), spread over the locations that
faction owns, so a mission maker can dial the whole map's AI density up or
down without hand-placing groups.

Three facts constrain the design:

- Nothing in the addon currently records who owns a location. Ownership has
  to be introduced before population can consume it.
- `GVARMAIN(FactionData)` (addons/factions) already indexes every faction's
  CfgGroups entries by type (`Group_Infantry`, `Group_Motorized`, ...) and
  every CfgVehicles unit by `BIS_fnc_objectType` category, so the raw
  material for "what can this faction spawn" exists.
- The project convention (ADR 0001) is to keep decision logic in the Rust
  extension, where it is deterministic, unit-tested and benchmarked, and
  keep SQF to collection and side effects.

## Decision

### Ownership is per side, budget is per faction

Ownership is a *side* attribute, not a faction one: Arma's model is that
sides fight and factions compose a side's force, and a mission maker wants
to say "a large NATO force with some CTRG support" without pre-deciding
which villages CTRG holds. Each entry in `GVAR(Locations)` gains an `owner`
key whose value is a side name (`"WEST"`, `"EAST"`, `"GUER"`) or `""` for
unowned. The same value is mirrored on the real `Location` object as the
variable `dynops_locations_Owner` so `nearestLocations` consumers can read
it without the hashmap.

Initial assignment, in `fnc_assignOwners` (addons/locations):

1. The mission variable `dynops_locations_defaultOwner` (a side name, set
   in `init.sqf`/`initServer.sqf`; default `"EAST"`) owns every location to
   start with.
2. Editor markers named `dynops_owner_<SIDE>` with any suffix (e.g.
   `dynops_owner_WEST_1`, `dynops_owner_none` to make an area unowned)
   override: every location whose centre lies inside the marker area is
   reassigned. Markers are processed in name order so later markers win on
   overlap.

Budgets are per faction, in the mission variable
`dynops_population_budgets`, an array of `[factionClass, points]` pairs
(e.g. `[["BLU_F", 900], ["BLU_CTRG_F", 100], ["OPF_F", 500]]`). A
faction's side comes from `GVARMAIN(FactionData)`. Allocation itself runs
once per *side*: the side's total budget is the sum of its factions'
budgets and is spread over the locations the side owns, and the group
catalogue is the union of all its factions' groups. Faction budgets then
only steer which faction a picked group comes from, so "900 NATO / 100
CTRG" places roughly nine NATO groups for every CTRG group (when their
groups cost about the same), mixed through the same settlements.

Ownership changes at runtime (capture) are out of scope here, but the data
layout (one side string per location, mirrored on the object) is what a
capture system would mutate.

### Unit cost comes from a project-owned table

A new config class, `CfgDynOpsCosts`, maps `BIS_fnc_objectType` category
pairs (the same `"<type>_<subtype>"` keys FactionData already uses, e.g.
`Soldier_Infantry`, `Vehicle_Car`, `Vehicle_APC`) to point costs. A
CfgGroups group's cost is the sum of its members' unit costs; unknown
categories fall back to `CfgDynOpsCosts >> "default"`. Costs are read once
in `fnc_initPopulation` and cached per faction as
`[[groupClass, cost], ...]`.

Rejected: using CfgVehicles' `cost` field directly. It is tuned for AI
threat evaluation, varies wildly between mods, and would make the budget
number meaningless to a mission maker.

### Allocation is computed in the Rust extension

A `population` module and a `population:*` command group, mirroring the
`locations:*` session protocol:

| Command | Args | Effect |
|---|---|---|
| `population:begin` | `seed: u64` | new `Planning` session |
| `population:addLocations` | chunk of `weight\n...` (index = order sent) | accumulate candidate locations |
| `population:addFactions` | chunk of `budget\n...` (index = order sent) | accumulate the side's faction budgets |
| `population:addGroups` | chunk of `factionIdx,cost\n...` (index = order sent) | accumulate the side-wide group catalogue |
| `population:plan` | -- | compute, transition to `Planned`, return assignment count |
| `population:page` | `offset` | `[nextOffset, [[locIdx, groupIdx], ...]]` |
| `population:end` | -- | clear |

SQF calls this once per side. Indices are positions in the order the
caller sent them, so the extension never sees class names or positions; it only sees
weights and costs.

Algorithm (`population::plan(weights, faction_budgets, groups, seed)`):

1. **Location weight** is supplied by SQF (`fnc_locationWeight`): base
   weight by class (Farm 1, Hamlet 2, Village 4, Town 8, City 16), x3 if
   `isMilitary`, x2 if `isAirport`. Keeping the weight formula in SQF keeps
   mod-specific flags out of the extension.
2. **Budget split**: the side budget is `sum(faction_budgets)`; each
   location gets `budget * weight / sum(weights)`,
   rounded down, with the remainder distributed one point at a time to the
   highest-weight locations. This is exact and deterministic.
3. **Group selection** per location: a seeded PRNG (splitmix64, no new
   dependency) repeatedly picks a group until none fits. A candidate group
   must fit both the location's remaining share and its faction's remaining
   budget; the faction is drawn first, weighted by remaining faction budget,
   then a candidate group of that faction uniformly. Leftover points pool
   back into a global remainder that is spent the same way over all
   locations in weight order, so a budget is never silently wasted when a
   cheap group exists.
4. Output is the flat list of `(locIdx, groupIdx)` assignments.

Properties the tests pin: total spent `<= budget` and per-faction spend
`<=` that faction's budget; two factions with budgets 9 and 1 and
equal-cost groups yield exactly ten groups, nine from the first; no location receives a
group when its share is below the cheapest group and the global pool is
empty; same inputs + seed give identical output; `budget = 0` or an empty
catalogue yields no assignments; a catalogue whose cheapest group exceeds
the budget yields no assignments (not a panic).

### Spawning (first iteration: infantry only)

`fnc_populateLocation` spawns each assigned group with `BIS_fnc_spawnGroup`
at the location centre, then:

- if the location has `>= 4` building positions, the group is given a
  garrison pattern (units moved into `buildingPos` slots, `doStop`), else
- it gets a cycling patrol of 3-5 waypoints inside the location OBB.

Only `Group_Infantry` is used in this iteration. Motorized/armour groups
need road and open-ground placement that detection does not yet provide,
and are deferred. LAMBS is not a dependency; if `lambs_wp_fnc_taskGarrison`
exists it is preferred over the vanilla garrison, detected at runtime.

Every spawned group is tagged `dynops_population_Location` (the location
hashmap) and `dynops_population_Faction`, and `GVAR(Groups)` lists them so
a later cleanup/respawn system has a handle.

### Orchestration

`fnc_initPopulation` (addons/population, server-only, runs after
`DynOps_fnc_initLocations` via `XEH_postInit` ordering on
`dynops_locations`):

1. `fnc_assignOwners` on `GVAR(Locations)`.
2. Group `dynops_population_budgets` entries by side. For each side that
   owns at least one location: build the side-wide catalogue (each group
   tagged with its faction index) with costs, drive the `population:*`
   protocol over the side's locations, spawn.
3. Publish `GVAR(Groups)` and log per-faction spend.

## Alternatives considered

- **SQF-only allocation.** Less wiring, but the allocation is exactly the
  kind of pure, parameter-sensitive logic that benefits from fast
  unit tests and benches outside Arma, and determinism via a seed is much
  easier to guarantee in Rust than with SQF's `random`.
- **Per-unit (not per-group) spawning.** Finer budget granularity, but
  loses CfgGroups' authored composition (leaders, AT, medics) and makes
  each spawned squad a random bag of soldiers.
- **Proportional-to-buildings weighting instead of class-based.** Simpler,
  but cities would soak up almost the entire budget on Altis (19 cities vs
  786 farms), leaving the countryside empty. Class steps with military /
  airport multipliers are coarser but controllable.

## Consequences

- Introduces a second extension session type. `Mutex<Option<Session>>`
  becomes an enum over `Locations(..)` / `Population(..)` sessions, or a
  second mutex; either way the protocol-error-on-misuse guarantee from
  ADR 0001 is kept and tested.
- Budgets are plain mission variables for now. A future
  campaign/persistence layer would replace them with tracked values.
- Factions on one side share a single allocation, so a faction with a
  small budget shows up as a sprinkling of groups among the larger
  faction's, not as its own enclave. Enclaves need ownership markers plus a
  per-side budget split, which this design does not provide.
- Group cost depends on `CfgDynOpsCosts`, which ships with vanilla
  defaults; mods with exotic unit categories fall through to `default`
  cost, which may over- or under-value them until the table is extended.
- Capture/ownership change, vehicles, statics and respawn are explicitly
  deferred and each gets its own ADR when picked up.

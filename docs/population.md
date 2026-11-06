# Population

Spreads AI groups over the locations a side owns, from one budget per
faction. The allocation is computed in the Rust extension from plain weights,
budgets and costs; SQF collects those inputs and spawns the result. See
[`adr/0002-population.md`](../adr/0002-population.md) for the design,
ownership model and cost model, and [`docs/locations.md`](locations.md) for
the locations this consumes.

Allocation runs once per *side*. The side's locations get one plan, and its
factions' budgets only steer which faction each placed group comes from: a
budget of 900 for one faction and 100 for another places roughly nine groups
of the first for every one of the second (when their groups cost about the
same), mixed through the same settlements.

## Extension protocol

`population:*` follows the same chunked session pattern as `locations:*`
(see [`docs/locations.md`](locations.md#extension-protocol) for why): the
state lives in the extension's single `Mutex<Option<Session>>` slot
(`src/lib.rs`), and every command either requires a specific state or
transitions between them. Calling one out of order is a protocol error
(`Result::Err`), not undefined behaviour.

The session has two population states, `Planning` (accumulating weights,
faction budgets and groups) and `Planned` (holding the result). SQF runs one
session per side.

| Command | Args | Returns | Requires | Leaves session in |
|---|---|---|---|---|
| `population:begin` | `seed: u64` | `"OK"` | any | `Planning` (empty), replacing whatever session existed |
| `population:addLocations` | `chunk: String` ("weight\nweight...") | locations accumulated so far | `Planning` | `Planning` |
| `population:addFactions` | `chunk: String` ("budget\nbudget...") | factions accumulated so far | `Planning` | `Planning` |
| `population:addGroups` | `chunk: String` ("factionIdx,cost\n...") | groups accumulated so far | `Planning` | `Planning` |
| `population:plan` | -- | assignment count; invalid input returns a protocol error | `Planning` | `Planned` on success; unchanged (`Planning`) on invalid input |
| `population:page` | `offset: usize` | `[nextOffset, [[locIdx, groupIdx], ...]]` | `Planned` | `Planned` |
| `population:end` | -- | `"OK"` | any | cleared (`None`) |

There is one session slot shared with `locations:*`, so `population:begin`
replaces a `locations:*` session and `locations:begin` replaces a population
one. Run the two pipelines one after the other, never interleaved. A call
that does not match the current state is rejected without touching the
session, so a stray call never destroys a session it was not meant for.

`onMissionEnded` (see [`docs/locations.md`](locations.md#extension-protocol))
clears the slot whichever protocol filled it.

### Chunk formats

One line per entry, blank lines ignored, whitespace around values and around
the comma is trimmed. A chunk is parsed completely before it is applied, so a
bad line rejects the whole chunk and leaves the session as it was.

| Command | Line | Index of an entry |
|---|---|---|
| `population:addLocations` | `weight`: a finite, non-negative decimal (`1`, `16`, `2.5`). Weight 0 means "never populated". | order sent, across chunks, from 0 |
| `population:addFactions` | `budget`: a non-negative integer, the faction's points. | order sent, across chunks, from 0 |
| `population:addGroups` | `factionIdx,cost`: the faction's index as sent to `population:addFactions`, and a positive integer point cost. | order sent, across chunks, from 0 |

The extension never sees class names or positions. SQF keeps its own arrays of
locations, factions and groups in the same order and maps the returned
indices back. Factions may be sent before or after the groups that name them.

`callExtension` caps an argument at 10240 bytes, so send long lists in
several chunks (a few hundred lines each).

Weights, costs, faction indices and the output size are validated at
`population:plan`, not when added. A protocol error is returned for:

- a weight that is NaN, infinite or negative, or a cost of 0 (a group that
  would never exhaust a budget), naming the offending index;
- a `factionIdx` with no matching `population:addFactions` entry, naming the
  group and the faction;
- budgets that could place more than 100,000 groups (`MAX_ASSIGNMENTS`), that
  is, the sum of the faction budgets divided by the cheapest group's cost is
  above the limit. The check ignores the locations, so a call is accepted or
  rejected the same way whatever it is planned over.

The session stays in `Planning`. A missing faction can be repaired by adding
the budget and planning again; a bad weight, cost or budget is still in the
session, so SQF should `population:begin` again.

### Planning

`population:plan` runs `population::plan` in `src/population.rs`. The side's
total budget is the sum of the faction budgets. It is split over the locations
in proportion to weight, then random groups are picked to fit each
location's share, and the points no location could use are spent in a final
round-robin from the heaviest location down. A group may be picked only if
it fits both the location's remaining share and its faction's remaining
budget. The faction is drawn first, weighted by what it has left, and then
one of its affordable groups uniformly. The ADR has the full algorithm and
the properties the tests pin. In short:

- no faction ever spends more than its own budget, so the total never exceeds
  the sum;
- when any location has positive weight, no group is left that both the
  unspent points and its own faction's remaining budget could still pay for;
- a faction with budget 0, or whose groups all cost more than its budget,
  places nothing;
- zero-weight locations get nothing;
- the same `seed`, weights, budgets and groups always give the same result;
- an empty location list, an empty catalogue, a zero total budget, or
  nothing affordable is not an error, it just returns 0 assignments.

The number of assignments is at most the total budget divided by the cheapest
group's cost, which `population:plan` caps at 100,000: they are all held in
memory until `population:end`, so a mistyped budget must not be able to
exhaust the Arma process. Real budgets are hundreds to thousands of points.

`seed` is a `u64`, but SQF numbers are 32-bit floats: it can only pass
integers exactly up to 2^24 (16,777,216), and a larger number may be rounded
when stringified. Keep seeds below that.

### Page entries

Each `population:page` entry is one group placed at one location:

```
[locIdx, groupIdx]
```

- `locIdx`: index into the weights sent with `population:addLocations`.
- `groupIdx`: index into the groups sent with `population:addGroups`. Its
  faction is whatever SQF sent on that line.

Entries are ordered by ascending `locIdx`, then in the order the groups were
picked, so all of one location's groups are contiguous.

`population:page` stops adding entries before the returned string would
exceed 9000 bytes (always including at least one entry, to guarantee forward
progress), well under the 10240-byte hard cap. This is the same rule, and the
same helper, as `locations:page`. Callers page from `offset = 0` until the
returned `nextOffset` stops advancing or reaches the count returned by
`population:plan`. A page past the end is valid and empty (`[n,[]]`).

### Example

Two locations (weights 1 and 3), two factions (budgets 900 and 100), and one
cost-5 group each:

```sqf
private _nl = toString [10];
"dynops" callExtension ["population:begin", [42]];
"dynops" callExtension ["population:addLocations", [["1", "3"] joinString _nl]];
"dynops" callExtension ["population:addFactions", [["900", "100"] joinString _nl]];
"dynops" callExtension ["population:addGroups", [["0,5", "1,5"] joinString _nl]];
private _count = parseNumber (("dynops" callExtension ["population:plan", []]) select 0);

private _offset = 0;
while { _offset < _count } do {
    private _page = parseSimpleArray (("dynops" callExtension ["population:page", [_offset]]) select 0);
    _page params ["_next", "_entries"];
    { _x params ["_locIdx", "_groupIdx"]; /* spawn */ } forEach _entries;
    if (_next <= _offset) exitWith {};
    _offset = _next;
};
"dynops" callExtension ["population:end", []];
```

## SQF entry points

Mission-maker setup (budgets, seed, owner markers, in-game cost checks) is in
[`addons/population/README.md`](../addons/population/README.md).

| Function | Role |
|---|---|
| `fnc_assignOwners` (`addons/locations/`) | Sets the `owner` side of every location: `dynops_locations_defaultOwner` first, then `dynops_owner_<SIDE>` editor markers override. Mirrored on the `Location` object as `dynops_locations_Owner`. |
| `fnc_locationWeight` | The weight sent with `population:addLocations`: class base weight (Farm 1, Hamlet 2, Village 4, Town 8, City 16), x3 if `isMilitary`, x2 if `isAirport`. |
| `fnc_groupCost` | A CfgGroups group's cost: the sum of its members' `CfgDynOpsCosts` entries by `BIS_fnc_objectType` category, falling back to the `default` entry. |
| `fnc_initPopulation` (`addons/population/`) | Orchestrator, server-only: groups `dynops_population_budgets` by the faction's side and calls `fnc_populateSide` for each side that owns locations. Publishes `GVAR(Groups)`. |
| `fnc_populateSide` | One side's run: builds the side-wide catalogue (each group tagged with its faction index and cost), drives the `population:*` protocol once, pages the assignments back and spawns each via `fnc_populateLocation`. |
| `fnc_populateLocation` | Spawns each group assigned to one location with `BIS_fnc_spawnGroup`: garrisons it in building positions if the location has 4 or more, else gives it a cycling patrol inside the location. |

# Location detection

Detects every settlement on the current map (farms through cities) from
building positions, and exposes them both as plain data and as real Arma
`Location` objects. See [`adr/0001-location-detection.md`](../adr/0001-location-detection.md)
for the detection algorithm and its measured accuracy.

## Extension protocol

`callExtension` caps a single call's return value at 10240 bytes. ~1100
locations on Altis serialise to ~50KB, and the input side (building
coordinates) is ~190KB for the same map -- neither direction fits in one
call, so both are chunked through a small session state machine held in the
`dynops` extension (`Mutex<Option<Session>>`, `src/lib.rs`).

The session has two states, `Collecting` (accumulating input points) and
`Detected` (holding the detection result), and every command below either
requires a specific state or transitions between them -- calling one out of
order is a protocol error (`Result::Err`), not undefined behaviour.

| Command | Args | Returns | Requires | Leaves session in |
|---|---|---|---|---|
| `locations:begin` | -- | `"OK"` | any | `Collecting([])` |
| `locations:add` | `chunk: String` ("x,y\nx,y...") | points accumulated so far | `Collecting` | `Collecting` |
| `locations:detect` | `eps, maxSpan, splitFactor, minEps, minBuildings` | location count; invalid params return a protocol error | `Collecting` | `Detected` on success; unchanged (`Collecting`) on invalid params |
| `locations:page` | `offset: usize` | `[nextOffset, [entry, ...]]` | `Detected` | `Detected` |
| `locations:end` | -- | `"OK"` | any | cleared (`None`) |

`locations:detect` validates its parameters before touching the session
(`DetectParams::new` in `src/locations.rs`): all finite; `eps`, `minEps`,
`maxSpan` > 0; `minEps <= eps`; `0 < splitFactor < 1`; `minBuildings >= 1`;
and `eps` must reach `minEps` within 64 `splitFactor` steps. Anything else is
a protocol error naming the offending parameter, and the session stays in
`Collecting` so the caller can retry.

`onMissionEnded` (a flat, non-grouped command registered for the `Ended`
mission event handler in `addons/extension/XEH_preInit.sqf`) also clears the
session, so a stale one from a previous mission can never be paged into.

Each `locations:page` entry, already in SQF marker order:

```
[[x, y], a, b, dirDegrees, buildingCount, classIndex]
```

- `a`, `b`: OBB half-extents in metres (matches `setMarkerSize`'s half-size
  convention).
- `dirDegrees`: Arma marker/location direction in degrees, clockwise. At 0,
  `a` lies along east and `b` along north; it is not a compass heading of the
  `a` axis. Already converted from the OBB's internal radians-CCW-from-+X
  representation by `geometry::obb_to_marker_dir` (negation) -- SQF never does
  angle math.
- `classIndex`: `0` Farm (2-4 buildings), `1` Hamlet (5-12), `2` Village
  (13-40), `3` Town (41-120), `4` City (120+).

`locations:page` stops adding entries before the returned string would
exceed 9000 bytes (always including at least one entry, to guarantee forward
progress), well under the 10240-byte hard cap. Callers page from `offset =
0` until the returned `nextOffset` stops advancing or reaches the total
count from `locations:detect`.

## SQF entry points (`addons/locations/`)

| Function | Role |
|---|---|
| `fnc_collectBuildings` | The one building scan (`nearestTerrainObjects` + `BIS_fnc_isBuildingEnterable`). Called once per pipeline run and threaded through, rather than re-scanned by every downstream function. |
| `fnc_detectLocations` | Drives the chunked protocol above: `begin`, batched `add` (400 buildings/chunk), `detect`, `page` to exhaustion, `end`. Returns an array of hashmaps (`pos`, `a`, `b`, `dir`, `buildings`, `class`). |
| `fnc_annotateLocations` | Enriches each detection in place with `name`/`type` (adopted from a real map-config location if its position falls inside the detection's OBB, else synthesised from class + `BIS_fnc_locationDescription`), `isAirport`, `isMilitary`, `hasHospital`. |
| `fnc_assignOwners` | Sets the `owner` key on every location (`"WEST"`, `"EAST"`, `"GUER"` or `""` for unowned): the mission variable `dynops_locations_defaultOwner` (default `"EAST"`) first, then `dynops_owner_<SIDE>*` editor markers override it for every location whose centre is inside the marker. See [Ownership](#ownership). |
| `fnc_createLocationObjects` | Creates real Arma `Location` objects, gated by a minimum class (default `1`, Hamlet) -- 1106 locations on Altis, most of them 2-4 building farms, would slow down every `nearestLocations` call in the mission if all became real Locations. Mirrors each location's owner onto the object as the variable `dynops_locations_Owner`. |
| `fnc_drawLocationMarkers` | Debug tool. Runs the full pipeline standalone and draws one `RECTANGLE` marker per location, coloured by class, labelled with name + building count. Clears its own previously-drawn markers first, so it's safe to re-run from the debug console. |
| `fnc_initLocations` | Orchestrator: collects buildings once, runs detect + annotate + assign owners + create, and publishes `GVAR(EnterableBuildings)`, `GVAR(Locations)` (all detections, annotated), and `GVAR(LocationObjects)` (the gated real `Location`s). Called from `XEH_postInit.sqf`, server-guarded. |

### Ownership

Every location has an `owner` key naming the side that holds it, which
population uses to decide whose force goes where
([`adr/0002-population.md`](../adr/0002-population.md)). The same value is
readable from a `Location` object as `dynops_locations_Owner`, without going
through the hashmap:

```sqf
(nearestLocation [player, "NameVillage"]) getVariable "dynops_locations_Owner"   // "EAST"
```

Initial assignment, in `fnc_assignOwners`:

1. Every location is owned by `dynops_locations_defaultOwner` (a side name,
   `"WEST"`, `"EAST"`, `"GUER"` or `"none"`; default `"EAST"`).
2. Editor markers named `dynops_owner_<SIDE>` plus any suffix
   (`dynops_owner_WEST_1`, `dynops_owner_none`) override it for every location
   whose centre is inside the marker. Use rectangle or ellipse markers. `<SIDE>`
   is case-insensitive and read up to the next `_`. Markers apply in name
   order, so on overlap the later name wins. A marker with an invalid side is
   logged and ignored.

Budgets are per faction but placement is per side: population plans each
side's factions together over the locations that side owns, see
[`addons/population/README.md`](../addons/population/README.md).

`fnc_initLocations` runs once the mission clock has started (`time > 0`), after
`init.sqf` and `initServer.sqf` have been launched, so the default can be set
there.

### Usage

```sqf
// Debug: visualize detections on the map without touching mission state.
call DynOps_fnc_drawLocationMarkers;

// Production: runs automatically via XEH_postInit on mission start.
// Downstream code reads:
count DynOps_locations_Locations        // every detection, as hashmaps
count DynOps_locations_LocationObjects  // real Location objects (Hamlet+)
nearestLocations [player, ["NameVillage"], 5000]
```

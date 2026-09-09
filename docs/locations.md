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
| `locations:detect` | `eps, maxSpan, splitFactor, minEps, minBuildings` | location count | `Collecting` | `Detected` |
| `locations:page` | `offset: usize` | `[nextOffset, [entry, ...]]` | `Detected` | `Detected` |
| `locations:end` | -- | `"OK"` | any | cleared (`None`) |

`onMissionEnded` (a flat, non-grouped command registered for the `Ended`
mission event handler in `addons/extension/XEH_preInit.sqf`) also clears the
session, so a stale one from a previous mission can never be paged into.

Each `locations:page` entry, already in SQF marker order:

```
[[x, y], a, b, dirDegrees, buildingCount, classIndex]
```

- `a`, `b`: OBB half-extents in metres (matches `setMarkerSize`'s half-size
  convention).
- `dirDegrees`: compass degrees (0 = north, clockwise), already converted
  from the OBB's internal radians-CCW-from-+X representation by
  `geometry::obb_to_marker_dir` -- SQF never does angle math.
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
| `fnc_createLocationObjects` | Creates real Arma `Location` objects, gated by a minimum class (default `1`, Hamlet) -- 1106 locations on Altis, most of them 2-4 building farms, would slow down every `nearestLocations` call in the mission if all became real Locations. |
| `fnc_drawLocationMarkers` | Debug tool. Runs the full pipeline standalone and draws one `RECTANGLE` marker per location, coloured by class, labelled with name + building count. Clears its own previously-drawn markers first, so it's safe to re-run from the debug console. |
| `fnc_initLocations` | Orchestrator: collects buildings once, runs detect + annotate + create, and publishes `GVAR(EnterableBuildings)`, `GVAR(Locations)` (all detections, annotated), and `GVAR(LocationObjects)` (the gated real `Location`s). Called from `XEH_postInit.sqf`, server-guarded. |

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

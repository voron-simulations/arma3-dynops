# DynOps Population

Spawns infantry groups across the locations each side owns, with the size of
each force controlled by budget numbers. Design and rationale:
[`adr/0002-population.md`](../../adr/0002-population.md). Location ownership:
[`docs/locations.md`](../../docs/locations.md).

## Mission setup

Set these in `init.sqf` or `initServer.sqf`. Population starts once the
mission clock is running, after those scripts have been launched.

```sqf
dynops_locations_defaultOwner = "EAST";   // WEST, EAST, GUER or "none"; owner of every location by default
dynops_population_budgets = [["BLU_F", 900], ["BLU_CTRG_F", 100], ["OPF_F", 500]];   // [faction class, points]
dynops_population_seed = 1;               // optional; same seed, same layout
```

Override ownership for an area with an editor marker (rectangle or ellipse,
set its alpha to 0 to hide it) named `dynops_owner_<SIDE>` plus any suffix,
for example `dynops_owner_WEST_1` or `dynops_owner_none`. Every location whose
centre is inside the marker is reassigned. Markers apply in name order, so on
overlap the later name wins.

Budgets are per faction, but placement is per side. All factions of a side
are planned together over the locations that side owns, so their groups end up
mixed across the same settlements. A faction spends at most its own budget, and
only on its own groups, so the budgets set the mix. With `["BLU_F", 900]` and
`["BLU_CTRG_F", 100]` NATO gets nine times CTRG's points: nine NATO groups for
every CTRG group when the groups cost the same. Locations are weighted by size
(military x3, airport x2).

Group prices come from `CfgDynOpsCosts` in this addon, summed over the group's
soldiers: infantry costs 1 point each, so a nine-man squad costs about 9 and
1000 points buys on the order of a hundred squads, not ten. Add entries from
your own addon for unit categories a mod introduces, anything unlisted costs
`default`.

Spawned groups are listed in `DynOps_population_Groups` and carry the
variables `dynops_population_Location` and `dynops_population_Faction`.

## Verifying in-game

Unit categories come from `BIS_fnc_objectType`. A category that is missing
from `CfgDynOpsCosts` is silently priced at `default` (5), which would make an
infantry squad several times too expensive. Run this in the debug console to
list the categories a faction's infantry groups use that have no price, and
change the faction class as needed:

```sqf
private _missing = createHashMap;
{
	{
		{
			private _tags = (getText (_x >> "vehicle")) call BIS_fnc_objectType;
			private _key = (_tags # 0) + "_" + (_tags # 1);
			if (!isNumber (configFile >> "CfgDynOpsCosts" >> _key)) then { _missing set [_key, true]; };
		} forEach ("true" configClasses _x);
	} forEach ("true" configClasses (_x >> "BLU_F" >> "Infantry"));
} forEach ("true" configClasses (configFile >> "CfgGroups"));
keys _missing
```

An empty result means every unit is priced explicitly. To see what one group
costs:

```sqf
[configFile >> "CfgGroups" >> "West" >> "BLU_F" >> "Infantry" >> "BUS_InfSquad"] call DynOps_fnc_groupCost
```

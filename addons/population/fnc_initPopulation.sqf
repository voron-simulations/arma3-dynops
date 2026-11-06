/*
	Server-side orchestrator: groups the faction budgets by side and populates
	each side's locations in one plan per side.

	Budgets are per faction but placement is per side: a side's groups are
	drawn from all of its factions' catalogues into the locations that side
	owns, and each faction's budget only sets its share of that draw. See
	adr/0002-population.md.

	Reads two mission variables, normally set in init.sqf/initServer.sqf:
		dynops_population_budgets - ARRAY of [faction class, points] pairs,
			e.g. [["BLU_F", 900], ["BLU_CTRG_F", 100], ["OPF_F", 500]];
			default []
		dynops_population_seed - NUMBER, default 0; the same seed over the
			same map and mission variables reproduces the same layout

	An unknown faction is skipped with a warning, as is a side that owns no
	location; neither stops the others.

	Publishes:
		GVAR(Groups) - ARRAY of GROUP, every group spawned
			(tagged dynops_population_Location / dynops_population_Faction)

	Requires DynOps_fnc_initLocations to have run (it reads
	DynOps_locations_Locations).
*/
#include "script_component.hpp"

GVAR(Groups) = [];

if (isNil QEGVAR(locations,Locations)) exitWith {
	WARNING("No location data, population skipped: location detection has not run");
};

private _budgets = missionNamespace getVariable [QGVAR(budgets), []];
private _seed = missionNamespace getVariable [QGVAR(seed), 0];

// Side as str -> [faction, budget] pairs for that side's factions.
private _bySide = createHashMap;
{
	_x params [["_faction", "", [""]], ["_budget", 0, [0]]];
	private _factionData = GVARMAIN(FactionData) getOrDefault [_faction, createHashMap];
	private _side = _factionData getOrDefault ["Side", sideUnknown];

	if (_side in [west, east, resistance, civilian]) then {
		private _entries = _bySide getOrDefault [str _side, []];
		_entries pushBack [_faction, _budget];
		_bySide set [str _side, _entries];
	} else {
		WARNING_1("Skipping '%1': unknown faction",_faction);
	};
} forEach _budgets;

// Fixed side order, not budget order, so the result does not depend on how the
// mission maker happened to list the factions. The index doubles as the seed
// offset (it matches the CfgFactionClasses side numbering), which keeps the
// sides' allocations uncorrelated.
private _sides = [east, west, resistance, civilian];
{
	private _side = _x;
	private _entries = _bySide getOrDefault [str _side, []];

	if (_entries isNotEqualTo []) then {
		// Owner names are "WEST"/"EAST"/"GUER", which is what str gives for
		// those sides.
		private _owned = EGVAR(locations,Locations) select { (_x getOrDefault ["owner", ""]) == str _side };
		private _created = [_side, _entries, _owned, _seed + _forEachIndex] call DynOps_fnc_populateSide;
		GVAR(Groups) append _created;
	};
} forEach _sides;

INFO_2("Population complete: %1 groups for %2 factions",count GVAR(Groups),count _budgets);

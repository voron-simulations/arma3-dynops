/*
	Populates every location one side owns in a single plan. Allocation is per
	side so that a side's locations are filled from one shared pool of groups;
	the per-faction budgets only set how that pool is drawn from. With budgets
	of 900 for NATO and 100 for CTRG, roughly nine in ten placed groups are
	NATO and one in ten CTRG, however many locations there are.

	The side-wide catalogue is built here, sent to the extension ("population:*"
	protocol, see adr/0002-population.md), and the resulting assignments are
	spawned. The allocation itself is not done in SQF: it is pure,
	seed-sensitive logic that is unit-tested and benchmarked in Rust, which only
	ever sees weights, budgets and costs, never positions or class names.

	Only the Infantry group type is used for now; motorised and armoured groups
	need road and open-ground placement that detection does not provide.

	callExtension caps a single call's input and output, so entries are sent in
	batches and assignments are paged back out, the same way
	DynOps_fnc_detectLocations does it.

	Params:
		0: SIDE - the side being populated; only used for logging and to keep
			the call site self-describing
		1: ARRAY - [faction class, budget in points] pairs for the factions on
			this side. A faction with a budget below 1 or without a usable
			infantry group is dropped with a warning, because it could never
			be drawn.
		2: ARRAY of HashMap - candidate locations, already filtered to those
			this side owns
		3: NUMBER (optional) - seed, default 0. Callers pass a different one
			per side so two sides' allocations are not correlated.

	Returns: ARRAY of GROUP - the spawned groups (empty if nothing could be
		spawned)
*/
#include "script_component.hpp"

params [
	["_side", sideUnknown, [sideUnknown]],
	["_factionBudgets", [], [[]]],
	["_locations", [], [[]]],
	["_seed", 0, [0]]
];

// Same batch size as fnc_detectLocations; one short line per entry keeps a
// chunk far below callExtension's input limits.
private _batchSize = 400;

// CfgGroups nests factions under a side class (West/East/Indep/Civilian) and
// FactionData does not record which one, so look for the side that holds the
// faction's Infantry groups instead of maintaining a side-name table.
private _findInfantryConfig = {
	params ["_faction"];
	private _found = configNull;
	{
		private _candidate = _x >> _faction >> "Infantry";
		if (isClass _candidate) exitWith { _found = _candidate; };
	} forEach ("true" configClasses (configFile >> "CfgGroups"));
	_found
};

// The extension's faction index is the position in _factions, so the list
// only holds factions that can actually be drawn from: a faction with no
// usable group would otherwise carry budget that is silently never spent.
private _factions = [];
private _budgets = [];
// [groupConfig, factionIndex, cost] for every usable group of every faction.
private _catalogue = [];
{
	_x params [["_faction", "", [""]], ["_budget", 0, [0]]];
	private _infantry = [_faction] call _findInfantryConfig;
	private _groupClasses = (GVARMAIN(FactionData) getOrDefault [_faction, createHashMap]) getOrDefault ["Group_Infantry", []];
	private _groupConfigs = (_groupClasses apply { _infantry >> _x }) select {
		isClass _x && { count ("true" configClasses _x) > 0 }
	};

	if (_budget < 1 || { _groupConfigs isEqualTo [] }) then {
		WARNING_3("Faction %1 ignored: budget %2, %3 usable infantry groups",_faction,_budget,count _groupConfigs);
	} else {
		private _factionIndex = count _factions;
		_factions pushBack _faction;
		_budgets pushBack (round _budget);
		{ _catalogue pushBack [_x, _factionIndex, [_x] call DynOps_fnc_groupCost]; } forEach _groupConfigs;
	};
} forEach _factionBudgets;

if (_catalogue isEqualTo [] || _locations isEqualTo []) exitWith {
	WARNING_3("Side %1 not populated: %2 usable infantry groups, %3 owned locations",_side,count _catalogue,count _locations);
	[]
};

// Sends one line per entry, in batches, so a large catalogue never exceeds
// callExtension's input limit.
private _sendChunked = {
	params ["_command", "_lines"];
	private _sent = 0;
	while { _sent < count _lines } do {
		[_command, [(_lines select [_sent, _batchSize]) joinString endl]] call DynOps_fnc_call;
		_sent = _sent + _batchSize;
	};
};

// SQF prints numbers with 7 significant digits and switches to scientific
// notation beyond that, which the extension cannot parse as an integer, so
// keep the seed well below that.
["population:begin", [_seed mod 1000000]] call DynOps_fnc_call;
["population:addLocations", _locations apply { str ([_x] call DynOps_fnc_locationWeight) }] call _sendChunked;
["population:addFactions", _budgets apply { str _x }] call _sendChunked;
["population:addGroups", _catalogue apply { format ["%1,%2", _x # 1, _x # 2] }] call _sendChunked;

private _assignmentCount = parseNumber (["population:plan"] call DynOps_fnc_call);

private _assignments = [];
private _offset = 0;
while { _offset < _assignmentCount } do {
	private _page = parseSimpleArray (["population:page", [_offset]] call DynOps_fnc_call);
	// An unparseable page (extension error) parses to [], and unpacking it
	// would hand nil to the comparison below.
	if (count _page < 2) exitWith {
		WARNING_2("Side %1: unreadable assignment page at offset %2",_side,_offset);
	};
	_page params ["_nextOffset", "_entries"];
	_assignments append _entries;

	// A page never regresses; if it somehow failed to advance, bail rather
	// than loop forever.
	if (_nextOffset <= _offset) exitWith {};
	_offset = _nextOffset;
};

["population:end"] call DynOps_fnc_call;

private _groups = [];
private _perFaction = _factions apply { 0 };
{
	_x params ["_locationIndex", "_groupIndex"];
	private _inRange = _locationIndex >= 0 && _locationIndex < count _locations
		&& _groupIndex >= 0 && _groupIndex < count _catalogue;

	if (_inRange) then {
		(_catalogue select _groupIndex) params ["_groupConfig", "_factionIndex"];
		private _group = [_locations select _locationIndex, _groupConfig, _factions select _factionIndex] call DynOps_fnc_populateLocation;
		if (!isNull _group) then {
			_groups pushBack _group;
			_perFaction set [_factionIndex, (_perFaction select _factionIndex) + 1];
		};
	} else {
		WARNING_3("Side %1: ignoring out-of-range assignment [%2, %3]",_side,_locationIndex,_groupIndex);
	};
} forEach _assignments;

private _totalBudget = 0;
{ _totalBudget = _totalBudget + _x; } forEach _budgets;
INFO_4("Side %1: budget %2, %3 locations, %4 groups",_side,_totalBudget,count _locations,count _groups);
{
	INFO_3("Side %1, faction %2: %3 groups",_side,_x,_perFaction select _forEachIndex);
} forEach _factions;

_groups;

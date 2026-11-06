/*
	Spawns one CfgGroups group at a location and gives it something to do:
	garrisons the location's buildings when there are enough building
	positions, otherwise sets it patrolling inside the location's bounding box.

	If LAMBS Danger is loaded (lambs_wp_fnc_taskGarrison exists) its garrison
	is used in preference to the vanilla one. It is detected at runtime, not
	declared as a dependency, so the addon works without it.

	The group is tagged so a later cleanup/respawn system has a handle:
		dynops_population_Location - the location HashMap it was spawned for
		dynops_population_Faction - the faction class it belongs to

	Params:
		0: HashMap - the target location, as in DynOps_locations_Locations
		1: CONFIG - the CfgGroups group to spawn
		2: STRING - faction class the group is spawned for; its side comes
			from DynOps_FactionData

	Returns: GROUP - the spawned group, or grpNull if it could not be created
*/
#include "script_component.hpp"

params [
	["_location", createHashMap, [createHashMap]],
	["_groupConfig", configNull, [configNull]],
	["_faction", "", [""]]
];

private _side = (GVARMAIN(FactionData) getOrDefault [_faction, createHashMap]) getOrDefault ["Side", sideUnknown];
// isEqualType would accept sideUnknown too, so test membership instead.
if !(_side in [west, east, resistance, civilian]) exitWith {
	WARNING_1("Cannot populate: no side known for faction '%1'",_faction);
	grpNull
};

private _pos = _location get "pos";
private _a = _location get "a";
private _b = _location get "b";
private _dir = _location get "dir";
private _area = [_pos, _a, _b, _dir, true];
// Circle enclosing the OBB: nearestTerrainObjects has no rotated-box query, so
// search the circle and filter by inArea.
private _radius = sqrt ((_a * _a) + (_b * _b));

private _group = [_pos, _side, _groupConfig] call BIS_fnc_spawnGroup;
if (isNull _group) exitWith {
	WARNING_2("BIS_fnc_spawnGroup failed for %1 at %2",configName _groupConfig,_pos);
	grpNull
};

// Same type list as DynOps_fnc_collectBuildings, so the garrison considers
// exactly the buildings detection counted. Non-enterable ones simply yield no
// positions.
private _houses = nearestTerrainObjects [_pos, ["BUILDING", "HOUSE"], _radius, false, true];
_houses = _houses select { (getPos _x) inArea _area };
private _slots = [];
{ _slots append (_x buildingPos -1); } forEach _houses;

if (count _slots >= 4) then {
	if (isNil "lambs_wp_fnc_taskGarrison") then {
		{
			// Units beyond the available slots stay where they spawned.
			if (_slots isEqualTo []) exitWith {};
			private _slot = _slots deleteAt (floor random count _slots);
			_x setPosATL _slot;
			doStop _x;
			_x disableAI "PATH";
		} forEach units _group;
	} else {
		// Radius is the longer half-extent, not the diagonal: the LAMBS search
		// is a circle, and the diagonal would reach into neighbouring settlements.
		[_group, _pos, _a max _b, [], true, false, -1] call lambs_wp_fnc_taskGarrison;
	};
} else {
	// Rejection sampling inside the rotated box: inArea owns the angle
	// convention, so no marker-direction math is repeated here. Water is
	// rejected so coastal locations do not send infantry swimming.
	private _randomPoint = {
		private _point = _pos;
		for "_try" from 1 to 20 do {
			private _candidate = _pos getPos [_radius * sqrt random 1, random 360];
			if ((_candidate inArea _area) && { !surfaceIsWater _candidate }) exitWith { _point = _candidate; };
		};
		_point
	};

	for "_i" from 1 to (3 + floor random 3) do {
		private _waypoint = _group addWaypoint [call _randomPoint, 0];
		_waypoint setWaypointType "MOVE";
	};
	private _cycle = _group addWaypoint [_pos, 0];
	_cycle setWaypointType "CYCLE";
};

_group setVariable [QGVAR(Location), _location];
_group setVariable [QGVAR(Faction), _faction];
_group deleteGroupWhenEmpty true;

_group;

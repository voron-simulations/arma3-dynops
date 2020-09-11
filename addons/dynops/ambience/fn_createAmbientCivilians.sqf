params ["_location"];

private _area = _location call DW_fnc_toArea;
private _buildings = _area call DW_fnc_buildingsInArea;

if (count _buildings <= 3) exitWith { "Less than 3 buildings in location" call DW_fnc_log; };

private _module_group = createGroup sideLogic;
private _pos = _area # 0;
private _size = [_area # 1, _area # 2];

// Create spawnpoint and waypoint modules
private _selected_buildings = [_buildings, 10, true] call DW_fnc_randomSubset;
private _ModuleCivilianPresenceUnit_F = [];
private _ModuleCivilianPresenceSafeSpot_F = [];

{
	private _pos = _x buildingExit 0;
	private _spawnpoint = _module_group createUnit ["ModuleCivilianPresenceUnit_F", _pos, [], 0, "NONE"];
	private _safespot = _module_group createUnit ["ModuleCivilianPresenceSafeSpot_F", _pos, [], 0, "NONE"];
	_safespot setVariable ["#capacity",5];
	_safespot setVariable ["#usebuilding",true];
	_safespot setVariable ["#terminal",true];
	_safespot setVariable ["#type", 1];

	_ModuleCivilianPresenceUnit_F pushBack _spawnpoint;
	_ModuleCivilianPresenceSafeSpot_F pushBack _safespot;
} forEach _selected_buildings;

// Create presence module
_civ_module = _module_group createUnit ["ModuleCivilianPresence_F",	_pos, [], 0, "NONE"];

_civ_module setVariable ["#useagents", true];
//_civ_module setVariable ["#debug", true];
// _civ_module setVariable ["#oncreated", {_this setUnitLoadout (getUnitLoadout selectRandom civilian_loadouts);}];
_civ_module setVariable ["#usepanicmode", true];
_civ_module setVariable ["#unitcount", 10];
_civ_module setVariable ["ModuleCivilianPresenceUnit_F", _ModuleCivilianPresenceUnit_F];
_civ_module setVariable ["ModuleCivilianPresenceSafeSpot_F", _ModuleCivilianPresenceSafeSpot_F];
_civ_module setVariable ["#area", [_pos, _size select 0, _size select 1, true,-1]];

// Create activation trigger
_trigger = createTrigger ["EmptyDetector", _pos];
_trigger setTriggerArea (_area select [1,99]);
_trigger setTriggerActivation ["ANYPLAYER", "PRESENT", true];
_trigger setTriggerStatements ["this", "hint 'CIV trigger on'", "hint 'CIV trigger off'"];
_civ_module synchronizeObjectsAdd [_trigger];

// Activate presence module
_civ_module setVariable ["bis_fnc_initmodules_activate", true, true];

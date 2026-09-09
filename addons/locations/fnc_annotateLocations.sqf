/*
	Enriches detected locations (from DynOps_fnc_detectLocations) in place
	with a display name/type and isAirport/isMilitary/hasHospital flags.

	Naming is hybrid: a detection adopts the name/type of a real map-config
	location (NameCityCapital/NameCity/NameVillage/NameLocal) if that
	location's position falls inside the detection's OBB, otherwise a name is
	synthesised from the detection's class and BIS_fnc_locationDescription.

	Params:
		0: ARRAY of HashMap - detected locations, as returned by
			DynOps_fnc_detectLocations

	Returns: ARRAY of HashMap - the same locations, annotated in place with
		"name" (String), "type" (String), "isAirport" (Boolean),
		"isMilitary" (Boolean), "hasHospital" (Boolean)
*/
#include "script_component.hpp"

params ["_locations"];

// Index-aligned with the Rust `LocationClass` enum (src/locations.rs).
private _classNames = ["Farm", "Hamlet", "Village", "Town", "City"];
private _classTypes = ["NameLocal", "NameLocal", "NameVillage", "NameCity", "NameCityCapital"];
private _militaryKeywords = ["military"];

private _airports = call DynOps_fnc_getAirports;

private _isMilitaryText = {
	params ["_text"];
	private _lower = toLower _text;
	private _found = false;
	{ if (_x in _lower) exitWith { _found = true }; } forEach _militaryKeywords;
	_found
};

{
	private _location = _x;
	private _pos = _location get "pos";
	private _a = _location get "a";
	private _b = _location get "b";
	private _dir = _location get "dir";
	private _area = [_pos, _a, _b, _dir, true];
	private _searchRadius = sqrt ((_a * _a) + (_b * _b)) + 50;

	// Name/type
	private _named = nearestLocations [_pos, ["NameCityCapital", "NameCity", "NameVillage", "NameLocal"], _searchRadius];
	_named = _named select { (locationPosition _x) inArea _area };
	if (_named isNotEqualTo []) then {
		private _best = _named # 0;
		_location set ["name", text _best];
		_location set ["type", type _best];
	} else {
		private _type = _classTypes select (_location get "class");
		private _tempLoc = createLocation [_type, _pos, 0, 0];
		private _description = _tempLoc call BIS_fnc_locationDescription;
		deleteLocation _tempLoc;
		_location set ["name", format ["%1 %2", _classNames select (_location get "class"), _description]];
		_location set ["type", _type];
	};

	// Airport: does any known airport zone fall inside this detection?
	_location set ["isAirport", (_airports findIf { (_x # 0) inArea _area }) != -1];

	// Military: LocationBase_F objects, or nearby Airport/NameLocal
	// locations whose text mentions a military keyword, inside this
	// detection's OBB.
	private _militaryBases = _pos nearObjects ["LocationBase_F", _searchRadius];
	_militaryBases = _militaryBases select { (getPos _x) inArea _area };
	private _militaryNamed = nearestLocations [_pos, ["Airport", "NameLocal"], _searchRadius];
	_militaryNamed = _militaryNamed select {
		((locationPosition _x) inArea _area) && ([text _x] call _isMilitaryText)
	};
	_location set ["isMilitary", (_militaryBases isNotEqualTo []) || (_militaryNamed isNotEqualTo [])];

	// Hospital
	private _hospitals = nearestTerrainObjects [_pos, ["HOSPITAL"], _searchRadius, false, true];
	_location set ["hasHospital", ({ (getPos _x) inArea _area } count _hospitals) > 0];
} forEach _locations;

_locations;

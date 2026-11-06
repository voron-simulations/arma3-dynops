/*
	Assigns an owning side to every location, in place. Population spends a
	faction's budget only over locations its side owns, and ownership is a
	side attribute rather than a faction one so a mission maker can say
	"large NATO force plus some CTRG" without deciding which villages CTRG
	holds (see adr/0002-population.md).

	Every location starts with the mission variable
	dynops_locations_defaultOwner (default "EAST"). Editor markers named
	dynops_owner_<SIDE> with any suffix then override it: every location
	whose centre lies inside the marker area is reassigned. <SIDE> is
	WEST, EAST, GUER or none (unowned), case-insensitive, read up to the next
	underscore, so dynops_owner_WEST_1 and dynops_owner_none both parse.
	Markers are applied in name order, so on overlap the later name wins.
	Invalid markers and an invalid default are logged and ignored rather than
	aborting, so one typo in the editor cannot disable population.

	Params:
		0: ARRAY of HashMap - annotated locations, as returned by
			DynOps_fnc_annotateLocations

	Returns: ARRAY of HashMap - the same locations, with "owner" (String:
		"WEST", "EAST", "GUER", or "" for unowned) set on each
*/
#include "script_component.hpp"

params [["_locations", [], [[]]]];

private _prefix = "dynops_owner_";

// Side names as written by mission makers -> the stored "owner" value. "none"
// maps to "" so an unowned location is falsy-comparable to any side string.
private _parseOwner = {
	params ["_text"];
	private _name = toUpper _text;
	if (_name == "NONE") exitWith { "" };
	if (_name in ["WEST", "EAST", "GUER"]) exitWith { _name };
	nil
};

private _default = missionNamespace getVariable [QGVAR(defaultOwner), "EAST"];
private _defaultOwner = if (_default isEqualType "") then { [_default] call _parseOwner } else { nil };
if (isNil "_defaultOwner") then {
	WARNING_1("Invalid %1, falling back to EAST",QGVAR(defaultOwner));
	_defaultOwner = "EAST";
};

{ _x set ["owner", _defaultOwner]; } forEach _locations;

// find on a lower-cased copy because find itself is case-sensitive and the
// convention is documented as case-insensitive.
private _markers = allMapMarkers select { (toLower _x) find _prefix == 0 };
_markers sort true;

private _applied = 0;
{
	private _marker = _x;
	private _suffix = _marker select [count _prefix, (count _marker) - (count _prefix)];
	private _owner = [(_suffix splitString "_") param [0, ""]] call _parseOwner;

	if (isNil "_owner") then {
		WARNING_2("Ignoring marker %1: '%2' is not one of WEST, EAST, GUER, none",_marker,_suffix);
	} else {
		{
			if ((_x get "pos") inArea _marker) then { _x set ["owner", _owner]; };
		} forEach _locations;
		_applied = _applied + 1;
	};
} forEach _markers;

INFO_3("Assigned owners: default %1, %2 of %3 owner markers applied",_defaultOwner,_applied,count _markers);
_locations;

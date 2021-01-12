#include "script_component.hpp"

params ["_hash", "_key", "_value"];

private _list = [_hash, _key] call CBA_fnc_hashGet;

if (typeName _value == "ARRAY") then {
	_list append _value;
	_list = _list arrayIntersect _list;
} else {
	_list pushBackUnique _value;
};

[_hash, _key, _list] call CBA_fnc_hashSet;

#include "script_component.hpp"

params ["_hash", "_key", "_value"];
private _list = _hash getOrDefault [_key, []];

if (typeName _value == "ARRAY") then {
	_list append _value;
	_list = _list arrayIntersect _list;
} else {
	_list pushBackUnique _value;
};

_hash set [_key, _list];

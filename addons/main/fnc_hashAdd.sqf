#include "script_component.hpp"

params ["_hashMap", "_key", "_value"];
private _list = _hashMap getOrDefault [_key, []];

if (typeName _value == "ARRAY") then {
	_list append _value;
	_list = _list arrayIntersect _list;
} else {
	_list pushBackUnique _value;
};

_hashMap set [_key, _list];

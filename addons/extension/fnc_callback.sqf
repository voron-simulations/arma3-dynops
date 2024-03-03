#include "script_component.hpp"

params ["_name", "_component", "_data"];

if ((tolower _name) != "dynops") exitWith { };

TRACE_2("Callback %1 called with args %2",_component,_data);

if (_component == "hint") exitWith { hint _data; };
if (_component == "systemChat") exitWith { _data remoteExec ["systemChat", 0]; };

_func = missionNamespace getVariable _component;
if (isNil "_func") exitWith { 
	private _error = format ["Cannot invoke function %1, nil", _component];
	_error call BIS_fnc_error;
	ERROR(_error);
};
_data spawn _func;

params ["_name", "_component", "_data"];
// hint format["Callback called with data %1", _this];

if ((tolower _name) != "dynops") exitWith { };
if (_component == "hint") exitWith { hint _data; };
if (_component == "systemChat") exitWith { _data remoteExec ["systemChat", 0]; };

_func = missionNamespace getVariable _component;
if (isNil "_func") exitWith { ["Cannot invoke function %1, nil", _component] call BIS_fnc_error; };
_data call _func;

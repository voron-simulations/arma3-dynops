#include "script_component.hpp"

params [
	["_logic", objNull, [objNull]],		// module logic
	["_units", [], [[]]],				// list of affected units
	["_activated", true, [true]]		// was module activated?
];

if (!isServer) exitWith { };

private _template = 
"The fictional state of Livonia is in state of armed conflict between rebel Freedom and Independence Army (FIA) 
faction and government Livonia Defence Force (LDF).
I want you to act as %1, %2 year old %3.

You know following facts:
%4.

Do not break character. Use vocabulary and vernacular which would be expected from person of such background.
Give short answers only";


{
	if (!isNull (_x getVariable QGVAR(agent_uid))) then { continue };

	private _name = name _x;
	private _age = _logic getVariable "age";
	private _occupation = _logic getVariable "occupation";
	private _facts = _logic getVariable "facts";
	private _uuid = call EFUNC(extension,uuid);

	_x setVariable [QGVAR(agent_uid), _uuid];
	GVAR(agents) set [_uuid, _x];

	private _input = format [_template, _name, _age, _occupation, _facts];
	["chat:init", [_uuid, _input]] call EFUNC(extension,call);
} forEach _units;

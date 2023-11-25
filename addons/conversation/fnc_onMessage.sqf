#include "script_component.hpp"

params ["_sender", "_text"];
private _peopleInRange = (position _sender) nearObjects ["Man", 3];

INFO_2("Chat message from %1: %2", name _sender, _text);

{
	private _agent_uid = _x getVariable QGVAR(agent_uid);
	if (!isNil "_agent_uid") exitWith
	{
		private _formatted_message = format ["%1: %2", name _sender, _text];
		["chat:message", [_agent_uid, _formatted_message]] call EFUNC(extension,call);
	};
	// systemChat format ["%1 heard you but doesn't want to talk", name _x];
} forEach _peopleInRange;

// _civilians = _civilians select { _x getVariable ["dynops_chat", 0] == 1; }; // Filter down to civilians with talk enabled
false;

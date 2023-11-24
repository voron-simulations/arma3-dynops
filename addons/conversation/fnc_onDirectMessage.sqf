#include "script_component.hpp"

params ["_channel", "_owner", "_from", "_text", "_person", "_name", "_strID", "_forcedDisplay", "_isPlayerMessage", "_sentenceType", "_chatMessageType"];
if (_channel != 5) exitWith { false; };
private _peopleInRange = (position _person) nearObjects ["Man", 3];

{
	private _agent_uid = _x getVariable QGVAR(agent_uid);
	if (!isNil "_agent_uid") exitWith
	{
		private _formatted_message = format ["%1: %2", name _person, _text];
		["chat:message", [_agent_uid, _formatted_message]] call EFUNC(extension,call);
	};
	// systemChat format ["%1 heard you but doesn't want to talk", name _x];
} forEach _peopleInRange;

// _civilians = _civilians select { _x getVariable ["dynops_chat", 0] == 1; }; // Filter down to civilians with talk enabled
false;

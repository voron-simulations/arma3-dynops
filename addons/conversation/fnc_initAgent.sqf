#include "script_component.hpp"

params ["_agent"];

if (is3DEN || !isServer || isPlayer _agent) exitWith { };
if (_agent getVariable ["DynOps_ConversationEnabled", false] != true) exitWith { };

private _name = name _agent;
private _occupation = _agent getVariable ["DynOps_ConversationOccupation", "Civilian"];
private _facts = [];

// Facts set for this person in editor
_facts pushBack (_agent getVariable "DynOps_ConversationFacts");

// Facts about location
_facts pushBack (format ["You are currently located %1, %2", _agent call BIS_fnc_locationDescription, worldName]);

// Facts about the team
if (side _agent != civilian) then
{
	if (_agent != leader _agent) then {
		_facts pushBack (format ["Your commander is %1", name leader _agent]);
	};
};

private _template = 
"I want you to act as %1, %2.

You know following facts:
%3.

Do not break character. Use vocabulary and vernacular which would be expected from person of such background.
Give short answers only";

private _current_uid = _agent getVariable QGVAR(agent_uid);
if (!isNil "_current_uid") exitWith { };

private _uuid = call EFUNC(extension,uuid);
_agent setVariable [QGVAR(agent_uid), _uuid];
GVAR(agents) set [_uuid, _agent];

private _input = format [_template, _name, _occupation, _facts joinString endl];
["chat:init", [_uuid, _input]] call EFUNC(extension,call);
INFO_3("Agent initialized %1 (%2) %3", _name, _occupation, _uuid);

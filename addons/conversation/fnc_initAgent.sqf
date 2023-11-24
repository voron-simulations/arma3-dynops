#include "script_component.hpp"

params ["_agent"];

if (!isServer) exitWith { };
if (isPlayer _agent) exitWith { };
if (_agent getVariable ["DynOps_ConversationEnabled", false] != true) exitWith { };

private _name = name _agent;
private _occupation = _agent getVariable ["DynOps_ConversationOccupation", "Civilian"];
private _facts = _agent getVariable ["DynOps_ConversationFacts", "Arma 4 will be released soon"];

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

private _input = format [_template, _name, _occupation, _facts];
["chat:init", [_uuid, _input]] call EFUNC(extension,call);
INFO_3("Conversation agent initialized name=%1 occupation=%2 uuid=%3", _name, _occupation, _uuid);

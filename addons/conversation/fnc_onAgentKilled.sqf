#include "script_component.hpp"

params ["_agent"];

private _uid = _agent getVariable QGVAR(agent_uid);

if (isNil "_agent_uid") exitWith { };

GVAR(agents) deleteAt _uid;
["chat:destroy", [_agent_uid]] call DynOps_fnc_call;
INFO_3("Agent %1 (%2) killed", _name, _uuid);

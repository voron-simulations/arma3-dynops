#include "script_component.hpp"

addMissionEventHandler ["HandleChatMessage", FUNC(onDirectMessage)];

GVAR(agents) = createHashMap;

if (hasInterface) then {
	[
		"cba_events_chatMessageSent", 
		{
			[player, _this # 0] call FUNC(onMessage); 
		}
	] call CBA_fnc_addEventHandler;
};

if (isMultiplayer && isServer) then {
	[
		"cba_events_chatMessageSent", 
		{ 
			[player, _this # 0] remoteExec [QFUNC(onMessage), 2, false]; 
		}
	] remoteExec ["CBA_fnc_addEventHandler", 2, true];
};

["Man", "init", { (_this select 0) spawn FUNC(initAgent); }] call CBA_fnc_addClassEventHandler;

#include "script_component.hpp"

GVAR(agents) = createHashMap;

// Global config
GVAR(ConversationRange) = 4;

if (isMultiplayer && isServer) then {
	[
		"cba_events_chatMessageSent", 
		{
			if (currentChannel == 5) then {
				[player, _this # 0] remoteExec ["DynOps_fnc_onMessage", 2, false];
			}
		}
	] remoteExec ["CBA_fnc_addEventHandler", 0, true];
};

["CAManBase", "Init", { (_this select 0) spawn DynOps_fnc_initAgent; }] call CBA_fnc_addClassEventHandler;
["CAManBase", "Killed", { (_this select 0) spawn DynOps_fnc_onAgentKilled; }] call CBA_fnc_addClassEventHandler;

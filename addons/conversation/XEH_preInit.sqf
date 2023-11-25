#include "script_component.hpp"

GVAR(agents) = createHashMap;

// if (hasInterface) then {
// 	[
// 		"cba_events_chatMessageSent", 
// 		{
// 			[player, _this # 0] call FUNC(onMessage); 
// 		}
// 	] call CBA_fnc_addEventHandler;
// };

if (isMultiplayer && isServer) then {
	[
		"cba_events_chatMessageSent", 
		{
			if (currentChannel == 5) then {
				[player, _this # 0] remoteExec [QFUNC(onMessage), 2, false];
			}
		}
	] remoteExec ["CBA_fnc_addEventHandler", 0, true];
};

["Man", "init", { (_this select 0) spawn FUNC(initAgent); }] call CBA_fnc_addClassEventHandler;

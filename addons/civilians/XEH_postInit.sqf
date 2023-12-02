#include "script_component.hpp"

GVAR(Casualties) = []; 

if (hasInterface && isMultiplayer) then
{
	QGVAR(Casualties) addPublicVariableEventHandler DynOps_fnc_onCivilianCasualtyListUpdated;
};

["Civilian", "Killed", DynOps_fnc_onCivilianKilled] call CBA_fnc_addClassEventHandler;
INFO("PostInit finished");

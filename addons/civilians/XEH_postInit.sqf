#include "script_component.hpp"

GVAR(Casualties) = []; 

if (hasInterface && isMultiplayer) then
{
	QGVAR(Casualties) addPublicVariableEventHandler FUNC(onCivilianCasualtyListUpdated);
};

["Civilian", "Killed", FUNC(onCivilianKilled)] call CBA_fnc_addClassEventHandler;
INFO("PostInit finished");

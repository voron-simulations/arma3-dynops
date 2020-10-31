#include "script_component.hpp"

["Civilian", "Killed", FUNC(onCivilianKilled)] call CBA_fnc_addClassEventHandler;

if (hasInterface && isMultiplayer) then
{
	QGVAR(Casualties) addPublicVariableEventHandler FUNC(onCivilianCasualtyListUpdated);
};

/*
	Event handler for "Killed" CBA extended event
*/
#include "script_component.hpp"
params ["_unit", "_killer", "_instigator"];

with missionNamespace do 
{
	GVAR(Casualties) pushBack (name _unit);
	INFO_2("Civilian casualty: %1 killed by %2", name _unit, name _instigator);
	publicVariable QGVAR(Casualties);
	if (hasInterface) then {
		[QGVAR(Casualties), GVAR(Casualties)] spawn FUNC(onCivilianCasualtyListUpdated);
	};
}

/*
	Event handler for "Killed" CBA extended event
*/
#include "script_component.hpp"
params ["_unit", "_killer", "_instigator"];

// DEBUG
hint format ["%1 was killed by %2", name _unit, name _instigator];

with missionNamespace do 
{
	if (isNil QGVAR(Casualties)) then { 
		GVAR(Casualties) = []; 
	};
	GVAR(Casualties) pushBack (name _unit);
	publicVariable QGVAR(Casualties);
	if (hasInterface) then {
		[QGVAR(Casualties), GVAR(Casualties)] spawn FUNC(onCivilianCasualtyListUpdated);
	};
}

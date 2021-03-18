/*
	Event handler for "Killed" CBA extended event
*/
#include "script_component.hpp"
params ["_unit", "_killer", "_instigator"];

with missionNamespace do 
{
	// UAV/UGV player operated road kill
	if (isNull _instigator) then { _instigator = UAVControl vehicle _killer select 0 };
	// player driven vehicle road kill
	if (isNull _instigator) then { _instigator = _killer };

	GVAR(Casualties) pushBack (name _unit);

	INFO_2("Civilian casualty: %1 killed by %2", name _unit, name _instigator);
	publicVariable QGVAR(Casualties);
	if (hasInterface) then {
		[QGVAR(Casualties), GVAR(Casualties)] spawn FUNC(onCivilianCasualtyListUpdated);
	};
}

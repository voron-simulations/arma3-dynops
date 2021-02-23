#include "script_component.hpp"

params ["_side", "_addedPoints"];

if (!isMultiplayer || isServer) then
{
	// If SP or MP server, perform the addition and publish result
	if (isNil QGVAR(SideIntel)) then {
		GVAR(SideIntel) = [[], 0] call CBA_fnc_hashCreate;
	};
	private _points = [GVAR(SideIntel), _side] call CBA_fnc_hashGet;
	_points = _points + _addedPoints;
	[GVAR(SideIntel), _side, _points] call CBA_fnc_hashSet;
	publicVariable QGVAR(SideIntel);
}
else 
{
	// Defer execution to server
	_this remoteExec [FUNC(addIntelPoints), 2, false];
};

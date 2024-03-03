#include "script_component.hpp"

params ["_side", "_addedPoints"];

// If SP or MP server, perform the addition and publish result
if (!isMultiplayer || isServer) then
{
	if (isNil QGVAR(SideIntel)) then {
		GVAR(SideIntel) = createHashMap;
	};
	private _points = GVAR(SideIntel) get _side;
	_points = if (isNil "_points") then { _addedPoints; } else { _points + _addedPoints; };
	GVAR(SideIntel) set [_side, _points];
	publicVariable QGVAR(SideIntel);
}
else 
{
	// Otherwise, defer execution to server
	_this remoteExec [DynOps_fnc_addIntelPoints, 2, false];
};

#include "script_component.hpp"

params ["_side", "_addedPoints"];

if (!isMultiplayer || isServer) then
{
	// If SP or MP server, perform the addition and publish result
	if (isNil QGVAR(SideIntel)) then {
		GVAR(SideIntel) = createHashMap;
	};
	private _points = GVAR(SideIntel) get _side;
	_points = _points + _addedPoints;
	GVAR(SideIntel) set [_side, _points];
	publicVariable QGVAR(SideIntel);
}
else 
{
	// Defer execution to server
	_this remoteExec [FUNC(addIntelPoints), 2, false];
};

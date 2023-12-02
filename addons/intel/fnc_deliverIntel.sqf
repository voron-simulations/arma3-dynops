/*
	Calculate the amount of intel player has and "cash it in"
	To be called in trigger activation when player returns to base
*/
#include "script_component.hpp"

private _intelItemsHashSet = GVAR(intelItemCosts);

private _magazines = magazines player;
private _side = side player;
private _points = 0;

// Cycle through "magazines" player has 
// and determine if any of them are intel items
// if it's an intel item, record points value and remove from inventory
{
	if (_x in _intelItemsHashSet) then {
		_points = _points + (_intelItemsHashSet get _x);
		player removeMagazineGlobal _x;
	}
} forEach _magazines;

// Show notification, add rating, increase side's counter
if (_points > 0) then {
	[_side, _points] call DynOps_fnc_addIntelPoints;
	player addRating _points;
	["IntelDelivered", [_points]] spawn BIS_fnc_showNotification;
};

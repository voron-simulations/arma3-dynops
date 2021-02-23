/*
	Calculate the amount of intel player has and "cash it in"
	To be called in trigger activation when player returns to base
*/
#include "script_component.hpp"

private _intelItemCosts = 
[
	["Money", 5],
	["Money_roll", 20],
	["Money_stack", 50],
	["NetworkStructure", 100],
	["DocumentsSecret", 500],
	["FileTopSecret", 2000],
	["FilesSecret", 500],
	["Files", 100],
	["FlashDisk", 200],
	["Keys", 50],
	["Laptop_Closed", 300],
	["Laptop_Unfolded", 300],
	["SmartPhone", 250],
	["MobilePhone", 100],
	["SatPhone", 100],
	["Wallet_ID", 50]
];

private _intelItemsHashSet = [_intelItemCosts, 0] call CBA_fnc_hashCreate;

private _magazines = magazines player;
private _side = side player;
private _keys = [_intelItemsHashSet] call CBA_fnc_hashKeys;
private _points = 0;

// Cycle through "magazines" player has 
// and determine if any of them are intel items
// if it's an intel item, record points value and remove from inventory
{
	if ([_intelItemsHashSet, _x] call CBA_fnc_hashHasKey) then {
		_points = _points + ([_intelItemsHashSet, _x] call CBA_fnc_hashGet);
		player removeMagazineGlobal _x;
	}
} forEach _magazines;

// Show notification, add rating, increase side's counter
if (_points > 0) then {
	[_side, _points] call FUNC(addIntelPoints);
	player addRating _points;
	["IntelDelivered", [_points]] spawn BIS_fnc_showNotification;
};

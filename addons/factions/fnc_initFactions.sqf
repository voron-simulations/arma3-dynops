#include "script_component.hpp"

private _factions = _this;

if (isNil QGVARMAIN(FactionData)) then {
	GVARMAIN(FactionData) = [[], []] call CBA_fnc_hashCreate;
};

{
	private _factionData = _x call EFUNC(factions,getFactionData);
	[GVARMAIN(FactionData), _x, _factionData] call CBA_fnc_hashSet;
} forEach _factions;


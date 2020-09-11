private _factions = _this;

if (isNil "FactionData") then {
	FactionData = [[], []] call CBA_fnc_hashCreate;
};

{
	private _factionData = _x call DW_fnc_getFactionData;
	[FactionData, _x, _factionData] call CBA_fnc_hashSet;
} forEach _factions;
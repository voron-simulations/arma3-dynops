params ["_faction"];

private _factionData = [[],[]] call CBA_fnc_hashCreate;
private _sideId = _faction call DW_fnc_getFactionSideId;
private _side = _sideId call BIS_fnc_sideType;

// Populate groups data
{
	[_factionData, "Group_" + _x, [_faction, _x] call DW_fnc_getFactionGroups] call CBA_fnc_hashSet;
} forEach ["Infantry", "SpecOps", "Mechanized", "Motorized", "Armored"];

// Populate units data
{
	private _types = (configName _x) call BIS_fnc_objectType;
	private _key = _types # 0 + "_" + _types # 1; // See https://community.bistudio.com/wiki/BIS_fnc_objectType
	private _list = [_factionData, _key] call CBA_fnc_hashGet;
	_list pushBackUnique _x;
	[_factionData, _key, _list] call CBA_fnc_hashSet;
} forEach (_faction call DW_fnc_getFactionUnits);

[_factionData, "Side", _side] call CBA_fnc_hashSet;
[_factionData, "SideId", _sideId] call CBA_fnc_hashSet;
_factionData;
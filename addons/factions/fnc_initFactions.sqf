#include "script_component.hpp"

/****** STAGE 0: Preparation ******/
// Get all faction configs
private _factions = "true" configClasses (configFile >> "CfgFactionClasses");

// Filter down to factions belonging to BLU/OPF/IND/CIV
_factions = _factions select {
    private _side = getNumber ( _x >> "side");
    _side >= 0 && _side <= 3;
};

private _factionNames = _factions apply { configName _x; };

// Create the root hashset
if (isNil QGVARMAIN(FactionData)) then {
	GVARMAIN(FactionData) = [[], []] call CBA_fnc_hashCreate;
};

// Create empty hashset for every faction
{
	private _emptySet = [[],[]] call CBA_fnc_hashCreate;
	[GVARMAIN(FactionData), _x, _emptySet] call CBA_fnc_hashSet;
} forEach _factionNames;

/****** STAGE 1: Units data ******/

// Enumerate all units in CfgVehicles
private _cfgVehicles = "true" configClasses (configFile >> "CfgVehicles");
// Filter down to CfgVehicles belonging to 'interesting' factions
_cfgVehicles = _cfgVehicles select { getText (_x >> "faction") in _factionNames && getNumber (_x >> "scope") == 2 };
// Put in units data for each faction
{
	private _faction = getText (_x >> "faction");
	private _tags = (configName _x) call BIS_fnc_objectType;
	private _key = (_tags # 0) + "_" + (_tags # 1); // See https://community.bistudio.com/wiki/BIS_fnc_objectType
	private _factionData = [GVARMAIN(FactionData), _faction] call CBA_fnc_hashGet;
	private _list = [_factionData, _key] call CBA_fnc_hashGet;
	_list pushBackUnique configName _x;
	[_factionData, _key, _list] call CBA_fnc_hashSet;

	// Fill weapons/items/magazines while we're at it
	if ((configName _x) isKindOf "Man") then {
		private _magazines = [_factionData, "Magazines"] call CBA_fnc_hashGet;
		private _weapons = [_factionData, "Weapons"] call CBA_fnc_hashGet;
		private _items = [_factionData, "Items"] call CBA_fnc_hashGet;

		_magazines append getArray (_x >> "magazines");
		_weapons append getArray (_x >> "weapons");
		_items append getArray (_x >> "linkedItems");

		[_factionData, "Magazines", _magazines] call CBA_fnc_hashSet;
		[_factionData, "Weapons", _weapons] call CBA_fnc_hashSet;
		[_factionData, "Items", _items] call CBA_fnc_hashSet;
	};
} forEach _cfgVehicles;

/****** STAGE 2: Groups data ******/
// Enumerate "group side" classes
private _groupSides = "true" configClasses (configFile >> "CfgGroups");
// Enumerate "group faction" classes
private _groupFactions = [];
{
	_groupFactions append ("true" configClasses (_x));
} forEach _groupSides;
// Filter down to "interesting" factions
_groupFactions = _groupFactions select { configName _x in _factionNames; };
// Faction loop
{
	private _faction = configName (_x);
	private _factionData = [GVARMAIN(FactionData), _faction] call CBA_fnc_hashGet;
	// Group type loop
	{
		private _groupType = configName (_x);
		// Group loop
		{
			private _groupName = configName (_x);
			private _key = "Group_" + _groupType;
			private _list = [_factionData, _key] call CBA_fnc_hashGet;
			_list pushBackUnique _groupName;
			[_factionData, _key, _list] call CBA_fnc_hashSet;
		} forEach ("true" configClasses (_x));
	} forEach ("true" configClasses (_x));
} forEach _groupFactions;


/****** STAGE 5: Cleanup ******/
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
INFO_1("Detected factions: %1", _factionNames);

// Create the root hashset
if (isNil QGVARMAIN(FactionData)) then {
	GVARMAIN(FactionData) = [[], []] call CBA_fnc_hashCreate;
};

// Create base hashset for every faction
{
	private _baseSet = [[],[]] call CBA_fnc_hashCreate;
	// Pre-fill items which are available for any faction
	[_baseSet, "Items", ["ItemGPS","FirstAidKit","ItemWatch","ItemCompass","ItemRadio","ItemMap","MineDetector","Medikit","ToolKit"]] call EFUNC(main,hashAdd);

	[GVARMAIN(FactionData), _x, _baseSet] call CBA_fnc_hashSet;	
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
		// Get person's equipment from config
		private _backpack = getText (_x >> "backpack");
		private _items = getArray (_x >> "linkeditems");
		private _magazines = getArray (_x >> "magazines");
		private _uniform = getText (_x >> "uniformClass");
		private _weapons = getArray (_x >> "weapons");

		// Backpacks/weapons on units are customized - find 'empty' base class for them
		_backpack = [_backpack, "CfgVehicles"] call CBA_fnc_getNonPresetClass;
		_weapons = _weapons apply { [_x] call CBA_fnc_getNonPresetClass; };

		[_factionData, "Backpacks", _backpack] call EFUNC(main,hashAdd);
        [_factionData, "Items", _items] call EFUNC(main,hashAdd);
        [_factionData, "Magazines", _magazines] call EFUNC(main,hashAdd);
        [_factionData, "Uniforms", _uniform] call EFUNC(main,hashAdd);
        [_factionData, "Weapons", _weapons] call EFUNC(main,hashAdd);
	};
} forEach _cfgVehicles;

INFO_1("Processed %1 CfgVehicle configs", count _cfgVehicles);

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

/****** STAGE 3: Generic data ******/
{
	private _sideId = getNumber (_x >> "side");
	private _side = _sideId call BIS_fnc_sideType;
	private _displayName = getText (_x >> "displayName");

	private _factionData = [GVARMAIN(FactionData), configName _x] call CBA_fnc_hashGet;
	[_factionData, "Side", _side] call CBA_fnc_hashSet;
	[_factionData, "SideId", _sideId] call CBA_fnc_hashSet;
	[_factionData, "DisplayName", _displayName] call CBA_fnc_hashSet;
} forEach _factions;

/****** STAGE 4: Canned data ******/
call FUNC(factionStaticData);

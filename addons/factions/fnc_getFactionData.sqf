#include "script_component.hpp"

params ["_faction"];

private _factionData = [[],[]] call CBA_fnc_hashCreate;
private _sideId = _faction call EFUNC(factions,getFactionSideId);
private _side = _sideId call BIS_fnc_sideType;

// Populate groups
{
	[_factionData, "Group_" + _x, [_faction, _x] call EFUNC(factions,getFactionGroups)] call CBA_fnc_hashSet;
} forEach ["Infantry", "SpecOps", "Mechanized", "Motorized", "Armored"];

private _factionUnits = _faction call EFUNC(factions,getFactionUnits);

// Populate units/objects
{
	private _types = (configName _x) call BIS_fnc_objectType;
	private _key = _types # 0 + "_" + _types # 1; // See https://community.bistudio.com/wiki/BIS_fnc_objectType
	private _list = [_factionData, _key] call CBA_fnc_hashGet;
	_list pushBackUnique _x;
	[_factionData, _key, _list] call CBA_fnc_hashSet;
} forEach _factionUnits;

private _weapons = [];
private _magazines = [];
private _items = [];

// Populate personnel inventory items
{
	_magazines append getArray (_x >> "magazines");
	_weapons append getArray (_x >> "weapons");
	_items append getArray (_x >> "linkedItems");
} forEach (_factionUnits select { (configName _x) isKindOf "Man"; } );

_weapons = _weapons arrayIntersect _weapons;
_magazines = _magazines arrayIntersect _magazines;
_items = _items arrayIntersect _items;

_weapons = _weapons - ["Throw", "Put"];

_weapons sort true;
_magazines sort true;
_items sort true;

[_factionData, "Weapons", _weapons] call CBA_fnc_hashSet;
[_factionData, "Magazines", _magazines] call CBA_fnc_hashSet;
[_factionData, "Items", _items] call CBA_fnc_hashSet;

[_factionData, "Side", _side] call CBA_fnc_hashSet;
[_factionData, "SideId", _sideId] call CBA_fnc_hashSet;
_factionData;

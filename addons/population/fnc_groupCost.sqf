/*
	Budget cost of one CfgGroups group: the sum of its member units' costs.

	A unit's cost comes from the project-owned CfgDynOpsCosts table, keyed by
	the same "<type>_<subtype>" BIS_fnc_objectType category FactionData uses,
	rather than from CfgVehicles' own "cost" field, which is tuned for AI
	threat evaluation and varies wildly between mods. Categories missing from
	the table fall back to CfgDynOpsCosts >> "default".

	A unit's cost is memoised per class in GVAR(UnitCostCache): every faction
	shares most unit classes between its groups, and BIS_fnc_objectType is
	comparatively slow.

	Params:
		0: CONFIG - a CfgGroups group, e.g.
			configFile >> "CfgGroups" >> "West" >> "BLU_F" >> "Infantry" >> "BUS_InfSquad"

	Returns: NUMBER - integer cost, at least 1 (the extension rejects
		zero-cost groups because they could be assigned without limit)
*/
#include "script_component.hpp"

params [["_groupConfig", configNull, [configNull]]];

if (isNil QGVAR(UnitCostCache)) then { GVAR(UnitCostCache) = createHashMap; };

private _costs = configFile >> "CfgDynOpsCosts";
private _defaultCost = getNumber (_costs >> "default");

private _total = 0;
{
	private _unitClass = getText (_x >> "vehicle");
	private _cost = GVAR(UnitCostCache) getOrDefault [_unitClass, -1];

	if (_cost < 0) then {
		private _tags = _unitClass call BIS_fnc_objectType;
		private _entry = _costs >> ((_tags # 0) + "_" + (_tags # 1));
		// isNumber rather than getNumber alone: a missing entry reads as 0,
		// which would make unknown categories free instead of default-priced.
		_cost = if (isNumber _entry) then { getNumber _entry } else { _defaultCost };
		GVAR(UnitCostCache) set [_unitClass, _cost];
	};

	_total = _total + _cost;
} forEach (("true" configClasses _groupConfig) select { getText (_x >> "vehicle") != "" });

1 max (round _total);

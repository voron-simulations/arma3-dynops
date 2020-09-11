params ["_faction", "_type"];

// Find the needed CfgGroups node
private _sideId = _faction call DW_fnc_getFactionSideId;
if (_sideId < 0 || _sideId > 3) exitWith { []; };
private _configs = "true" configClasses (configFile >> "CfgGroups") select { getNumber (_x >> "side") == _sideId; };
if (count _configs <= 0) exitWith { []; };

_configs = "true" configClasses ((_configs # 0) >> _faction >> _type);
_configs;
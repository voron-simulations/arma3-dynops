params ["_faction"];

private _classes = "true" configClasses (configFile >> "CfgVehicles");
_classes = _classes select { getText (_x >> "faction") == _faction && getNumber (_x >> "scope") == 2 };
_classes;

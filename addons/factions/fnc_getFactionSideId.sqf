params ["_faction"];

private _configs = "true" configClasses (configFile >> "CfgFactionClasses") select { configName _x == _faction };
if (count _configs != 1) exitWith { 4; }; // sideUnknown
getNumber ((_configs # 0) >> "side");

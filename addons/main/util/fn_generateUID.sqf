params ["_prefix"];

if (isNil "_prefix") then { _prefix = "uid"; }

with missionNamespace do {
	if (isNil "GlobalUIDCounter") then { GlobalUIDCounter = 0; };
	GlobalUIDCounter = GlobalUIDCounter + 1;
	publicVariable "GlobalUIDCounter";
};

_prefix + "_" + str GlobalUIDCounter;
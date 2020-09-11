private _activationDistance = (dynamicSimulationDistance "Group") / 4;

fnc_entityExcludedFromDynSim = {
	params ["_unit"];
	isPlayer _unit || _unit isKindOf "Air" || unitIsUAV _unit;
};

fnc_shouldBeManagedDynamically = {
	params ["_group"];
	
	private _units = units _group;
	private _vehicles = _group call DW_fnc_getGroupVehicles;

	if (_group getVariable ["DynSimExclude", false]) exitWith { false; };
	if ( { _x call fnc_entityExcludedFromDynSim; } count (_units + _vehicles) > 0 ) exitWith { false; };

	(_x call DW_fnc_getCurrentWaypointRange) < _activationDistance;
};

// Enable dynamic simulation for all groups at once to ease startup
{ _x enableDynamicSimulation true; } forEach allGroups;

while { true; } do {
	{
		_x enableDynamicSimulation (_x call fnc_shouldBeManagedDynamically); // [_x, _enable] remoteExecCall ["enableDynamicSimulation", 2];
		_x deleteGroupWhenEmpty true;
		sleep 0.1;
	} forEach allGroups;
};
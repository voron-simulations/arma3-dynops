// Generic military location population script
params ["_location", "_faction"];

private _factionData = [FactionData, _faction] call CBA_fnc_hashGet;
private _side = [_factionData, "Side"] call CBA_fnc_hashGet;
private _groups = [_factionData, "Group_Infantry"] call CBA_fnc_hashGet;

private _radius = (selectMax size _location) / 2;
private _buildingPositions = [];
private _houses = (locationPosition _location) nearObjects ["House", _radius] inAreaArray _location;
{ _buildingPositions append (_x buildingPos -1); } forEach _houses;

for "_i" from (count _buildingPositions) to 10 step -30 do {
	private _garrison = [locationPosition _x, _side, selectRandom _groups] call BIS_fnc_spawnGroup;
	[_garrison, locationPosition _x, _radius, [], true, false, -1] call lambs_wp_fnc_taskGarrison;
};

for "_i" from random 3 to 0 step -1 do {
	private _patrol = [locationPosition _x, _side, selectRandom _groups] call BIS_fnc_spawnGroup;
	[_patrol, locationPosition _x, 500] call lambs_wp_fnc_taskPatrol;
};

private _helipads = [_location, "HeliH"] call DW_fnc_getObjectsInLocation;
{ _x call DW_fnc_placeHelipadClutter; } forEach _helipads;

if (count _helipads > 0 && random 10 < 5) then {
	private _helicopters = [_factionData, "Vehicle_Helicopter"] call CBA_fnc_hashGet;
	private _pad = selectRandom _helipads;
	private _vehicle = createVehicle [configName (selectRandom _helicopters), getPos _pad, [], 0];
	private _group = createVehicleCrew _vehicle;
	_group enableDynamicSimulation true;
	_group deleteGroupWhenEmpty true;
	_vehicle setVariable ["Helipad", _pad, true];
}
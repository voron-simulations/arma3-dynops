/* Finds locations on map and sets relevant variables in mission namespace
 Output variables: 
	CivilianLocations
	MilitaryLocations
	AirportLocations
    StrategicLocations (all of the above)
*/

params ["_area"];

private _radius = worldSize / (sqrt 2);
private _center = [worldSize/2, worldSize/2, 0];

private _civilianLocationTypes = [
	"storage",
	"factory",
	"power plant"
];
private _militaryLocationTypes = [
	"military"
];
if (isNil "_area") then {
	_area = [_center, _radius, _radius, 0, true];
};

fnc_isMilitary = {
	params ["_location"];
	_found = false;
	private _name = text _location;
	{
		if (_x in toLower _name) exitWith { _found = true; };
	} forEach _militaryLocationTypes;
	_found;
};

// Civilian locations
CivilianLocations = nearestLocations [_center, ["NameCityCapital", "NameCity", "NameVillage"], _radius];
CivilianLocations = CivilianLocations select { (locationPosition _x) inArea _area; };

// Airports
AirportLocations = [];
{
	_x params ["_pos", "_size"];
	private _loc = createLocation ["Airport", _pos, _size select 0, _size select 1];
	_loc setRectangular true;
	_loc setVariable ["kind", "Airport"];
	AirportLocations pushBack _loc;
} forEach (call DW_fnc_getAirports);
AirportLocations = AirportLocations select { (locationPosition _x) inArea _area; };

// Military locations
private _militaryLocations = nearestLocations [_center, ["Airport", "NameLocal"], _radius];
_militaryLocations = _militaryLocations select { _x call fnc_isMilitary };

// Create copies of existing locations, to allow using variables
_militaryLocations = _militaryLocations apply {
	private _loc = createLocation ["Strategic", locationPosition _x, size _x # 0, size _x # 1];
	_loc setRectangular rectangular _x;
	_loc setText text _x;
	_loc;
};

{
	private _pos = position _x;
	private _loc = createLocation ["Strategic", _pos, 250, 250];
	_loc setText format ["Base %1", _loc call BIS_fnc_locationDescription];
	_loc setRectangular false;
	_militaryLocations pushBack _loc;
	deleteVehicle _x;
} forEach (_center nearObjects ["LocationBase_F", _radius]);

MilitaryLocations = _militaryLocations select { (locationPosition _x) inArea _area; };

StrategicLocations = MilitaryLocations + CivilianLocations + AirportLocations;

diag_log format [
	"Initialized %1 military, %2 civilian, %3 airport locations", 
	count MilitaryLocations,
	count CivilianLocations,
	count AirportLocations
];
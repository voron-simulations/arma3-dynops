/*
	Computes a bounding square for every airfield on the current map: the main
	airport (cfgWorlds >> worldName) plus each class under its SecondaryAirports.

	Each square is centred on the bounding box of the airfield's ILS taxi points
	(ilsTaxiIn + ilsTaxiOff). sizeX and sizeY are both the larger span (max - min)
	of that box, scaled by 1.2 to cover the apron.

	Returns: ARRAY of [[x, y], [sizeX, sizeY]] - one entry per airfield, with
		[x, y] the centre of the taxi-point bounding box
*/
private _configItems = "true" configClasses (configFile >> "cfgWorlds" >> worldName >> "SecondaryAirports");
_configItems pushBack (configFile >> "cfgWorlds" >> worldName);

private _airfields = [];

{
	private _pos = getArray (_x >> "ilsPosition");
	private _dir = getArray (_x >> "ilsDirection");
	private _taxi_points = getArray (_x >> "ilsTaxiIn");
	_taxi_points append getArray (_x >> "ilsTaxiOff");
	
	private _min_x = worldSize;	
	private _max_x = 0;
	private _min_y = worldSize;
	private _max_y = 0;

	for "_i" from 0 to (count _taxi_points - 1) step 2 do {
		_min_x = _min_x min (_taxi_points select _i);
		_max_x = _max_x max (_taxi_points select _i);
		_min_y = _min_y min (_taxi_points select (_i + 1));
		_max_y = _max_y max (_taxi_points select (_i + 1));
	};

	private _pos_x = (_max_x + _min_x) / 2;
	private _pos_y = (_max_y + _min_y) / 2;
	private _size = (_max_x - _min_x) max (_max_y - _min_y);

	_airfields pushBack [[_pos_x,_pos_y],[_size * 1.2, _size * 1.2]];
} forEach _configItems;

_airfields;

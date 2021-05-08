private _configItems = "true" configClasses (configFile >> "cfgWorlds" >> worldName >> "SecondaryAirports");
_configItems pushBack (configFile >> "cfgWorlds" >> worldName);

private _airfields = [];

//_cnt = 0;

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

		// DEBUG
		//_cnt = _cnt + 1;
		//_marker = createMarker ["taxi_marker_" + str _cnt, [_taxi_points select _i, _taxi_points select (_i+1), 0]];
		//_marker setMarkerShape "ICON";
		//_marker setMarkerType "mil_dot";
	};

	private _pos_x = (_max_x + _min_x) / 2;
	private _pos_y = (_max_y + _min_y) / 2;
	private _size = (_max_x - _min_x) max (_max_y - _min_y);

	_airfields pushBack [[_pos_x,_pos_y],[_size * 1.2, _size * 1.2]];
} forEach _configItems;

_airfields;
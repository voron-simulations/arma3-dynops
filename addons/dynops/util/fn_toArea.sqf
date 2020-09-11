private _loc = _this;

// Already an array
if (typeName _loc == "ARRAY") exitWith { _loc; };

// Location
if (typeName _loc == "LOCATION") exitWith {
	private _size = size _loc;
	private _pos = locationPosition _loc;
	[
		[_pos # 0, _pos # 1],
		(_size # 0) / 2, (_size # 1) / 2,
		direction _loc,
		rectangular _loc
	];
};

// Marker
if (typeName _loc == "STRING") exitWith {
	private _pos = getMarkerPos _loc;
	private _size = getMarkerPos _loc;
	[
		[_pos # 0, _pos # 1],
		_size # 0, _size # 1,
		markerDir _loc,
		markerShape _loc == "RECTANGULAR"
	];
};

// Trigger
if (typeName _loc == "OBJECT") exitWith {
	private _pos = getPos _loc;
	[[_pos # 0, _pos # 1]] + (triggerArea _loc);
};
private _id = 0;
{
	_id = _id + 1;
	_size = selectMax size _x; // diameter
	
	_marker = createMarker ["location_marker_" + str _id, locationPosition _x];
	if (rectangular _x) then { _marker setMarkerShape "RECTANGLE"; } else { _marker setMarkerShape "ELLIPSE" };
	_marker setMarkerDir direction _x;
	_marker setMarkerSize [_size / 2, _size / 2];
	_marker setMarkerPos position _x;
	_marker setMarkerText text _x;
	_marker setMarkerColor (_x call DW_fnc_getSideColor);
} forEach _this;
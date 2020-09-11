private _locations = _this;
private _helipads = [_locations, "HeliH"] call DW_fnc_getObjectsInLocation;

private _id = 0;
{
	_x params ["_pad", "_side"];
	_id = _id + 1;
	
	_marker = createMarker ["helipad_marker_" + str _id, position _pad];
	_marker setMarkerShape "ICON";
	_marker setMarkerType "c_air";
	_marker setMarkerDir direction _pad;
} forEach _helipads;
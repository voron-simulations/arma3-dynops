// 'Location' is the only input parameter.
// Can be Location, Marker, Trigger, Area array
private _area = _this call BIS_fnc_getArea;

(_area # 0) params ["_size_x", "_size_y"];
_radius = [0, 0, 0] vectorDistance [_size_x, _size_y, 0];

// Find all buildings in radius
private _buildings = [_size_x, _size_y] nearObjects ["House", _radius];

// Filter them down to those which are actually in area
_buildings = (_buildings inAreaArray _area) select { _x call BIS_fnc_isBuildingEnterable};

_buildings;
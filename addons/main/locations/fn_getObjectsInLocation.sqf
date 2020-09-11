params ["_locations", "_type"];

fnc_getSingleLocationObjects = {
	private _pos = locationPosition _this;
	private _size = selectMax size _this;

	private _objects = _pos nearObjects [_type, _size * (sqrt 2)] inAreaArray _this;
	
	_objects;
};

if (typeName _locations == "ARRAY") then {
	private _result = [];
	{ _result append (_x call fnc_getSingleLocationObjects); } forEach _locations;
	_result = _result arrayIntersect _result;
	_result;
} else {
	_locations call fnc_getSingleLocationObjects;
};
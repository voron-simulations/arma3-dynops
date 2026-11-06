/*
	Creates real Arma Location objects for annotated detections, gated by a
	minimum class -- creating one Location per 2-house farm across all of
	Altis (~780 of them) would slow down every nearestLocations call in the
	mission for marginal value.

	Params:
		0: ARRAY of HashMap - annotated locations, as returned by
			DynOps_fnc_annotateLocations
		1: NUMBER (optional) - minimum class index to create an object for
			(default: 1, Hamlet)

	Returns: ARRAY of Location - the created Location objects
*/
#include "script_component.hpp"

params ["_locations", ["_minClass", 1]];

private _created = [];
{
	private _location = _x;
	if ((_location get "class") >= _minClass) then {
		private _loc = createLocation [
			_location get "type",
			_location get "pos",
			_location get "a",
			_location get "b"
		];
		_loc setDirection (_location get "dir");
		_loc setText (_location get "name");
		_loc setRectangular true;
		_loc setVariable [QGVAR(Detection), _location];
		_created pushBack _loc;
	};
} forEach _locations;

INFO_2("Created %1 location objects (of %2 detections)",count _created,count _locations);
_created;

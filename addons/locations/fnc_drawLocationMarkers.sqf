/*
	Debug tool: runs the full detection pipeline (collect -> detect ->
	annotate) and draws one RECTANGLE marker per detected location, coloured
	by class and labelled with its name and building count. Clears its own
	previously-drawn markers first, so it is safe to re-run repeatedly from
	the debug console.

	Params:
		0: NUMBER (optional) - minimum class index to draw (default: 0, Farm)

	Returns: ARRAY of String - created marker names
*/
#include "script_component.hpp"

params [["_minClass", 0]];

if (!isNil QGVAR(DebugMarkers)) then {
	{ deleteMarkerLocal _x } forEach GVAR(DebugMarkers);
};

// Index-aligned with the Rust `LocationClass` enum (src/locations.rs).
private _classColors = ["ColorYellow", "ColorOrange", "ColorRed", "ColorPink", "ColorBlue"];

private _buildings = call DynOps_fnc_collectBuildings;
private _locations = [_buildings] call DynOps_fnc_detectLocations;
_locations = [_locations] call DynOps_fnc_annotateLocations;
_locations = _locations select { (_x get "class") >= _minClass };

private _markers = _locations apply {
	private _uuid = call DynOps_fnc_uuid;
	createMarkerLocal [_uuid, _x get "pos"];
	_uuid setMarkerShapeLocal "RECTANGLE";
	_uuid setMarkerSizeLocal [_x get "a", _x get "b"];
	_uuid setMarkerDirLocal (_x get "dir");
	_uuid setMarkerColorLocal (_classColors select (_x get "class"));
	_uuid setMarkerAlphaLocal 0.6;
	_uuid setMarkerTextLocal format ["%1 (%2)", _x get "name", _x get "buildings"];
	_uuid
};

GVAR(DebugMarkers) = _markers;
INFO_2("Drew %1 location markers (of %2 detections)",count _markers,count _locations);
_markers;

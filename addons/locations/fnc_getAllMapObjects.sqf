/*
  Finds locations on map and sets relevant variables in mission namespace
*/

#include "script_component.hpp"

INFO("Started location detection");
private _radius = worldSize / (sqrt 2);
private _center = [worldSize/2, worldSize/2, 0];

GVAR(Hospitals) = nearestTerrainObjects [_center, ["HOSPITAL"], _radius, false, true];

//private _fences = nearestTerrainObjects [_center, ["WALL", "FENCE"], _radius, false, true];
private _houses = nearestTerrainObjects [_center, ["BUILDING", "HOUSE"], _radius, false, true];
private _enterables = _houses; // TODO: better filtering
GVAR(EnterableBuildings) = _enterables;

INFO_1("Detected %1 objects",count _enterables);
private _input = _enterables joinString endl;

private _clusters = parseSimpleArray (["cluster", [_input]] call DynOps_fnc_call);

{
	private _uuid = call DynOps_fnc_uuid;
	createMarker [_uuid, _x # 0];
	_uuid setMarkerSizeLocal [ _x # 1, _x # 2];
	_uuid setMarkerDirLocal ( _x # 3 );
	_uuid setMarkerShapeLocal "RECTANGLE";
	// Last call not local to propagate all changes
	_uuid setMarkerColor "ColorRed";
} forEach _clusters;

INFO("Locations initialized");

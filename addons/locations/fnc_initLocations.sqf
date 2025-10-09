/*
  Finds locations on map and sets relevant variables in mission namespace
*/

#include "script_component.hpp"

private _radius = worldSize / (sqrt 2);
private _center = [worldSize/2, worldSize/2, 0];

private _houses = nearestTerrainObjects [_center, ["BUILDING", "HOUSE"], _radius, false, true];
private _enterables = _houses select { _x call BIS_fnc_isBuildingEnterable};
private _input = (_enterables apply { format ["%1,%2", position _x # 0, position _x # 1] }) joinString endl;

private _clusters = parseSimpleArray (["cluster", [_input]] call DynOps_fnc_call);

{
	private _uuid = call DynOps_fnc_uuid;
	createMarker [_uuid, _x # 0];
	_uuid setMarkerSizeLocal [ _x # 1, _x # 2];
	_uuid setMarkerDirLocal ( _x # 3 );
	_uuid setMarkerShapeLocal "ELLIPSE";
	_uuid setMarkerColor "ColorRed";
} forEach _clusters;

INFO("Locations initialized");

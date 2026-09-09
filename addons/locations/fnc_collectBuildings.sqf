/*
	Scans the whole map for enterable buildings. The single canonical building
	scan for the locations component -- replaces the duplicated
	nearestTerrainObjects calls that used to live in fnc_initLocations.sqf and
	fnc_getAllMapObjects.sqf.

	Returns: ARRAY of Object - enterable BUILDING/HOUSE class objects
*/
#include "script_component.hpp"

private _center = [worldSize / 2, worldSize / 2, 0];
private _radius = worldSize / sqrt 2;

private _buildings = nearestTerrainObjects [_center, ["BUILDING", "HOUSE"], _radius, false, true];
_buildings select { _x call BIS_fnc_isBuildingEnterable };

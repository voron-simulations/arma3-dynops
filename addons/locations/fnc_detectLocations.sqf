/* Finds locations on map and sets relevant variables in mission namespace
 Output variables: 
*/

#include "script_component.hpp"

private _radius = worldSize / (sqrt 2);
private _center = [worldSize/2, worldSize/2, 0];
private _enterables = nearestTerrainObjects [_center, ["House"], _radius, false, true];
_enterables = _enterables select { _x call BIS_fnc_isBuildingEnterable; };

// Extract 2D position from pertinent objects
_enterables = _enterables apply { 
	private _pos = position _x; 
	[_pos # 0, _pos # 1];
};

private _input = _enterables joinString endl;

private _clusters = parseSimpleArray (["cluster", [_input]] call EFUNC(extension,callExtension));

{
	private _uuid = call EFUNC(extension,uuid);
	createMarker [_uuid, _x # 0];
	_uuid setMarkerSize [ 2 * _x # 1, 2 * _x # 2];
	_uuid setMarkerDir ( _x # 3 );
	_uuid setMarkerShape "ELLIPSE";
	_uuid setMarkerColor "ColorRed";
} forEach _clusters;

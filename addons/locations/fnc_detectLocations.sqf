/* Finds locations on map and sets relevant variables in mission namespace
 Output variables: 
	GVAR(CivilianLocations)
	GVAR(MilitaryLocations)
	GVAR(AirportLocations)
    GVAR(StrategicLocations) (all of the above)
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

_input = _enterables joinString endl;

["cluster", [_input]] call EFUNC(extension,callExtension);

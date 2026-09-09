/*
	Orchestrates the full location-detection pipeline for the current map:
	collects buildings, detects settlements, annotates them, and creates real
	Location objects for downstream (population/intel/dynsim) use.

	Publishes:
		GVAR(EnterableBuildings) - ARRAY of Object, the buildings the
			detection ran against
		GVAR(Locations) - ARRAY of HashMap, every detected+annotated
			location (including ones too small to get a real Location object)
		GVAR(LocationObjects) - ARRAY of Location, the created Location
			objects (gated to GVAR(Locations) entries at or above the
			default minimum class)
*/
#include "script_component.hpp"

INFO("Starting location detection");

GVAR(EnterableBuildings) = call DynOps_fnc_collectBuildings;
INFO_1("Collected %1 enterable buildings",count GVAR(EnterableBuildings));

private _locations = [GVAR(EnterableBuildings)] call DynOps_fnc_detectLocations;
_locations = [_locations] call DynOps_fnc_annotateLocations;
GVAR(Locations) = _locations;

GVAR(LocationObjects) = [_locations] call DynOps_fnc_createLocationObjects;

INFO_2("Location detection complete: %1 detections, %2 Location objects",count _locations,count GVAR(LocationObjects));

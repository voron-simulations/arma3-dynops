#include "script_component.hpp"

if (!isMultiplayer || isServer) then {
	// postInit is not ordered after init.sqf/initServer.sqf, so mission
	// variables such as dynops_locations_defaultOwner may not be set yet. The
	// mission clock only starts once those scripts have been launched.
	[{ time > 0 }, { call DynOps_fnc_initLocations; }] call CBA_fnc_waitUntilAndExecute;
};

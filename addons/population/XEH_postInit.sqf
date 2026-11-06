#include "script_component.hpp"

if (!isMultiplayer || isServer) then {
	// Budgets and the seed are mission variables set in init.sqf/initServer.sqf,
	// which postInit is not ordered after, so wait for the mission clock. The
	// second condition makes the dependency on location detection explicit
	// rather than relying on callback order.
	[
		{ time > 0 && { !isNil QEGVAR(locations,Locations) } },
		{ call DynOps_fnc_initPopulation; }
	] call CBA_fnc_waitUntilAndExecute;
};

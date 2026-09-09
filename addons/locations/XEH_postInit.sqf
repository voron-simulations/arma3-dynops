#include "script_component.hpp"

if (!isMultiplayer || isServer) then {
	call DynOps_fnc_initLocations;
};

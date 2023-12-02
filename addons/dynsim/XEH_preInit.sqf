#include "script_component.hpp"

[
	"CAManBase", 
	"Init", 
	{ (_this select 0) addEventHandler ["PathCalculated", DynOps_fnc_onPathCalculated]; }
] call CBA_fnc_addClassEventHandler;

INFO("PreInit finished");

#include "script_component.hpp"

params ["_side"];

[GVAR(SideIntel), _side] call CBA_fnc_hashGet;

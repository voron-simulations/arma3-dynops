#include "script_component.hpp"

params ["_unit", "_anim"];
if (local _unit && {_anim == AI_STUCK_RELOAD_ANIM}) then {
    [_unit] call DynOps_fnc_fixAiReloadAnim;
};

#include "script_component.hpp"

params ["_unit"];
if (!local _unit || isPlayer _unit) exitWith {};

_unit addEventHandler ["AnimChanged", DynOps_fnc_onAiAnimChanged];

// Catch units already spawned into the stuck state before this handler was attached.
if (animationState _unit == AI_STUCK_RELOAD_ANIM) then {
    [_unit] call DynOps_fnc_fixAiReloadAnim;
};

#include "script_component.hpp"

addMissionEventHandler ["ExtensionCallback", DynOps_fnc_callback];

addMissionEventHandler ["Ended", { 
    params ["_endType"];
    ["onMissionEnded"] call DynOps_fnc_call;
}];

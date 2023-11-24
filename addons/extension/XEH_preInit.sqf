#include "script_component.hpp"

addMissionEventHandler ["ExtensionCallback", FUNC(callback)];

addMissionEventHandler ["Ended", { 
    params ["_endType"];
    ["onMissionEnded"] call FUNC(call);
}];

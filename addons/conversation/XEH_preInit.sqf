#include "script_component.hpp"

addMissionEventHandler ["HandleChatMessage", FUNC(onDirectMessage)];

GVAR(agents) = createHashMap;

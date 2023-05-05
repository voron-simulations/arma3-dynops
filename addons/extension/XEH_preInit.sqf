#include "script_component.hpp"

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_component", "_data"];
    if ((tolower _name) != "dynops") exitWith {};
    hint format ["%1 %2 %3", _name, _component, _data];
}];

INFO("PreStart finished");

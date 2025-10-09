#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = COMPONENT_NAME;
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"dynops_main", "dynops_factions", "dynops_civilians", "cba_main"};

        VERSION_CONFIG;
    };
};

#include "CfgFunctions.hpp"

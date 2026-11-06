#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = COMPONENT_NAME;
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"dynops_main", "dynops_extension", "dynops_factions", "dynops_locations", "dynops_civilians", "cba_main"};

        VERSION_CONFIG;
    };
};

#include "CfgDynOpsCosts.hpp"
#include "CfgEventHandlers.hpp"
#include "CfgFunctions.hpp"

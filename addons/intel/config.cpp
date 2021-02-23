#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = COMPONENT_NAME;
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"dynops_main", "cba_main"};
        author = ECSTRING(main,Team);
        VERSION_CONFIG;
    };
};

#include "CfgFunctions.hpp"
#include "CfgNotifications.hpp"

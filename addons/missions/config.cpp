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

class CfgMissions {
    class MPMissions {
        class MP_COOP_Dynops_Altis_CTRG_Freeroam {
            directory = "z\dynops\addons\missions\MPMissions\CTRG_Freeroam.altis";
            briefingName = "DynOps | Altis CTRG Freeroam";
            author = "Dynamic Operations Team";
        };
    };
};

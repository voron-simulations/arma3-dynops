#include "script_component.hpp"

class CfgPatches
{
    class ADDON
    {
        name = COMPONENT_NAME;
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"cba_main"};
        author = CSTRING(Team);
        VERSION_CONFIG;
    };
};

class CfgMods
{
    class DynamicOperations
    {
        dir = "@dynops";
        name = "Dynamic Operations";
        picture = "A3\Ui_f\data\Logos\arma3_expansion_alpha_ca";
        hidePicture = "true";
        hideName = "true";
        actionName = "Website";
        action = "https://github.com/DarkWanderer/DynamicOperations";
        description = "Issue Tracker: https://github.com/DarkWanderer/DynamicOperations/issues";
    };
};

#include "CfgEventHandlers.hpp"
#include "CfgFunctions.hpp"

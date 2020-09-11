#include "CfgVehicles.hpp"
#include "CfgFunctions.hpp"

class CfgPatches {
    class DynamicOperations {
        name = "Dynamic Operations";
        units[] = {};
        weapons[] = {};
        requiredVersion = 1.98;
        requiredAddons[] = {"cba_main"};
        author = "DarkWanderer";
        url = "https://github.com/DarkWanderer/DynamicOperations";
    };
};

class CfgMods {
    class DynamicOperations {
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
#include "script_component.hpp"

[
    _this, // Object the action is attached to
    "Hack", // Title of the action
    "\a3\ui_f\data\IGUI\Cfg\holdactions\holdAction_hack_ca.paa", // Idle icon shown on screen
    "\a3\ui_f\data\IGUI\Cfg\holdactions\holdAction_hack_ca.paa", // Progress icon shown on screen
    "_this distance _target < 3", // Condition for the action to be shown
    "_caller distance _target < 3", // Condition for the action to progress
    {
        player setPos (_target getRelPos [0.5, 0]);
        player setDir (180 + direction _target);
        [_caller, "Acts_Accessing_Computer_Loop"] remoteExec ["playMove"];
        ["a3\missions_f_oldman\data\sound\intel_laptop\1sec\intel_laptop_1sec_02.wss", target] remoteExec ["playSound3d"];
        _target setObjectTextureGlobal [0, "a3\structures_f\items\electronics\data\electronics_screens_laptop_co.paa"];
    }, // Code executed when action starts
    {}, // Code executed on every progress tick
    { 
        _caller addMagazineGlobal "FlashDisk"; 
        [_caller,"Acts_Accessing_Computer_Out_Short"] remoteExec ["playMove"];
    }, // Code executed on completion
    { [_caller,"Acts_Accessing_Computer_Out_Short"] remoteExec ["playMove"]; }, // Code executed on interrupted
    [], // Arguments passed to the scripts as _this select 3
    8, // Action duration in seconds
    0, // Priority
    true, // Remove on completion
    false // Show in unconscious state
] remoteExec ["BIS_fnc_holdActionAdd", 0, _this]; // MP compatible implementation

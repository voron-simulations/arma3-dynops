#include "\dw\dynops\addons\main\script_mod.hpp"


#define COMPONENT main
#define COMPONENT_BEAUTIFIED Main

// AI can get stuck in this prone-reload animation state and never leave it; force them into the standing equivalent when it happens.
#define AI_STUCK_RELOAD_ANIM "acinpknlmstpsraswrfldnon"
#define AI_STUCK_RELOAD_ANIM_FIX "amovppnemstpsraswrfldnon"

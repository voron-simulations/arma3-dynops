
private _message = format _this;

if (isMultiplayer) exitWith { diag_log _message; };
if (serverCommandAvailable "#kick") exitWith { player sideChat _message; };
#include "script_component.hpp"

INFO("Starting performance monitoring");

while { true; } do {
	sleep 120;
	private _fps = diag_fps;
	private _activeScripts = diag_activeScripts;
	private _activeGroups = count allGroups;
	private _activeUnits = { simulationEnabled _x } count allUnits;
	private _totalUnits = count allUnits;
	INFO_5("FPS: %1, scripts spawn/execVM/SQS/FSM: %2, groups: %3, units active/total: %4/%5",
		_fps, _activeScripts, _activeGroups, _activeUnits, _totalUnits);
};

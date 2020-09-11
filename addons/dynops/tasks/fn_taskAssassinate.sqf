params ["_owner", "_target"];

private _taskId = "taskAssassinate" call DW_fnc_generateUID;

[
	_owner,
	_taskId,
	["Eliminate provided target. Location is currently unknown", "Eliminate " + name _target, ""],
	objNull,
	"CREATED",
	0,
	true,
	"target"
] call BIS_fnc_taskCreate;

[_taskId, _target] call BIS_fnc_taskDestination;

removeFromRemainsCollector [_target];
waitUntil { sleep 1; !alive _target; };

[_taskId, "SUCCEEDED", true] call BIS_fnc_taskSetState;
addToRemainsCollector [_target];
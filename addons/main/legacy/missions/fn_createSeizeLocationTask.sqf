params ["_owner", "_location"];

private _pos = locationPosition _location;
private _taskId = "seize" + str random 999999999;

[	
	_owner, _taskId,
	["Capture military base at designated coordinates", "Capture " + text _location, "marker"],
	_pos, "CREATED", -1, true, "attack"
] call BIS_fnc_taskCreate;

private _trigger = createTrigger ["EmptyDetector", _pos, true];
_trigger setTriggerArea (_location call CBA_fnc_getArea);
_trigger setTriggerTimeout [20, 30, 60, true];
_trigger setTriggerActivation ["WEST SEIZED", "PRESENT", true];
_trigger setTriggerStatements [
	"this", 
	format["['%1', 'SUCCEEDED', true] call BIS_fnc_taskSetState;", _taskId], 
	format["['%1', 'CREATED', true] call BIS_fnc_taskSetState;", _taskId]
];
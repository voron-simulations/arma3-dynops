params ["_helipad"];

private _clutterClasses = [
	"CargoNet_01_barrels_F",
	"CargoNet_01_box_F",
	"FlexibleTank_01_forest_F",
	"Land_CanisterFuel_White_F",
	"Land_DischargeStick_01_F",
	"Land_HelicopterWheels_01_assembled_F",
	"Land_RefuelingHose_01_F",
	"Land_RotorCoversBag_01_F",
	"Land_WheelChock_01_F"
];

private _minrange = 8;
private _maxrange = 12;

// Place clutter
for "_i" from random 5 to 0 step -1 do {
	private _pos = [getPos _helipad, _minrange, _maxrange] call BIS_fnc_findSafePos;
	if (count _pos == 2) then { _pos = _pos + [0]; };
	createSimpleObject [selectRandom _clutterClasses, ATLToASL _pos, false];
};
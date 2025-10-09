params ["_caller", "_pos", "_target", "_is3D", "_id"];

if (count _pos < 1) exitWith {};

_pos params ["_x", "_y", "_z"];

_altitude = 1200;
_radius = 1000;

private _start_pos = [0,0,_altitude];
private _target_pos = [_x, _y, _altitude];

private _uav = createVehicle ["B_UAV_05_F", _start_pos, [], 0, "FLY"];
_uav setPosATL _start_pos;
_uav setDir (_uav getRelDir _target_pos);
_uav setVelocity (vectorDir _uav vectorMultiply 150);
_group = createVehicleCrew _uav;

// Set UAV loadout
// Previously used PylonRack_4Rnd_LG_scalpel
private _pylons = ["PylonRack_Bomb_SDB_x4", "PylonRack_Bomb_SDB_x4"];
private _pylonPaths = (configProperties [configOf _uav >> "Components" >> "TransportPylonsComponent" >> "Pylons", "isClass _x"]) apply {getArray (_x >> "turret")};
{ _uav removeWeaponGlobal getText (configFile >> "CfgMagazines" >> _x >> "pylonWeapon") } forEach getPylonMagazines _uav;
{ _uav setPylonLoadout [_forEachIndex + 1, _x, true, _pylonPaths select _forEachIndex] } forEach _pylons;

_uav removeMagazineGlobal "Laserbatteries";
_uav removeWeaponGlobal "Laserdesignator_mounted";

// Restrict UAV driver control
_uav lockDriver true;
_uav enableUAVWaypoints false;

// Add LOITER waypoint around the target location
_wp = (group _uav) addWaypoint [_target_pos,0];
_wp setWaypointType "LOITER";
_wp setWaypointLoiterRadius _radius;
_wp setWaypointLoiterType "CIRCLE_L";
_uav flyInHeight _altitude;

_uav setVehicleReportRemoteTargets true;
_uav setVehicleReceiveRemoteTargets true;

_start = time;

while { time < _start + 300 && alive _uav; } do
{
    // Loop while
};

if (alive _uav) then {
    // Disconnect UAV controllers
    _controller = UAVControl _uav select 0;
    _controller connectTerminalToUAV objNull;
    _uav lockTurret [[0],true];

    _wp = (group _uav) addWaypoint [_start_pos, 500];
    _wp setWaypointType "MOVE";
    _wp setWaypointSpeed "FULL";
    (group _uav) setCurrentWaypoint _wp;
};

waitUntil { !alive _uav || _uav distance _start_pos < 1000; };

if (alive _uav) then {
    { _uav deleteVehicleCrew _x } forEach (crew _uav);
    deleteVehicle _uav;
};

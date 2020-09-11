params ["_group"];

private _currentWaypoint = (waypoints _group) param [currentWaypoint _group];
if (isNil "_currentWaypoint") then { 0; } else { (leader _group) distance waypointPosition _currentWaypoint; };

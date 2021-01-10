params ["_side", "_points"];

if (!isMultiplayer || isServer) then
{
	// If server, perform the addition and publish result
	if (isNil "IntelPoints") then { IntelPoints = 0; };
	IntelPoints = IntelPoints + _points;
	publicVariable "IntelPoints";
}
else 
{
	// Defer execution to server
	_this remoteExec ["DW_fnc_addIntelPoints", 2, false]; 
};

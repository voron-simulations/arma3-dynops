/*
	Drives the chunked "locations:*" extension protocol against a set of
	building positions and returns the detected settlements.

	callExtension caps a single call's output at 10240 bytes and this
	component's own input chunks stay well under that too (~190KB of raw
	coordinates for Altis needs many "add" calls) -- so both directions are
	chunked: buildings are sent in batches, and results are paged back out
	until DynOps_fnc_call's underlying "locations:page" offset catches up
	with the detected count.

	Params:
		0: ARRAY of Object - buildings to detect settlements from, as
			returned by DynOps_fnc_collectBuildings
		1: ARRAY (optional) - detection params [eps, maxSpan, splitFactor,
			minEps, minBuildings], default [100, 700, 0.7, 35, 2]

	Returns: ARRAY of HashMap - one per detected location, keys:
		"pos" (Position2D), "a" (Number), "b" (Number), "dir" (Number,
		compass degrees), "buildings" (Number), "class" (Number, 0=Farm
		.. 4=City)
*/
#include "script_component.hpp"

params [["_buildings", [], [[]]], ["_params", [100, 700, 0.7, 35, 2], [[]]]];
_params params ["_eps", "_maxSpan", "_splitFactor", "_minEps", "_minBuildings"];

// Batch size tuned so each "x,y\n..." chunk (~20 bytes/coordinate pair)
// stays comfortably under callExtension's input size limits.
private _batchSize = 400;

["locations:begin"] call DynOps_fnc_call;

private _sent = 0;
while { _sent < count _buildings } do {
	private _batch = _buildings select [_sent, _batchSize];
	private _chunk = (_batch apply { format ["%1,%2", position _x # 0, position _x # 1] }) joinString endl;
	["locations:add", [_chunk]] call DynOps_fnc_call;
	_sent = _sent + _batchSize;
};

private _detectedCount = parseNumber (["locations:detect", [_eps, _maxSpan, _splitFactor, _minEps, _minBuildings]] call DynOps_fnc_call);
INFO_1("Detected %1 locations",_detectedCount);

private _locations = [];
private _offset = 0;
while { _offset < _detectedCount } do {
	private _page = parseSimpleArray (["locations:page", [_offset]] call DynOps_fnc_call);
	_page params ["_nextOffset", "_entries"];

	{
		_x params ["_pos", "_a", "_b", "_dir", "_buildingCount", "_classIndex"];
		private _location = createHashMap;
		_location set ["pos", _pos];
		_location set ["a", _a];
		_location set ["b", _b];
		_location set ["dir", _dir];
		_location set ["buildings", _buildingCount];
		_location set ["class", _classIndex];
		_locations pushBack _location;
	} forEach _entries;

	// A page never regresses; if it somehow failed to advance, bail rather
	// than loop forever.
	if (_nextOffset <= _offset) exitWith {};
	_offset = _nextOffset;
};

["locations:end"] call DynOps_fnc_call;

_locations;

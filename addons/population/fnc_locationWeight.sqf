/*
	Relative share of a faction's budget a location should receive. The class
	steps are coarse on purpose: weighting by raw building count would let
	cities soak up nearly the whole budget and leave the countryside empty
	(see adr/0002-population.md). The formula lives in SQF rather than the
	extension so that mod-specific flags stay out of the Rust side, which only
	ever sees the resulting number.

	Params:
		0: HashMap - an annotated location, as produced by
			DynOps_fnc_annotateLocations

	Returns: NUMBER - class base (Farm 1, Hamlet 2, Village 4, Town 8, City
		16), x3 if the location is military, x2 if it contains an airport
*/
#include "script_component.hpp"

params [["_location", createHashMap, [createHashMap]]];

// Clamped so an unexpected class index from a future extension version still
// yields a weight instead of a script error on select.
private _class = 0 max ((_location getOrDefault ["class", 0]) min 4);
private _weight = [1, 2, 4, 8, 16] select _class;

if (_location getOrDefault ["isMilitary", false]) then { _weight = _weight * 3; };
if (_location getOrDefault ["isAirport", false]) then { _weight = _weight * 2; };

_weight;

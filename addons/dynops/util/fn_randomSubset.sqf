params ["_items", "_count", "_unique"];

private _result = [];
private _source = +_items; // Copy array

while { count _source > 0 && count _result < _count } do {
	private _index = floor random count _source;
	if (_unique) then {
		_result pushBackUnique (_source # _index);
	} else {
		_result pushBack (_source # _index);
	};
	_source deleteAt _index;
};
_result;
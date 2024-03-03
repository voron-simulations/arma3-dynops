#include "script_component.hpp"

params ["_unit", "_path"];

hint format["PathCalculated: %1 %2", _unit, _path];
_unit setVariable [QGVAR(Path), _path];

#include "script_component.hpp"

params ["_uid", "_text"];
hint _this;
private _agent = GVAR(agents) get _uid;

if (isNil "_agent") exitWith {
	ERROR_1("Unknown agent %1", _uid);
};

private _playersInRange = (position _sender) nearObjects ["CAManBase", GVAR(ConversationRange)];
_playersInRange = _playersInRange select { isPlayer _x; };

INFO_2("%1 replies: %2", name _agent, _text);

_text remoteExec ["systemChat", _playersInRange];

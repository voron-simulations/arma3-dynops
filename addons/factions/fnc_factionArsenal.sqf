#include "script_component.hpp"

params ["_box", "_faction"];

["AmmoboxInit", _box] spawn BIS_fnc_arsenal;

private _factionData = GVARMAIN(FactionData) get _faction;

private _backpacks = _factionData get "Backpacks";
private _items = _factionData get "Items";
private _magazines = _factionData get "Magazines";
private _uniforms = _factionData get "Uniforms";
private _weapons = _factionData get "Weapons";

[_box, _backpacks, true] call BIS_fnc_addVirtualBackpackCargo;
[_box, _items, true] call BIS_fnc_addVirtualItemCargo;
[_box, _magazines, true] call BIS_fnc_addVirtualMagazineCargo;
[_box, _uniforms, true] call BIS_fnc_addVirtualItemCargo;
[_box, _weapons, true] call BIS_fnc_addVirtualWeaponCargo;

_box;

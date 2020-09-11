params ["_entity"];
switch (side _entity) do {
	case east: { "colorOPFOR" };
	case west: { "colorBLUFOR" };
	case independent: { "colorIndependent" };
	case civilian : { "colorCivilian" };
	default { "ColorUNKNOWN" };
};
private _missions = MilitaryLocations apply  {
	[
		locationPosition _x,
		{ hint str _this; },
		format ["Capture %1", text _x],
		"Eliminate all hostiles at this location",
		name player,
		"\A3\Data_F_Argo\Logos\arma3_argo_logoTitle_ca.paa",
		1.0,
		[ _x ]
	];
};

[
	findDisplay 46,
	[worldSize / 2, worldSize / 2,0],
	_missions,
	[], // ORBAT
	[], // MARKERS
	[
		[
			"\A3\Ui_f\data\Logos\arma3_white_ca.paa",
			[0,0,0,1],
			[4000,4000,0],
			8,
			8,
			0,
			"Arma 3 Logo",
			true
		]
	],
	0,
	false,
	1,
	true,
	"Mission selection",
	false
] call DW_fnc_strategicMapOpen;
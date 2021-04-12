#include "script_component.hpp"
/* ----------------------------------------------------------------------------
  Fills faction data structures with items/vehicles which are omitted in 
  CfgUnits config
---------------------------------------------------------------------------- */

// CTRG
private _factionData = [GVARMAIN(FactionData), "BLU_CTRG_F"] call CBA_fnc_hashGet;
private _items = [
	"H_HelmetB_TI_tna_F",
	"H_HelmetB_TI_arid_F",
	"G_Balaclava_TI_blk_F",
	"G_Balaclava_TI_tna_F",
	"G_Balaclava_TI_G_blk_F",
	"G_Balaclava_TI_G_tna_F",
	"V_PlateCarrier2_blk",
	"V_PlateCarrier1_blk",
	"V_PlateCarrierL_CTRG",
	"V_PlateCarrierH_CTRG",
	"V_PlateCarrier1_rgr_noflag_F",
	"V_PlateCarrier2_rgr_noflag_F",
	"muzzle_snds_338_black",
	"muzzle_snds_338_green",
	"muzzle_snds_338_sand",
	"bipod_01_F_snd",
	"bipod_01_F_blk",
	"bipod_01_F_mtp",
	"optic_ERCO_blk_F",
	"optic_ERCO_khk_F",
	"optic_ERCO_snd_F",
	"bipod_01_F_khk",
	"ItemGPS",
	"FirstAidKit",
	"B_UavTerminal",
	"NVGogglesB_blk_F",
	"NVGogglesB_grn_F",
	"NVGogglesB_gry_F"
];
private _weapons = ["srifle_dmr_02_f","srifle_dmr_02_camo_f","srifle_dmr_02_sniper_f","arifle_spar_01_blk_f","arifle_spar_01_khk_f","arifle_spar_01_snd_f","arifle_spar_01_gl_blk_f","arifle_spar_01_gl_khk_f","arifle_spar_01_gl_snd_f","arifle_spar_03_blk_f","arifle_spar_03_khk_f","arifle_spar_03_snd_f"];
private _magazines = ["10Rnd_338_Mag"];
private _uniforms = ["U_B_CTRG_1","U_B_CTRG_2","U_B_CTRG_3","U_B_CTRG_Soldier_F","U_B_CTRG_Soldier_2_F","U_B_CTRG_Soldier_3_F","U_B_CTRG_Soldier_Arid_F","U_B_CTRG_Soldier_2_Arid_F","U_B_CTRG_Soldier_3_Arid_F","U_B_CTRG_Soldier_urb_1_F","U_B_CTRG_Soldier_urb_2_F","U_B_CTRG_Soldier_urb_3_F"];

[_factionData, "Items", _items] call EFUNC(main,hashAdd);
[_factionData, "Magazines", _magazines] call EFUNC(main,hashAdd);
[_factionData, "Uniforms", _uniforms] call EFUNC(main,hashAdd);
[_factionData, "Weapons", _weapons] call EFUNC(main,hashAdd);

true;
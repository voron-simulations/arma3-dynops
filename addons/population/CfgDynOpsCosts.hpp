// Point cost per unit category, summed per group by DynOps_fnc_groupCost to
// price a group against a faction's population budget.
//
// Keys are "<type>_<subtype>" exactly as returned by BIS_fnc_objectType, see
// https://community.bistudio.com/wiki/BIS_fnc_objectType
// Units whose category is not listed here cost "default". Add entries (in
// this class, from your own addon's config) for categories a mod introduces.
class CfgDynOpsCosts {
    // Infantry. Every infantry role BIS_fnc_objectType can return is listed
    // explicitly so that a normal squad is never priced at the default.
    Soldier_Infantry = 1;
    Soldier_AT = 1;
    Soldier_MG = 1;
    Soldier_Medic = 1;
    Soldier_Officer = 1;
    Soldier_Civilian = 1;
    Soldier_Pilot = 1;
    Soldier_UAVPilot = 1;
    Soldier_Diver = 2;
    Soldier_Sniper = 2;
    Soldier_SpecialForces = 2;

    // Wheeled and light vehicles.
    Vehicle_Motorcycle = 2;
    Vehicle_Car = 5;
    Vehicle_WheeledAPC = 15;

    // Armour.
    Vehicle_TrackedAPC = 15;
    Vehicle_Tank = 30;

    // Air.
    Vehicle_Helicopter = 40;
    Vehicle_Plane = 60;

    // Naval.
    Vehicle_Ship = 10;
    Vehicle_Submarine = 10;

    // Emplaced weapons.
    Vehicle_StaticWeapon = 5;

    default = 5;
};

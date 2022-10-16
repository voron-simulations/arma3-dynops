#include "script_component.hpp"

private _intelItemCosts = 
[
	["Money", 5],
	["Money_roll", 20],
	["Money_stack", 50],
	["NetworkStructure", 100],
	["DocumentsSecret", 500],
	["FileTopSecret", 2000],
	["FilesSecret", 500],
	["Files", 100],
	["FlashDisk", 200],
	["Keys", 50],
	["Laptop_Closed", 300],
	["Laptop_Unfolded", 300],
	["SmartPhone", 250],
	["MobilePhone", 100],
	["SatPhone", 100],
	["Wallet_ID", 50]
];

GVAR(intelItemCosts) = createHashMapFromArray _intelItemCosts;

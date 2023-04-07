addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_component", "_data"];
    if ((tolower _name) != "dynops") exitWith {};
    hint _data;
    global_data = _data;
}];

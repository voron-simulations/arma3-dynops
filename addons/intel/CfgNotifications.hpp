class CfgNotifications {
    class IntelDelivered
    {
        title = "INTEL DELIVERED";  // Title displayed as text on black background. Filled by arguments.
        iconPicture = "\A3\ui_f\data\igui\cfg\simpleTasks\types\intel_ca.paa";
        description = "Intel delivered: +%1 points";   // Brief description displayed as structured text. Colored by "color", filled by arguments.
        color[] = {1,1,1,1};
        priority = 6;               // Priority; higher number = more important; tasks in queue are selected by priority
        sound = "taskSucceeded";
    };
};

class CfgFactionClasses {
    class NO_CATEGORY;
    class DynOps_Modules_Conversation: NO_CATEGORY {
        displayName = "Conversation";
    };
};

class CfgVehicles
{
	class Logic;
	class Module_F : Logic
	{
		class AttributesBase
		{
			class Default;
			class Edit;					// Default edit box (i.e. text input field)
			class Combo;				// Default combo box (i.e. drop-down menu)
			class Checkbox;				// Default checkbox (returned value is Boolean)
			class CheckboxNumber;		// Default checkbox (returned value is Number)
			class ModuleDescription;	// Module description
			class Units;				// Selection of units on which the module is applied
		};

		// Description base classes (for more information see below):
		class ModuleDescription
		{
			class AnyBrain;
		};
	};

	class DynOps_Module_Conversation : Module_F
	{
		// Standard object definitions:
		scope = 2;										// Editor visibility; 2 will show it in the menu, 1 will hide it.
		displayName = "Init Conversation Agent";		// Name displayed in the menu
		// icon = "";	// Map icon. Delete this entry to use the default icon.
		category = "DynOps_Modules_Conversation";

		function = QFUNC(initAgent);			// Name of function triggered once conditions are met
		functionPriority = 10;				// Execution priority, modules with lower number are executed first. 0 is used when the attribute is undefined
		isGlobal = 0;						// 0 for server only execution, 1 for global execution, 2 for persistent global execution
		isTriggerActivated = 0;				// 1 for module waiting until all synced triggers are activated
		isDisposable = 1;					// 1 if modules is to be disabled once it is activated (i.e. repeated trigger activation will not work)
		is3DEN = 0;							// 1 to run init function in Eden Editor as well
		curatorCanAttach = 1;				// 1 to allow Zeus to attach the module to an entity
		// curatorInfoType = "RscDisplayAttributeModuleNuke"; // Menu displayed when the module is placed or double-clicked on by Zeus

		// 3DEN Attributes Menu Options
		canSetArea = 0;						// Allows for setting the area values in the Attributes menu in 3DEN
		canSetAreaShape = 0;				// Allows for setting "Rectangle" or "Ellipse" in Attributes menu in 3DEN
		class AttributeValues
		{
			// This section allows you to set the default values for the attributes menu in 3DEN
			size3[] = { 100, 100, -1 };		// 3D size (x-axis radius, y-axis radius, z-axis radius)
			isRectangle = 0;				// Sets if the default shape should be a rectangle or ellipse
		};

		// Module attributes (uses https://community.bistudio.com/wiki/Eden_Editor:_Configuring_Attributes#Entity_Specific):
		class Attributes : AttributesBase
		{
			// Arguments shared by specific module type (have to be mentioned in order to be present):
			class Units : Units
			{
				property = "units";
			};

			class Occupation : Edit
			{
				property = "occupation";
				displayName = "Occupation";
				tooltip = "Occupation of the person";
				// Because this is an expression, one must have a string within a string to return a string
				defaultValue = """Office Clerk""";
			};

			class Age : Combo 
			{
				property = "age";
				displayName = "Age";
				tooltip = "How old is the person (has some effect on agent vocabulary)";
				typeName = "NUMBER";
				defaultValue = "25";

				// Listbox items
				class Values
				{
					class y20 { name = "20 years"; value = 20; };
					class y25 { name = "25 years"; value = 25; };
					class y30 { name = "30 years"; value = 30; };
					class y35 { name = "35 years"; value = 35; };
					class y40 { name = "40 years"; value = 40; };
				};
			};

			class Facts : Default
			{
				property = "facts";
				displayName = "Facts";
				tooltip = "Information which the person knows and may reference in conversation";
                control = "EditMulti5";
				defaultValue = "'Pyrgos is the capital of Altis'";
			};

			class ModuleDescription : ModuleDescription {}; // Module description should be shown last
		};

		// Module description (must inherit from base class, otherwise pre-defined entities won't be available)
		class ModuleDescription : ModuleDescription
		{
			description = "Short module description";	// Short description, will be formatted as structured text
			sync[] = { "LocationArea_F" };				// Array of synced entities (can contain base classes)

			class LocationArea_F
			{
				description[] = { // Multi-line descriptions are supported
					"Provides background for conversation agent which will be fed to language model",
					"Should contain basic text in instructional form"
				};
				position = 0;	// Position is taken into effect
				direction = 0;	// Direction is taken into effect
				optional = 0;	// Synced entity is optional
				duplicate = 0;	// Multiple entities of this type can be synced
				synced[] = { "AnyPerson" };	// Pre-defined entities like "AnyBrain" can be used (see the table below)
			};
		};
	};
};
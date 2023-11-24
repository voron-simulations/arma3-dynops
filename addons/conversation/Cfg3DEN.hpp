class Cfg3DEN {
	class Object {
		class AttributeCategories {
			class MyCategory {
				displayName = "Dynamic Conversation";
				collapsed = 1;
				class Attributes {
					class DynOps_ConversationEnabled {
						//--- Mandatory properties
						displayName = "Enable"; // Name assigned to UI control class Title
						tooltip = "Enable ChatGPT-based conversation for this person. Works only in multiplayer";
						property = "DynOps_ConversationEnabled"; // Unique config property name saved in SQM
						control = "Checkbox"; // UI control base class displayed in Edit Attributes window, points to Cfg3DEN >> Attributes
						expression = "_this setVariable ['%s', _value];";
						defaultValue = "false";

						//--- Optional properties
						condition = "objectBrain"; // Condition for attribute to appear (see the table below)
						typeName = "BOOL"; // Defines data type of saved value, can be STRING, NUMBER or BOOL. Used only when control is "Combo", "Edit" or their variants. This is a scripted feature and has no engine support. See code in (configFile >> "Cfg3DEN" >> "Attributes" >> "Combo" >> "attributeSave")
					};

					class DynOps_ConversationOccupation {
						//--- Mandatory properties
						displayName = "Occupation"; // Name assigned to UI control class Title
						tooltip = "Occupation of this person. Affects vocabulary and conversation style";
						property = "DynOps_ConversationOccupation"; // Unique config property name saved in SQM
						control = "Edit"; // UI control base class displayed in Edit Attributes window, points to Cfg3DEN >> Attributes
						expression = "_this setVariable ['%s', _value];";
						defaultValue = "Farmer";

						//--- Optional properties
						condition = "objectBrain"; // Condition for attribute to appear
						typeName = "STRING"; // Defines data type of saved value, can be STRING, NUMBER or BOOL. Used only when control is "Combo", "Edit" or their variants. This is a scripted feature and has no engine support. See code in (configFile >> "Cfg3DEN" >> "Attributes" >> "Combo" >> "attributeSave")
					};

					class DynOps_ConversationFacts {
						//--- Mandatory properties
						displayName = "Known Facts"; // Name assigned to UI control class Title
						tooltip = "Facts which this person knows about the world and can use in conversation";
						property = "DynOps_ConversationFacts"; // Unique config property name saved in SQM
						control = "EditMulti5"; // UI control base class displayed in Edit Attributes window, points to Cfg3DEN >> Attributes
						expression = "_this setVariable ['%s', _value];";
						defaultValue = "Arma 4 will be released soon";

						//--- Optional properties
						condition = "objectBrain"; // Condition for attribute to appear
						typeName = "STRING"; // Defines data type of saved value, can be STRING, NUMBER or BOOL. Used only when control is "Combo", "Edit" or their variants. This is a scripted feature and has no engine support. See code in (configFile >> "Cfg3DEN" >> "Attributes" >> "Combo" >> "attributeSave")
					};
				};
			};
		};
	};
};

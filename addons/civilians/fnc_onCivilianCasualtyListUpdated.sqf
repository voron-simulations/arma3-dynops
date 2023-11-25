#include "script_component.hpp"

params ["_varName", "_casualties"];

if (!(player diarySubjectExists "civilians")) then { player createDiarySubject ["civilians", "Civilians"]; };

if (isNil QGVAR(CasualtiesDiaryRecord)) then { GVAR(CasualtiesDiaryRecord) = player createDiaryRecord ["civilians", ["Casualties of war",""]]; };

private _text = "<img image='\dw\dynops\addons\civilians\images\graves.jpg' width='400' height='250' /><br /><br />";
_text = _text + format ["A total of %1 civilians became casualties of war:<br />", count _casualties];
_text = _text + (_casualties joinString ";  ");

player setDiaryRecordText [["civilians", GVAR(CasualtiesDiaryRecord)], ["Casualties of war", _text]];

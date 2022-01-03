Param($directory = $(Get-Location))

$addons = Get-ChildItem -Path $directory -Directory
ForEach ($addon in $addons) {
    $functionFiles = Get-ChildItem -Path $directory\$addon -File -Filter "fnc_*.sqf"
    $functions = $functionFiles -replace 'fnc_', '' -replace '.sqf', ''

    If ( $functions.Count -eq 0 )
    {
        Write-Output "No function files in $directory\$addon"
        continue
    }

    $cfgFunctions = "class CfgFunctions {
    class ADDON {
        class COMPONENT {
"

    ForEach ($fnc in $functions)
    {
        $cfgFunctions += "            PATHTO_FNC($fnc);
"
    }

    $cfgFunctions += 
"        };
	};
};"
    $cfgFunctions | Out-File -FilePath $directory\$addon\CfgFunctions.hpp -Encoding ascii
}

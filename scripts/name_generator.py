import json

squads = []
companions = []


def as_string_function(type, entries):
    code = f"/// Convert a {type} guid into its (english) name\n"
    code += f"pub fn {type}_as_string(s: &str) -> Option<&'static str> {{\n"
    code += "    match s {\n"
    for [guid, name] in entries:
        if name == "": continue
        code += f"        \"{guid}\" => Some(\"{name}\"),\n"
    code += """        _ => {
            info!("Unknown party member found: {}", s);
            None
        }\n"""
    code += "    }\n}"  # TODO

    return code

# Link or copy this file from your local PF WotR installation folder
# eg. C:\GOG Galaxy\Games\Pathfinder Wrath of the Righteous\Bundles
with open('samples/cheatdata.json') as file:

    data = json.load(file)

    for entry in data['Entries']:
        name = entry['Name']
        type = entry['TypeFullName']
        guid = entry['Guid']

        if name.endswith('_Companion'):
            name = name[:-10]
            companions.append([guid, name])

        if name.startswith('AnimalCompanionUnit'):
            name = name[19:]
            companions.append([guid, name])

        if name == "AzataDragonUnit":
            companions.append([guid, "Aivu"])
        
        # AzataDragonUnit
        # Should I do all the BlueprintUnit ? It's ~4k entries at the time of writing.

        if name.startswith('Army') and type == 'Kingmaker.Blueprints.BlueprintUnit':
            name = name[4:]
            squads.append([guid, name])

with open('src/data/names.rs', 'w') as file:
    code = "use log::info;\n\n"
    code += as_string_function('squad', squads)
    code += "\n\n"
    code += as_string_function('companion', companions)
    code += "\n"

    file.write(code)

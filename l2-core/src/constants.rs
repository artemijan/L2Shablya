use anyhow::bail;

pub const PROTOCOL_REVISION: i32 = 0x0106;
/// # Errors
/// - when server id is not in the list
pub fn try_get_server_name_by_id(server_id: u8) -> anyhow::Result<String> {
    if (0..127).contains(&server_id) {
        return Ok(SERVER_NAMES[server_id as usize - 1].to_owned());
    }
    bail!("Server ID out of range: {}", server_id);
}
pub const SERVER_NAMES: &[&str; 127] = &[
    "Bartz",
    "Sieghardt",
    "Kain",
    "Lionna",
    "Erica",
    "Gustin",
    "Devianne",
    "Hindemith",
    "Teon (EURO)",
    "Franz (EURO)",
    "Luna (EURO)",
    "Sayha",
    "Aria",
    "Phoenix",
    "Chronos",
    "Naia",
    "Shilen",
    "Magmeld",
    "Bartz (GMT-5)",
    "Event Server",
    "Frikios",
    "Ophylia",
    "Shakdun",
    "Tarziph",
    "Aria",
    "Esenn",
    "Elcardia",
    "Yiana",
    "Seresin",
    "Tarkai",
    "Khadia",
    "Roien",
    "Kallint (Non-PvP)",
    "Baium",
    "Kamael",
    "Beleth",
    "Anakim",
    "Lilith",
    "Thifiel",
    "Lithra",
    "Lockirin",
    "Kakai",
    "Cadmus",
    "Athebaldt",
    "Blackbird",
    "Ramsheart",
    "Esthus",
    "Vasper",
    "Lancer",
    "Ashton",
    "Waytrel",
    "Waltner",
    "Tahnford",
    "Hunter",
    "Dewell",
    "Rodemaye",
    "Ken Rauhel",
    "Ken Abigail",
    "Ken Orwen",
    "Van Holter",
    "Desperion",
    "Einhovant",
    "Schuneimann",
    "Faris",
    "Tor",
    "Carneiar",
    "Dwyllios",
    "Baium",
    "Hallate",
    "Zaken",
    "Core",
    "72",
    "73",
    "74",
    "75",
    "76",
    "77",
    "78",
    "79",
    "80",
    "81",
    "82",
    "83",
    "84",
    "85",
    "86",
    "87",
    "88",
    "89",
    "90",
    "91",
    "92",
    "93",
    "94",
    "95",
    "96",
    "97",
    "98",
    "99",
    "100",
    "101",
    "102",
    "103",
    "104",
    "105",
    "106",
    "107",
    "108",
    "109",
    "110",
    "111",
    "112",
    "113",
    "114",
    "115",
    "116",
    "117",
    "118",
    "119",
    "120",
    "121",
    "122",
    "123",
    "124",
    "125",
    "126",
    "???",
];

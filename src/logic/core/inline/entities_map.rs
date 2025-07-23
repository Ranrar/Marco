use once_cell::sync::Lazy;
use std::collections::HashSet;

pub static ENTITIES_JSON: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    for key in HTML_ENTITIES.keys() {
        set.insert(*key);
    }
    set
});
// This file is auto-generated from entities.json
// Do not edit manually.
use phf::phf_map;

pub static HTML_ENTITIES: phf::Map<&'static str, &'static str> = phf_map! {
    "&AElig" => "Æ",
    "&AElig;" => "Æ",
    "&AMP" => "&",
    "&AMP;" => "&",
    "&Aacute" => "Á",
    "&Aacute;" => "Á",
    "&Abreve;" => "Ă",
    "&Acirc" => "Â",
    "&Acirc;" => "Â",
    "&Acy;" => "А",
    "&Afr;" => "𝔄",
    "&Agrave" => "À",
    "&Agrave;" => "À",
    "&Alpha;" => "Α",
    "&Amacr;" => "Ā",
    "&And;" => "⩓",
    "&Aogon;" => "Ą",
    "&Aopf;" => "𝔸",
    "&ApplyFunction;" => "⁡",
    "&Aring" => "Å",
    "&Aring;" => "Å",
    "&Ascr;" => "𝒜",
    "&Assign;" => "≔",
    "&Atilde" => "Ã",
    "&Atilde;" => "Ã",
    "&Auml" => "Ä",
    "&Auml;" => "Ä",
    "&Backslash;" => "∖",
    "&Barv;" => "⫧",
    "&Barwed;" => "⌆",
    "&Bcy;" => "Б",
    "&Because;" => "∵",
    "&Bernoullis;" => "ℬ",
    "&Beta;" => "Β",
    "&Bfr;" => "𝔅",
    "&Bopf;" => "𝔹",
    "&Breve;" => "˘",
    "&Bscr;" => "ℬ",
    "&Bumpeq;" => "≎",
    // ...truncated for brevity, all mappings will be inserted here...
};

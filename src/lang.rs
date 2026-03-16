//! This file was generated and should not be edited directly.
//! The source code can be found at scripts/build.py

use std::{
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

// The idea is from https://github.com/johnstonskj/rust-codes/tree/main
pub trait Code: Clone + Debug + Display + FromStr + AsRef<str> + PartialEq + Eq + Hash {}

impl Code for Lang {}
impl Code for EditionSpec {}
impl Code for Edition {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Lang {
    /// Albanian
    Sq,
    /// Egyptian Arabic
    Arz,
    /// Gulf Arabic
    Afb,
    /// Arabic
    Ar,
    /// North Levantine Arabic
    Apc,
    /// South Levantine Arabic
    Ajp,
    /// Assyrian Neo-Aramaic
    Aii,
    /// Asturian
    Ast,
    /// Belarusian
    Be,
    /// Bengali
    Bn,
    /// Bulgarian
    Bg,
    /// Cantonese
    Yue,
    /// Catalan
    Ca,
    /// Cebuano
    Ceb,
    /// Chinese
    Zh,
    /// Czech
    Cs,
    /// Danish
    Da,
    /// Dutch
    Nl,
    /// English
    En,
    /// Middle English
    Enm,
    /// Old English
    Ang,
    /// Simple English
    Simple,
    /// Esperanto
    Eo,
    /// Estonian
    Et,
    /// Finnish
    Fi,
    /// French
    Fr,
    /// Galician
    Gl,
    /// Georgian
    Ka,
    /// German
    De,
    /// Greek
    El,
    /// Ancient Greek
    Grc,
    /// Hawaiian
    Haw,
    /// Hebrew
    He,
    /// Hindi
    Hi,
    /// Hungarian
    Hu,
    /// Icelandic
    Is,
    /// Indonesian
    Id,
    /// Irish
    Ga,
    /// Old Irish
    Sga,
    /// Italian
    It,
    /// Japanese
    Ja,
    /// Kannada
    Kn,
    /// Kazakh
    Kk,
    /// Khmer
    Km,
    /// Korean
    Ko,
    /// Kurdish
    Ku,
    /// Lao
    Lo,
    /// Latin
    La,
    /// Latvian
    Lv,
    /// Lithuanian
    Lt,
    /// Macedonian
    Mk,
    /// Malay
    Ms,
    /// Maltese
    Mt,
    /// Marathi
    Mr,
    /// Mongolian
    Mn,
    /// Norwegian
    No,
    /// Norwegian Bokmål
    Nb,
    /// Norwegian Nynorsk
    Nn,
    /// Persian
    Fa,
    /// Polish
    Pl,
    /// Portuguese
    Pt,
    /// Romanian
    Ro,
    /// Russian
    Ru,
    /// Sanskrit
    Sa,
    /// Serbo-Croatian
    Sh,
    /// Sicilian
    Scn,
    /// Slovene
    Sl,
    /// Spanish
    Es,
    /// Swedish
    Sv,
    /// Tagalog
    Tl,
    /// Telugu
    Te,
    /// Thai
    Th,
    /// Toki Pona
    Tok,
    /// Turkish
    Tr,
    /// Ukrainian
    Uk,
    /// Urdu
    Ur,
    /// Vietnamese
    Vi,
    /// Welsh
    Cy,
    /// Yiddish
    Yi,
}

impl From<Edition> for Lang {
    fn from(value: Edition) -> Self {
        match value {
            Edition::Zh => Self::Zh,
            Edition::Cs => Self::Cs,
            Edition::Nl => Self::Nl,
            Edition::En => Self::En,
            Edition::Simple => Self::Simple,
            Edition::Fr => Self::Fr,
            Edition::De => Self::De,
            Edition::El => Self::El,
            Edition::Id => Self::Id,
            Edition::It => Self::It,
            Edition::Ja => Self::Ja,
            Edition::Ko => Self::Ko,
            Edition::Ku => Self::Ku,
            Edition::Ms => Self::Ms,
            Edition::Pl => Self::Pl,
            Edition::Pt => Self::Pt,
            Edition::Ru => Self::Ru,
            Edition::Es => Self::Es,
            Edition::Th => Self::Th,
            Edition::Tr => Self::Tr,
            Edition::Vi => Self::Vi,
        }
    }
}

impl Lang {
    pub const fn help_isos() -> &'static str {
        "Supported isos: sq | arz | afb | ar | apc | ajp | aii | ast | be | bn | bg | yue | ca | ceb | zh | cs | da | nl | en | enm | ang | simple | eo | et | fi | fr | gl | ka | de | el | grc | haw | he | hi | hu | is | id | ga | sga | it | ja | kn | kk | km | ko | ku | lo | la | lv | lt | mk | ms | mt | mr | mn | no | nb | nn | fa | pl | pt | ro | ru | sa | sh | scn | sl | es | sv | tl | te | th | tok | tr | uk | ur | vi | cy | yi"
    }

    pub const fn help_isos_coloured() -> &'static str {
        "Supported isos: sq | arz | afb | ar | apc | ajp | aii | ast | be | bn | bg | yue | ca | ceb | [32mzh[0m | [32mcs[0m | da | [32mnl[0m | [32men[0m | enm | ang | [32msimple[0m | eo | et | fi | [32mfr[0m | gl | ka | [32mde[0m | [32mel[0m | grc | haw | he | hi | hu | is | [32mid[0m | ga | sga | [32mit[0m | [32mja[0m | kn | kk | km | [32mko[0m | [32mku[0m | lo | la | lv | lt | mk | [32mms[0m | mt | mr | mn | no | nb | nn | fa | [32mpl[0m | [32mpt[0m | ro | [32mru[0m | sa | sh | scn | sl | [32mes[0m | sv | tl | te | [32mth[0m | tok | [32mtr[0m | uk | ur | [32mvi[0m | cy | yi"
    }

    pub const fn help_editions() -> &'static str {
        "Supported editions: zh | cs | nl | en | simple | fr | de | el | id | it | ja | ko | ku | ms | pl | pt | ru | es | th | tr | vi"
    }

    pub const fn long(&self) -> &'static str {
        match self {
            Self::Sq => "Albanian",
            Self::Arz => "Egyptian Arabic",
            Self::Afb => "Gulf Arabic",
            Self::Ar => "Arabic",
            Self::Apc => "North Levantine Arabic",
            Self::Ajp => "South Levantine Arabic",
            Self::Aii => "Assyrian Neo-Aramaic",
            Self::Ast => "Asturian",
            Self::Be => "Belarusian",
            Self::Bn => "Bengali",
            Self::Bg => "Bulgarian",
            Self::Yue => "Cantonese",
            Self::Ca => "Catalan",
            Self::Ceb => "Cebuano",
            Self::Zh => "Chinese",
            Self::Cs => "Czech",
            Self::Da => "Danish",
            Self::Nl => "Dutch",
            Self::En => "English",
            Self::Enm => "Middle English",
            Self::Ang => "Old English",
            Self::Simple => "Simple English",
            Self::Eo => "Esperanto",
            Self::Et => "Estonian",
            Self::Fi => "Finnish",
            Self::Fr => "French",
            Self::Gl => "Galician",
            Self::Ka => "Georgian",
            Self::De => "German",
            Self::El => "Greek",
            Self::Grc => "Ancient Greek",
            Self::Haw => "Hawaiian",
            Self::He => "Hebrew",
            Self::Hi => "Hindi",
            Self::Hu => "Hungarian",
            Self::Is => "Icelandic",
            Self::Id => "Indonesian",
            Self::Ga => "Irish",
            Self::Sga => "Old Irish",
            Self::It => "Italian",
            Self::Ja => "Japanese",
            Self::Kn => "Kannada",
            Self::Kk => "Kazakh",
            Self::Km => "Khmer",
            Self::Ko => "Korean",
            Self::Ku => "Kurdish",
            Self::Lo => "Lao",
            Self::La => "Latin",
            Self::Lv => "Latvian",
            Self::Lt => "Lithuanian",
            Self::Mk => "Macedonian",
            Self::Ms => "Malay",
            Self::Mt => "Maltese",
            Self::Mr => "Marathi",
            Self::Mn => "Mongolian",
            Self::No => "Norwegian",
            Self::Nb => "Norwegian Bokmål",
            Self::Nn => "Norwegian Nynorsk",
            Self::Fa => "Persian",
            Self::Pl => "Polish",
            Self::Pt => "Portuguese",
            Self::Ro => "Romanian",
            Self::Ru => "Russian",
            Self::Sa => "Sanskrit",
            Self::Sh => "Serbo-Croatian",
            Self::Scn => "Sicilian",
            Self::Sl => "Slovene",
            Self::Es => "Spanish",
            Self::Sv => "Swedish",
            Self::Tl => "Tagalog",
            Self::Te => "Telugu",
            Self::Th => "Thai",
            Self::Tok => "Toki Pona",
            Self::Tr => "Turkish",
            Self::Uk => "Ukrainian",
            Self::Ur => "Urdu",
            Self::Vi => "Vietnamese",
            Self::Cy => "Welsh",
            Self::Yi => "Yiddish",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Sq,
            Self::Arz,
            Self::Afb,
            Self::Ar,
            Self::Apc,
            Self::Ajp,
            Self::Aii,
            Self::Ast,
            Self::Be,
            Self::Bn,
            Self::Bg,
            Self::Yue,
            Self::Ca,
            Self::Ceb,
            Self::Zh,
            Self::Cs,
            Self::Da,
            Self::Nl,
            Self::En,
            Self::Enm,
            Self::Ang,
            Self::Simple,
            Self::Eo,
            Self::Et,
            Self::Fi,
            Self::Fr,
            Self::Gl,
            Self::Ka,
            Self::De,
            Self::El,
            Self::Grc,
            Self::Haw,
            Self::He,
            Self::Hi,
            Self::Hu,
            Self::Is,
            Self::Id,
            Self::Ga,
            Self::Sga,
            Self::It,
            Self::Ja,
            Self::Kn,
            Self::Kk,
            Self::Km,
            Self::Ko,
            Self::Ku,
            Self::Lo,
            Self::La,
            Self::Lv,
            Self::Lt,
            Self::Mk,
            Self::Ms,
            Self::Mt,
            Self::Mr,
            Self::Mn,
            Self::No,
            Self::Nb,
            Self::Nn,
            Self::Fa,
            Self::Pl,
            Self::Pt,
            Self::Ro,
            Self::Ru,
            Self::Sa,
            Self::Sh,
            Self::Scn,
            Self::Sl,
            Self::Es,
            Self::Sv,
            Self::Tl,
            Self::Te,
            Self::Th,
            Self::Tok,
            Self::Tr,
            Self::Uk,
            Self::Ur,
            Self::Vi,
            Self::Cy,
            Self::Yi,
        ]
    }
}

impl TryInto<Edition> for Lang {
    type Error = &'static str;

    fn try_into(self) -> Result<Edition, Self::Error> {
        match self {
            Self::Zh => Ok(Edition::Zh),
            Self::Cs => Ok(Edition::Cs),
            Self::Nl => Ok(Edition::Nl),
            Self::En => Ok(Edition::En),
            Self::Simple => Ok(Edition::Simple),
            Self::Fr => Ok(Edition::Fr),
            Self::De => Ok(Edition::De),
            Self::El => Ok(Edition::El),
            Self::Id => Ok(Edition::Id),
            Self::It => Ok(Edition::It),
            Self::Ja => Ok(Edition::Ja),
            Self::Ko => Ok(Edition::Ko),
            Self::Ku => Ok(Edition::Ku),
            Self::Ms => Ok(Edition::Ms),
            Self::Pl => Ok(Edition::Pl),
            Self::Pt => Ok(Edition::Pt),
            Self::Ru => Ok(Edition::Ru),
            Self::Es => Ok(Edition::Es),
            Self::Th => Ok(Edition::Th),
            Self::Tr => Ok(Edition::Tr),
            Self::Vi => Ok(Edition::Vi),
            _ => Err("language has no edition"),
        }
    }
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sq" => Ok(Self::Sq),
            "arz" => Ok(Self::Arz),
            "afb" => Ok(Self::Afb),
            "ar" => Ok(Self::Ar),
            "apc" => Ok(Self::Apc),
            "ajp" => Ok(Self::Ajp),
            "aii" => Ok(Self::Aii),
            "ast" => Ok(Self::Ast),
            "be" => Ok(Self::Be),
            "bn" => Ok(Self::Bn),
            "bg" => Ok(Self::Bg),
            "yue" => Ok(Self::Yue),
            "ca" => Ok(Self::Ca),
            "ceb" => Ok(Self::Ceb),
            "zh" => Ok(Self::Zh),
            "cs" => Ok(Self::Cs),
            "da" => Ok(Self::Da),
            "nl" => Ok(Self::Nl),
            "en" => Ok(Self::En),
            "enm" => Ok(Self::Enm),
            "ang" => Ok(Self::Ang),
            "simple" => Ok(Self::Simple),
            "eo" => Ok(Self::Eo),
            "et" => Ok(Self::Et),
            "fi" => Ok(Self::Fi),
            "fr" => Ok(Self::Fr),
            "gl" => Ok(Self::Gl),
            "ka" => Ok(Self::Ka),
            "de" => Ok(Self::De),
            "el" => Ok(Self::El),
            "grc" => Ok(Self::Grc),
            "haw" => Ok(Self::Haw),
            "he" => Ok(Self::He),
            "hi" => Ok(Self::Hi),
            "hu" => Ok(Self::Hu),
            "is" => Ok(Self::Is),
            "id" => Ok(Self::Id),
            "ga" => Ok(Self::Ga),
            "sga" => Ok(Self::Sga),
            "it" => Ok(Self::It),
            "ja" => Ok(Self::Ja),
            "kn" => Ok(Self::Kn),
            "kk" => Ok(Self::Kk),
            "km" => Ok(Self::Km),
            "ko" => Ok(Self::Ko),
            "ku" => Ok(Self::Ku),
            "lo" => Ok(Self::Lo),
            "la" => Ok(Self::La),
            "lv" => Ok(Self::Lv),
            "lt" => Ok(Self::Lt),
            "mk" => Ok(Self::Mk),
            "ms" => Ok(Self::Ms),
            "mt" => Ok(Self::Mt),
            "mr" => Ok(Self::Mr),
            "mn" => Ok(Self::Mn),
            "no" => Ok(Self::No),
            "nb" => Ok(Self::Nb),
            "nn" => Ok(Self::Nn),
            "fa" => Ok(Self::Fa),
            "pl" => Ok(Self::Pl),
            "pt" => Ok(Self::Pt),
            "ro" => Ok(Self::Ro),
            "ru" => Ok(Self::Ru),
            "sa" => Ok(Self::Sa),
            "sh" => Ok(Self::Sh),
            "scn" => Ok(Self::Scn),
            "sl" => Ok(Self::Sl),
            "es" => Ok(Self::Es),
            "sv" => Ok(Self::Sv),
            "tl" => Ok(Self::Tl),
            "te" => Ok(Self::Te),
            "th" => Ok(Self::Th),
            "tok" => Ok(Self::Tok),
            "tr" => Ok(Self::Tr),
            "uk" => Ok(Self::Uk),
            "ur" => Ok(Self::Ur),
            "vi" => Ok(Self::Vi),
            "cy" => Ok(Self::Cy),
            "yi" => Ok(Self::Yi),
            _ => Err(format!("unsupported iso code '{s}'\n{}", Self::help_isos())),
        }
    }
}

impl AsRef<str> for Lang {
    fn as_ref(&self) -> &str {
        match self {
            Self::Sq => "sq",
            Self::Arz => "arz",
            Self::Afb => "afb",
            Self::Ar => "ar",
            Self::Apc => "apc",
            Self::Ajp => "ajp",
            Self::Aii => "aii",
            Self::Ast => "ast",
            Self::Be => "be",
            Self::Bn => "bn",
            Self::Bg => "bg",
            Self::Yue => "yue",
            Self::Ca => "ca",
            Self::Ceb => "ceb",
            Self::Zh => "zh",
            Self::Cs => "cs",
            Self::Da => "da",
            Self::Nl => "nl",
            Self::En => "en",
            Self::Enm => "enm",
            Self::Ang => "ang",
            Self::Simple => "simple",
            Self::Eo => "eo",
            Self::Et => "et",
            Self::Fi => "fi",
            Self::Fr => "fr",
            Self::Gl => "gl",
            Self::Ka => "ka",
            Self::De => "de",
            Self::El => "el",
            Self::Grc => "grc",
            Self::Haw => "haw",
            Self::He => "he",
            Self::Hi => "hi",
            Self::Hu => "hu",
            Self::Is => "is",
            Self::Id => "id",
            Self::Ga => "ga",
            Self::Sga => "sga",
            Self::It => "it",
            Self::Ja => "ja",
            Self::Kn => "kn",
            Self::Kk => "kk",
            Self::Km => "km",
            Self::Ko => "ko",
            Self::Ku => "ku",
            Self::Lo => "lo",
            Self::La => "la",
            Self::Lv => "lv",
            Self::Lt => "lt",
            Self::Mk => "mk",
            Self::Ms => "ms",
            Self::Mt => "mt",
            Self::Mr => "mr",
            Self::Mn => "mn",
            Self::No => "no",
            Self::Nb => "nb",
            Self::Nn => "nn",
            Self::Fa => "fa",
            Self::Pl => "pl",
            Self::Pt => "pt",
            Self::Ro => "ro",
            Self::Ru => "ru",
            Self::Sa => "sa",
            Self::Sh => "sh",
            Self::Scn => "scn",
            Self::Sl => "sl",
            Self::Es => "es",
            Self::Sv => "sv",
            Self::Tl => "tl",
            Self::Te => "te",
            Self::Th => "th",
            Self::Tok => "tok",
            Self::Tr => "tr",
            Self::Uk => "uk",
            Self::Ur => "ur",
            Self::Vi => "vi",
            Self::Cy => "cy",
            Self::Yi => "yi",
        }
    }
}

impl Lang {
    pub fn iso(&self) -> &str {
        match self {
            Self::Simple => "en",
            _ => self.as_ref(),
        }
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EditionSpec {
    /// All editions
    All,
    /// An `Edition`
    One(Edition),
}

impl EditionSpec {
    pub fn variants(&self) -> Vec<Edition> {
        match self {
            Self::All => Edition::all(),
            Self::One(lang) => vec![*lang],
        }
    }
}

impl From<Edition> for EditionSpec {
    fn from(val: Edition) -> Self {
        Self::One(val)
    }
}

impl TryInto<Edition> for EditionSpec {
    type Error = &'static str;

    fn try_into(self) -> Result<Edition, Self::Error> {
        match self {
            Self::All => Err("cannot convert from All"),
            Self::One(lang) => Ok(lang),
        }
    }
}

impl FromStr for EditionSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            other => Ok(Self::One(Edition::from_str(other)?)),
        }
    }
}

impl AsRef<str> for EditionSpec {
    fn as_ref(&self) -> &str {
        match self {
            Self::All => "all",
            Self::One(lang) => lang.as_ref(),
        }
    }
}

impl Display for EditionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Edition {
    /// Chinese
    Zh,
    /// Czech
    Cs,
    /// Dutch
    Nl,
    /// English
    En,
    /// Simple English
    Simple,
    /// French
    Fr,
    /// German
    De,
    /// Greek
    El,
    /// Indonesian
    Id,
    /// Italian
    It,
    /// Japanese
    Ja,
    /// Korean
    Ko,
    /// Kurdish
    Ku,
    /// Malay
    Ms,
    /// Polish
    Pl,
    /// Portuguese
    Pt,
    /// Russian
    Ru,
    /// Spanish
    Es,
    /// Thai
    Th,
    /// Turkish
    Tr,
    /// Vietnamese
    Vi,
}

impl Edition {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Zh,
            Self::Cs,
            Self::Nl,
            Self::En,
            Self::Simple,
            Self::Fr,
            Self::De,
            Self::El,
            Self::Id,
            Self::It,
            Self::Ja,
            Self::Ko,
            Self::Ku,
            Self::Ms,
            Self::Pl,
            Self::Pt,
            Self::Ru,
            Self::Es,
            Self::Th,
            Self::Tr,
            Self::Vi,
        ]
    }
}

impl FromStr for Edition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zh" => Ok(Self::Zh),
            "cs" => Ok(Self::Cs),
            "nl" => Ok(Self::Nl),
            "en" => Ok(Self::En),
            "simple" => Ok(Self::Simple),
            "fr" => Ok(Self::Fr),
            "de" => Ok(Self::De),
            "el" => Ok(Self::El),
            "id" => Ok(Self::Id),
            "it" => Ok(Self::It),
            "ja" => Ok(Self::Ja),
            "ko" => Ok(Self::Ko),
            "ku" => Ok(Self::Ku),
            "ms" => Ok(Self::Ms),
            "pl" => Ok(Self::Pl),
            "pt" => Ok(Self::Pt),
            "ru" => Ok(Self::Ru),
            "es" => Ok(Self::Es),
            "th" => Ok(Self::Th),
            "tr" => Ok(Self::Tr),
            "vi" => Ok(Self::Vi),
            _ => Err(format!("invalid edition '{s}'")),
        }
    }
}

impl AsRef<str> for Edition {
    fn as_ref(&self) -> &str {
        match self {
            Self::Zh => "zh",
            Self::Cs => "cs",
            Self::Nl => "nl",
            Self::En => "en",
            Self::Simple => "simple",
            Self::Fr => "fr",
            Self::De => "de",
            Self::El => "el",
            Self::Id => "id",
            Self::It => "it",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Ku => "ku",
            Self::Ms => "ms",
            Self::Pl => "pl",
            Self::Pt => "pt",
            Self::Ru => "ru",
            Self::Es => "es",
            Self::Th => "th",
            Self::Tr => "tr",
            Self::Vi => "vi",
        }
    }
}

impl Edition {
    pub fn iso(&self) -> &str {
        match self {
            Self::Simple => "en",
            _ => self.as_ref(),
        }
    }
}

impl Display for Edition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

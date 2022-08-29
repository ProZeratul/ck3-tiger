use fnv::FnvHashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::block::{Block, DefinitionItem};
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log};
use crate::fileset::{FileEntry, FileHandler};
use crate::localization::Localization;
use crate::pdxfile::PdxFile;
use crate::provinces::ProvId;
use crate::token::Token;

#[derive(Clone, Debug, Default)]
pub struct Titles {
    titles: FnvHashMap<String, Rc<Title>>,
    baronies: FnvHashMap<ProvId, Rc<Title>>,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Tier {
    Barony,
    County,
    Duchy,
    Kingdom,
    Empire,
}

impl TryFrom<&Token> for Tier {
    type Error = std::fmt::Error;
    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        let s = value.as_str();
        if s.starts_with("b_") {
            Ok(Tier::Barony)
        } else if s.starts_with("c_") {
            Ok(Tier::County)
        } else if s.starts_with("d_") {
            Ok(Tier::Duchy)
        } else if s.starts_with("k_") {
            Ok(Tier::Kingdom)
        } else if s.starts_with("e_") {
            Ok(Tier::Empire)
        } else {
            Err(std::fmt::Error)
        }
    }
}

impl TryFrom<Token> for Tier {
    type Error = std::fmt::Error;
    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Tier::try_from(&value)
    }
}

impl Display for Tier {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            Tier::Barony => write!(f, "barony"),
            Tier::County => write!(f, "county"),
            Tier::Duchy => write!(f, "duchy"),
            Tier::Kingdom => write!(f, "kingdom"),
            Tier::Empire => write!(f, "empire"),
        }
    }
}

impl Titles {
    pub fn load_item(
        &mut self,
        key: Token,
        block: &Block,
        values: Vec<(Token, Token)>,
        capital_of: Option<Token>,
    ) {
        if let Some(other) = self.titles.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(&key, ErrorKey::Duplicate) {
                error(
                    &key,
                    ErrorKey::Duplicate,
                    "title redefines an existing title",
                );
                info(&other.key, ErrorKey::Duplicate, "the other title is here");
            }
        }
        Title::validate(block);
        let title = Rc::new(Title::new(
            key.clone(),
            block.clone(),
            values.clone(),
            capital_of,
        ));
        self.titles.insert(key.to_string(), title.clone());

        let parent_tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        if parent_tier == Tier::Barony {
            if let Some(provid) = block.get_field_integer("province") {
                self.baronies.insert(provid as ProvId, title);
            } else {
                error(&key, ErrorKey::Validation, "barony without province id");
            }
        }

        let mut capital = parent_tier == Tier::County;
        for (k, v) in block.iter_pure_definitions() {
            if let Ok(tier) = Tier::try_from(k) {
                if tier >= parent_tier {
                    let msg = format!("can't put a {} inside a {}", tier, parent_tier);
                    error(k, ErrorKey::Validation, &msg);
                }
                let capital_of = if capital { Some(key.clone()) } else { None };
                self.load_item(k.clone(), v, values.clone(), capital_of);
                capital = false;
            }
        }
        if capital {
            error(key, ErrorKey::Validation, "county with no baronies!");
        }
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for title in self.titles.values() {
            title.check_have_locas(locas);
        }
    }

    pub fn capital_of(&self, prov: ProvId) -> Option<&Token> {
        self.baronies.get(&prov).and_then(|b| b.capital_of.as_ref())
    }
}

impl FileHandler for Titles {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/landed_titles")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let block = match PdxFile::read(entry.path(), entry.kind(), fullpath) {
            Ok(block) => block,
            Err(e) => {
                error_info(
                    entry,
                    ErrorKey::ReadError,
                    "could not read file",
                    &format!("{:#}", e),
                );
                return;
            }
        };

        let mut defined_values: Vec<(Token, Token)> = Vec::new();

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Assignment(key, value) => {
                    if key.as_str().starts_with('@') {
                        defined_values.push((key.clone(), value.clone()));
                    } else {
                        error(key, ErrorKey::Validation, "unexpected assignment");
                    }
                }
                DefinitionItem::Definition(key, b) => {
                    if Tier::try_from(key).is_ok() {
                        self.load_item(key.clone(), b, defined_values.clone(), None);
                    } else {
                        warn(key, ErrorKey::Validation, "expected title");
                    }
                }
            }
        }
    }

    fn finalize(&mut self) {
        for title in self.titles.values() {
            if let Some(capital) = title.block.get_field_value("capital") {
                if self.titles.get(capital.as_str()).is_none() {
                    error(
                        capital,
                        ErrorKey::Validation,
                        "capital is not defined as a title",
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Title {
    key: Token,
    values: Vec<(Token, Token)>,
    block: Block,
    tier: Tier,
    capital_of: Option<Token>, // for baronies
}

impl Title {
    pub fn new(
        key: Token,
        block: Block,
        values: Vec<(Token, Token)>,
        capital_of: Option<Token>,
    ) -> Self {
        let tier = Tier::try_from(&key).unwrap(); // guaranteed by caller
        Self {
            key,
            values,
            block,
            tier,
            capital_of,
        }
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        locas.verify_have_key(self.key.as_str(), &self.key, "title");
        // TODO: figure out when to recommend adding _adj or _pre titles
        // The _adj key is optional
        // The _pre key is optional

        if let Some(names) = self.block.get_field_block("cultural_names") {
            for (_, t) in names.get_assignments() {
                locas.verify_have_key(t.as_str(), t, "cultural name");
                // The _adj key is optional
            }
        }
    }

    fn validate(_block: &Block) {
        // TODO
    }
}
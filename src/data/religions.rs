use fnv::FnvHashMap;
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, DefinitionItem};
use crate::data::localization::Localization;
use crate::errorkey::ErrorKey;
use crate::errors::{error, error_info, info, warn, will_log, LogPauseRaii};
use crate::fileset::{FileEntry, FileHandler, FileKind, Fileset};
use crate::pdxfile::PdxFile;
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug, Default)]
pub struct Religions {
    religions: FnvHashMap<String, Religion>,
    faiths: FnvHashMap<String, Faith>,
}

impl Religions {
    fn load_item(&mut self, key: &Token, block: &Block) {
        if let Some(other) = self.religions.get(key.as_str()) {
            if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                error(
                    key,
                    ErrorKey::Duplicate,
                    "religion redefines an existing religion",
                );
                info(
                    &other.key,
                    ErrorKey::Duplicate,
                    "the other religion is here",
                );
            }
        }
        self.religions
            .insert(key.to_string(), Religion::new(key.clone(), block.clone()));

        if let Some(faith_block) = block.get_field_block("faiths") {
            for (faith, b) in faith_block.iter_pure_definitions_warn() {
                if let Some(other) = self.faiths.get(faith.as_str()) {
                    if other.key.loc.kind >= key.loc.kind && will_log(key, ErrorKey::Duplicate) {
                        error(
                            key,
                            ErrorKey::Duplicate,
                            "faith redefines an existing faith",
                        );
                        info(&other.key, ErrorKey::Duplicate, "the other faith is here");
                    }
                }
                let pagan = block.get_field_bool("pagan_roots").unwrap_or(false);
                self.faiths.insert(
                    faith.to_string(),
                    Faith::new(faith.clone(), b.clone(), key.clone(), pagan),
                );
            }
        }
    }

    pub fn check_have_customs(&self) {
        for faith in self.faiths.values() {
            let religion = &self.religions[faith.religion.as_str()];
            faith.check_have_customs(religion);
        }
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        for religion in self.religions.values() {
            religion.check_have_locas(locas);
        }
        for faith in self.faiths.values() {
            faith.check_have_locas(locas);
        }
    }

    pub fn check_have_files(&self, files: &Fileset) {
        for religion in self.religions.values() {
            religion.check_have_files(files);
        }
        for faith in self.faiths.values() {
            faith.check_have_files(files);
        }
    }

    pub fn verify_faith_exists(&self, item: &Token) {
        if !self.faiths.contains_key(item.as_str()) {
            error(
                item,
                ErrorKey::MissingItem,
                "faith not defined in common/religion/religions",
            );
        }
    }

    pub fn is_modded_faith(&self, item: &Token) -> bool {
        if let Some(faith) = self.faiths.get(item.as_str()) {
            faith.key.loc.kind != FileKind::VanillaFile
        } else {
            false
        }
    }
}

impl FileHandler for Religions {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/religion/religions")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let _pause = LogPauseRaii::new(entry.kind() != FileKind::ModFile);

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

        for def in block.iter_definitions_warn() {
            match def {
                DefinitionItem::Assignment(key, _) => {
                    error(key, ErrorKey::Validation, "unknown assignment")
                }
                DefinitionItem::Keyword(key) => error_info(
                    key,
                    ErrorKey::Validation,
                    "unexpected token",
                    "Did you forget an = ?",
                ),
                DefinitionItem::Definition(key, b) => {
                    self.load_item(key, b);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Religion {
    key: Token,
    block: Block,
}

impl Religion {
    pub fn new(key: Token, block: Block) -> Self {
        Self::validate(&block);
        Self { key, block }
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.req_field_value("family");
        vd.req_field_values("doctrine");
        vd.opt_field_blocks("doctrine_selection_pair"); // TODO: validate
        vd.opt_field_value("doctrine_background_icon");
        vd.opt_field_value("piety_icon_group");
        vd.opt_field_value("graphical_faith");
        vd.opt_field_bool("pagan_roots");
        vd.opt_field_validated_block("traits", validate_traits);
        vd.opt_field_list("custom_faith_icons");
        vd.opt_field_list("reserved_male_names");
        vd.opt_field_list("reserved_female_names");
        vd.opt_field_validated_block("holy_order_names", validate_holy_order_names);
        vd.opt_field_list("holy_order_maa");
        // TODO: check that all keys are there.
        // Which needs checking up the chain faith, religion, family
        vd.opt_field_block("localization");
        vd.opt_field_blocks("faiths");
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        locas.verify_exists(self.key.as_str(), &self.key);
        let loca = format!("{}_adj", self.key);
        locas.verify_exists(&loca, &self.key);
        let loca = format!("{}_adherent", self.key);
        locas.verify_exists(&loca, &self.key);
        let loca = format!("{}_adherent_plural", self.key);
        locas.verify_exists(&loca, &self.key);
        let loca = format!("{}_desc", self.key);
        locas.verify_exists(&loca, &self.key);

        if let Some(holy) = self.block.get_field_block("holy_order_names") {
            for b in holy.get_sub_blocks() {
                if let Some(name) = b.get_field_value("name") {
                    locas.verify_exists(name.as_str(), name);
                }
            }
        }

        if let Some(b) = self.block.get_field_block("localization") {
            for (_, loca) in b.get_assignments() {
                locas.verify_exists(loca.as_str(), loca);
            }
            if let Some(list) = b.get_field_list("GoodGodNames") {
                for loca in list {
                    locas.verify_exists(loca.as_str(), &loca);
                }
            }
            if let Some(list) = b.get_field_list("EvilGodNames") {
                for loca in list {
                    locas.verify_exists(loca.as_str(), &loca);
                }
            }
        }
    }

    pub fn check_have_files(&self, files: &Fileset) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(icons) = self.block.get_field_list("custom_faith_icons") {
            for icon in &icons {
                let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
                files.verify_exists_implied(&pathname, icon);
            }
        }

        if let Some(icon) = self.block.get_field_value("doctrine_background_icon") {
            let pathname = format!("gfx/interface/icons/faith_doctrines/{}", icon);
            files.verify_exists_implied(&pathname, icon);
        }
    }
}

fn validate_traits(block: &Block) {
    let mut vd = Validator::new(block);
    // TODO: parse these. Can be single tokens ("wrathful") or assignments ("wrathful = 3")
    // or even wrathful = { modifier = modifier_key scale = 2 }
    vd.req_field("virtues");
    vd.req_field("sins");
    vd.warn_remaining();
}

fn validate_holy_order_names(_block: &Block) {
    // TODO
    // It's a list of sub-blocks, each one having a name key and optional coat_of_arms key
}

#[derive(Clone, Debug)]
pub struct Faith {
    key: Token,
    block: Block,
    religion: Token,
    pagan: bool,
}

impl Faith {
    pub fn new(key: Token, block: Block, religion: Token, pagan: bool) -> Self {
        Self::validate(&block);
        // TODO: verify that reform_icon is set if a pagan faith
        Self {
            key,
            block,
            religion,
            pagan,
        }
    }

    pub fn kind(&self) -> FileKind {
        self.key.loc.kind
    }

    pub fn validate(block: &Block) {
        let mut vd = Validator::new(block);

        vd.req_field_validated_block("color", validate_color);
        vd.opt_field_value("icon");
        vd.opt_field_value("reformed_icon");
        vd.opt_field_value("graphical_faith");
        vd.opt_field_value("piety_icon_group");
        vd.opt_field_value("doctrine_background_icon");
        vd.opt_field_value("religious_head");
        vd.req_field_values("holy_site");
        vd.req_field_values("doctrine");
        vd.opt_field_blocks("doctrine_selection_pair"); // TODO: validate
        vd.opt_field_list("reserved_male_names");
        vd.opt_field_list("reserved_female_names");
        vd.opt_field_block("localization");
        vd.opt_field_validated_block("holy_order_names", validate_holy_order_names);
        vd.opt_field_list("holy_order_maa"); // TODO: verify this is allowed
        vd.warn_remaining();
    }

    pub fn check_have_locas(&self, locas: &Localization) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        locas.verify_exists(self.key.as_str(), &self.key);
        let loca = format!("{}_adj", self.key);
        locas.verify_exists(&loca, &self.key);
        let loca = format!("{}_adherent", self.key);
        locas.verify_exists(&loca, &self.key);
        let loca = format!("{}_adherent_plural", self.key);
        locas.verify_exists(&loca, &self.key);

        if self.pagan {
            let loca = format!("{}_old", self.key);
            locas.verify_exists(&loca, &self.key);
            let loca = format!("{}_old_adj", self.key);
            locas.verify_exists(&loca, &self.key);
            let loca = format!("{}_old_adherent", self.key);
            locas.verify_exists(&loca, &self.key);
            let loca = format!("{}_old_adherent_plural", self.key);
            locas.verify_exists(&loca, &self.key);
        }

        if let Some(b) = self.block.get_field_block("localization") {
            for (_, loca) in b.get_assignments() {
                locas.verify_exists(loca.as_str(), loca);
            }
            if let Some(list) = b.get_field_list("GoodGodNames") {
                for loca in list {
                    locas.verify_exists(loca.as_str(), &loca);
                }
            }
            if let Some(list) = b.get_field_list("EvilGodNames") {
                for loca in list {
                    locas.verify_exists(loca.as_str(), &loca);
                }
            }
        }
    }

    pub fn check_have_files(&self, files: &Fileset) {
        let _pause = LogPauseRaii::new(self.key.loc.kind != FileKind::ModFile);

        if let Some(icon) = self.block.get_field_value("icon") {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
            files.verify_exists_implied(&pathname, icon);
        } else {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", self.key);
            files.verify_exists_implied(&pathname, &self.key);
        }

        if let Some(icon) = self.block.get_field_value("reformed_icon") {
            let pathname = format!("gfx/interface/icons/faith/{}.dds", icon);
            files.verify_exists_implied(&pathname, icon);
        }
    }

    pub fn check_have_customs(&self, religion: &Religion) {
        let self_block = self.block.get_field_block("localization");
        let religion_block = religion.block.get_field_block("localization");
        for s in CUSTOM_RELIGION_LOCAS {
            if let Some(block) = self_block {
                if block.get_key(s).is_some() {
                    continue;
                }
            }
            if let Some(block) = religion_block {
                if block.get_key(s).is_some() {
                    continue;
                }
            }
            let msg = format!("faith or religion missing localization for `{}`", s);
            warn(&self.key, ErrorKey::MissingLocalization, &msg);
        }
    }
}

// LAST UPDATED VERSION 1.6.2.2
const CUSTOM_RELIGION_LOCAS: &[&str] = &[
    "AltPriestTermPlural",
    "BishopFemale",
    "BishopFemalePlural",
    "BishopMale",
    "BishopMalePlural",
    "BishopNeuter",
    "BishopNeuterPlural",
    "CreatorHerHim",
    "CreatorHerHis",
    "CreatorName",
    "CreatorNamePossessive",
    "CreatorSheHe",
    "DeathDeityHerHis",
    "DeathDeityName",
    "DeathDeityNamePossessive",
    "DeathDeitySheHe",
    "DevilHerHis",
    "DevilHerselfHimself",
    "DevilName",
    "DevilNamePossessive",
    "DevilSheHe",
    "DevoteeFemale",
    "DevoteeFemalePlural",
    "DevoteeMale",
    "DevoteeMalePlural",
    "DevoteeNeuter",
    "DevoteeNeuterPlural",
    "DivineRealm",
    "DivineRealm2",
    "DivineRealm3",
    "EvilGodNames",
    "FateGodHerHim",
    "FateGodHerHis",
    "FateGodName",
    "FateGodNamePossessive",
    "FateGodSheHe",
    "FertilityGodHerHim",
    "FertilityGodHerHis",
    "FertilityGodName",
    "FertilityGodNamePossessive",
    "FertilityGodSheHe",
    "GHWName",
    "GHWNamePlural",
    "GoodGodNames",
    "HealthGodHerHim",
    "HealthGodHerHis",
    "HealthGodName",
    "HealthGodNamePossessive",
    "HealthGodSheHe",
    "HighGodHerHis",
    "HighGodHerselfHimself",
    "HighGodName",
    "HighGodName2",
    "HighGodNameAlternate",
    "HighGodNameAlternatePossessive",
    "HighGodNamePossessive",
    "HighGodNameSheHe",
    "HouseOfWorship",
    "HouseOfWorship2",
    "HouseOfWorship3",
    "HouseOfWorshipPlural",
    "HouseholdGodHerHim",
    "HouseholdGodHerHis",
    "HouseholdGodName",
    "HouseholdGodNamePossessive",
    "HouseholdGodSheHe",
    "KnowledgeGodHerHim",
    "KnowledgeGodHerHis",
    "KnowledgeGodName",
    "KnowledgeGodNamePossessive",
    "KnowledgeGodSheHe",
    "NegativeAfterLife",
    "NegativeAfterLife2",
    "NegativeAfterLife3",
    "NightGodHerHim",
    "NightGodHerHis",
    "NightGodName",
    "NightGodNamePossessive",
    "NightGodSheHe",
    "PantheonTerm",
    "PantheonTerm2",
    "PantheonTerm3",
    "PantheonTermHasHave",
    "PositiveAfterLife",
    "PositiveAfterLife2",
    "PositiveAfterLife3",
    "PriestFemale",
    "PriestFemalePlural",
    "PriestMale",
    "PriestMalePlural",
    "PriestNeuter",
    "PriestNeuterPlural",
    "ReligiousHeadName",
    "ReligiousHeadTitleName",
    "ReligiousSymbol",
    "ReligiousSymbol2",
    "ReligiousSymbol3",
    "ReligiousText",
    "ReligiousText2",
    "ReligiousText3",
    "TricksterGodHerHim",
    "TricksterGodHerHis",
    "TricksterGodName",
    "TricksterGodNamePossessive",
    "TricksterGodSheHe",
    "WarGodHerHim",
    "WarGodHerHis",
    "WarGodName",
    "WarGodNamePossessive",
    "WarGodSheHe",
    "WaterGodHerHim",
    "WaterGodHerHis",
    "WaterGodName",
    "WaterGodNamePossessive",
    "WaterGodSheHe",
    "WealthGodHerHim",
    "WealthGodHerHis",
    "WealthGodName",
    "WealthGodNamePossessive",
    "WealthGodSheHe",
    "WitchGodHerHim",
    "WitchGodHerHis",
    "WitchGodMistressMaster",
    "WitchGodMotherFather",
    "WitchGodName",
    "WitchGodNamePossessive",
    "WitchGodSheHe",
];
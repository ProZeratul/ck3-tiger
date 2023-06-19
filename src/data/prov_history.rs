use fnv::{FnvHashMap, FnvHashSet};
use std::path::{Path, PathBuf};

use crate::block::validator::Validator;
use crate::block::{Block, Date, BV};
use crate::data::provinces::ProvId;
use crate::data::religions::Religions;
use crate::data::titles::Titles;
use crate::effect::validate_normal_effect;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::dup_error;
use crate::item::Item;
use crate::pdxfile::PdxFile;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug, Default)]
pub struct ProvinceHistories {
    provinces: FnvHashMap<ProvId, ProvinceHistory>,
}

impl ProvinceHistories {
    fn load_history(&mut self, id: ProvId, key: &Token, b: &Block) {
        if let Some(province) = self.provinces.get_mut(&id) {
            // Multiple entries are valid but could easily be a mistake.
            if province.key.loc.kind >= key.loc.kind {
                dup_error(key, &province.key, "province");
            }
            province.block.append(&mut b.clone());
        } else {
            self.provinces
                .insert(id, ProvinceHistory::new(key.clone(), b.clone()));
        }
    }

    pub fn validate(&self, data: &Everything) {
        let mut vec = self.provinces.values().collect::<Vec<&ProvinceHistory>>();
        vec.sort_unstable_by_key(|item| &item.key.loc);
        for item in vec {
            item.validate(data);
        }
    }

    pub fn check_pod_faiths(&self, religions: &Religions, titles: &Titles) {
        let mut warned = FnvHashSet::default();

        for bookmark in [
            Date::new(1230, 1, 4),
            Date::new(1230, 1, 5),
            Date::new(1230, 1, 6),
            Date::new(1375, 7, 5),
            Date::new(1510, 1, 3),
        ] {
            for (provid, provhist) in &self.provinces {
                if let Some(capital) = titles.capital_of(*provid) {
                    let religion = provhist.block.get_field_at_date("religion", bookmark);
                    if let Some(religion) = religion.and_then(BV::into_value) {
                        if !religions.is_modded_faith(&religion) && !warned.contains(provid) {
                            let msg = format!(
                                "Vanilla or unknown religion in prov {} (county {}) at {}",
                                provhist.key, capital, bookmark
                            );
                            warn(religion, ErrorKey::PrincesOfDarkness, &msg);
                            warned.insert(provid);
                        }
                    } else {
                        warn(&provhist.key, ErrorKey::PrincesOfDarkness, "no religion");
                    }
                }
            }
        }
    }
}

impl FileHandler for ProvinceHistories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("history/provinces")
    }

    fn handle_file(&mut self, entry: &FileEntry, fullpath: &Path) {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return;
        }

        let Some(block) = PdxFile::read_cp1252(entry, fullpath) else { return };
        for (key, b) in block.iter_pure_definitions_warn() {
            if let Ok(id) = key.as_str().parse() {
                self.load_history(id, key, b);
            } else {
                warn(
                    key,
                    ErrorKey::Validation,
                    "unexpected key, expected only province ids",
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProvinceHistory {
    key: Token,
    block: Block,
}

impl ProvinceHistory {
    fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    fn validate_common(vd: &mut Validator, data: &Everything) {
        vd.field_item("culture", Item::Culture);
        vd.field_item("religion", Item::Faith);
        vd.field_item("faith", Item::Faith);
        if let Some(token) = vd.field_value("holding") {
            if !token.is("auto") && !token.is("none") {
                data.verify_exists(Item::Holding, token);
            }
        }
        vd.field_list_items("buildings", Item::Building);
        vd.field_items("special_building_slot", Item::SpecialBuilding);
        vd.field_items("special_building", Item::SpecialBuilding);
        // TODO: check if province is duchy capital
        // TODO: check if building is duchy capital building
        vd.field_item("duchy_capital_building", Item::Building);

        vd.field_validated_block_rooted("effect", Scopes::Province, |block, data, sc| {
            validate_normal_effect(block, data, sc, Tooltipped::No);
        });
    }

    fn validate_history(_date: Date, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        Self::validate_common(&mut vd, data);
    }

    fn validate(&self, data: &Everything) {
        // TODO: verify that all county-capital provinces have a culture and religion
        // This needs province mappings to be loaded too
        let mut vd = Validator::new(&self.block, data);
        Self::validate_common(&mut vd, data);
        vd.field_value("terrain"); // TODO: this does not seem to be an Item::Terrain
        vd.validate_history_blocks(Self::validate_history);
    }
}

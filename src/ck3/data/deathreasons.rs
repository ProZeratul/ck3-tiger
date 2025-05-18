use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DeathReason {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DeathReason, DeathReason::add)
}

impl DeathReason {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DeathReason, key, block, Box::new(Self {}));
    }
}

impl DbKind for DeathReason {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(epidemic) = block.get_field_value("epidemic") {
            db.add_flag(Item::EpidemicDeathReason, epidemic.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_bool("public_knowledge");
        vd.advice_field("natural", "removed in 1.12");
        vd.field_integer("priority");
        vd.field_bool("default");

        data.verify_exists(Item::Localization, key);
        if !key.as_str().ends_with("_killer") && !block.field_value_is("natural", "yes") {
            // TODO: can we narrow down which death reasons need a _killer version?
            let loca = format!("{key}_killer");
            data.mark_used(Item::Localization, &loca);
        }

        vd.field_icon("icon", "NGameIcons|DEATH_REASON_ICON_PATH", "");
        vd.field_trigger("natural_death_trigger", Tooltipped::No, &mut sc);
        vd.field_item("use_equipped_artifact_in_slot", Item::ArtifactSlot);
        vd.field_item("epidemic", Item::EpidemicType);
    }
}

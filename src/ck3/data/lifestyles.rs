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
pub struct Lifestyle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Lifestyle, Lifestyle::add)
}

impl Lifestyle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Lifestyle, key, block, Box::new(Self {}));
    }
}

impl DbKind for Lifestyle {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_highlight_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let modif = format!("monthly_{key}_xp_gain_mult");
        data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        vd.field_trigger("is_highlighted", Tooltipped::No, &mut sc);
        vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc);
        vd.field_trigger("is_valid_showing_failures_only", Tooltipped::FailuresOnly, &mut sc);

        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|LIFESTYLE_ICON_PATH", icon, ".dds");
        data.verify_icon("NGameIcons|LIFESTYLE_BACKGROUND_PATH", icon, ".dds");

        vd.field_numeric("xp_per_level");
        vd.field_numeric("base_xp_gain");
    }
}

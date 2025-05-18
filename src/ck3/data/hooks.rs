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
pub struct Hook {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Hook, Hook::add)
}

impl Hook {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Hook, key, block, Box::new(Self {}));
    }
}

impl DbKind for Hook {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("target", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);

        vd.field_integer("expiration_days");
        vd.field_bool("strong");
        vd.field_bool("perpetual");
        vd.field_bool("requires_secret");

        vd.field_effect("on_used", Tooltipped::No, &mut sc);
    }
}

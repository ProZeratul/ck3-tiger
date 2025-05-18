use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Suggestion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Suggestion, Suggestion::add)
}

impl Suggestion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Suggestion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Suggestion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_label");
        data.mark_used(Item::Localization, &loca); // TODO: when is _label needed?
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_click");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let mut sc = ScopeContext::new(Scopes::Character, key);
        // TODO: "only interface effects are allowed"
        vd.field_effect_rooted("check_create_suggestion", Tooltipped::No, Scopes::Character);

        // TODO: "only interface effects are allowed"
        vd.field_effect_builder("effect", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            // TODO: The scope context will contain all scopes passed in the try_create_suggestion call
            sc.set_strict_scopes(false);
            sc
        });

        vd.field_item("soundeffect", Item::Sound);

        vd.field_validated_block_sc("weight", &mut sc, validate_modifiers_with_base);

        // TODO: The scope context will contain all scopes passed in the try_create_suggestion call
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.set_strict_scopes(false);
        vd.field_validated_block_sc("score", &mut sc, validate_modifiers_with_base);

        vd.field_trigger_builder("is_valid", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.set_strict_scopes(false); // same as above
            sc
        });
    }
}

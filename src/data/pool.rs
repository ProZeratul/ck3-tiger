use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct PoolSelector {}

impl PoolSelector {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoolSelector, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoolSelector {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
        sc.define_name("base", key.clone(), Scopes::Character);
        if key.is("auto_generated_baron") {
            sc.define_name("province", key.clone(), Scopes::Province);
        }

        vd.field_validated_block("valid_character", |block, data| {
            validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block_sc("character_score", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("config", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("background", Item::CharacterBackground);
            vd.field_list_integers_exactly("age", 2);
        });

        vd.field_integer("selection_count");

        // undocumented

        vd.field_value("pool");
        vd.field_choice("gender", &["commander", "clergy", "gender_law", "random"]);
        vd.field_value("culture");
        vd.field_bool("court");
    }
}
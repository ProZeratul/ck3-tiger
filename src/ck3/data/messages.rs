use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Message {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Message, Message::add)
}

impl Message {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Message, key, block, Box::new(Self {}));
    }
}

impl DbKind for Message {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("display", &["feed", "toast", "popup"]);
        vd.field_item("message_filter_type", Item::MessageFilterType);
        vd.field_item("title", Item::Localization); // docs say text
        vd.field_item("desc", Item::Localization);
        vd.field_item("tooltip", Item::Localization);
        vd.field_item("soundeffect", Item::Sound);
        vd.field_icon("icon", "NGameIcons|MESSAGE_ICON_PATH", ".dds");
        vd.field_choice("style", &["good", "bad", "neutral"]);

        vd.advice_field("flags", "removed in 1.12");
        vd.multi_field_value("flag");
        vd.field_bool("combine_into_one");
    }
}

#[derive(Clone, Debug)]
pub struct MessageFilterType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::MessageFilterType, MessageFilterType::add)
}

impl MessageFilterType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MessageFilterType, key, block, Box::new(Self {}));
    }
}

impl DbKind for MessageFilterType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("message_filter_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice("display", &["feed", "toast", "hidden", "popup"]);
        vd.field_bool("always_show");
        vd.field_bool("auto_pause");
        vd.field_integer("sort_order");
        vd.field_item("group", Item::MessageGroupType);
    }
}

#[derive(Clone, Debug)]
pub struct MessageGroupType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::MessageGroupType, MessageGroupType::add)
}

impl MessageGroupType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MessageGroupType, key, block, Box::new(Self {}));
    }
}

impl DbKind for MessageGroupType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("message_group_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_integer("sort_order");
    }
}

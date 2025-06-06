use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validate::validate_color;

#[derive(Clone, Debug)]
pub struct NamedColor {}

inventory::submit! {
    ItemLoader::Full(GameFlags::all(), Item::NamedColor, PdxEncoding::Utf8OptionalBom, ".txt", LoadAsFile::No, Recursive::Maybe, NamedColor::add)
}

impl NamedColor {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("colors") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::NamedColor, key, block, Box::new(Self {}));
            }
        } else {
            warn(ErrorKey::ParseError).msg("unexpected field").loc(key).push();
        }
    }
}

impl DbKind for NamedColor {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        validate_color(block, data);
    }
}

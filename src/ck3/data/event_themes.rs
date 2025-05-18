use std::sync::RwLock;

use crate::block::Block;
use crate::ck3::validate::{
    validate_theme_background, validate_theme_effect_2d, validate_theme_header_background,
    validate_theme_icon, validate_theme_sound, validate_theme_transition,
};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::Severity;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Debug)]
pub struct EventTheme {
    validated_scopes: RwLock<Scopes>,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::EventTheme, EventTheme::add)
}

impl EventTheme {
    pub fn new() -> Self {
        let validated_scopes = RwLock::new(Scopes::empty());
        Self { validated_scopes }
    }

    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTheme, key, block, Box::new(Self::new()));
    }
}

impl DbKind for EventTheme {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {}

    /// Themes are unusual in that they are validated through the events that use them.
    /// This means that unused themes are not validated, which is ok.
    /// The purpose is to allow the triggers to be validated in the context of the scope
    /// of the event that uses them.
    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _caller: &Token,
        _caller_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        // Check if the passed-in scope type has already been validated for
        if self.validated_scopes.read().unwrap().contains(sc.scopes()) {
            return;
        }
        *self.validated_scopes.write().unwrap() |= sc.scopes();

        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);

        vd.req_field("background");
        vd.req_field("icon");
        vd.req_field("sound");

        vd.multi_field_validated_sc("background", sc, validate_theme_background);
        vd.multi_field_validated_sc("header_background", sc, validate_theme_header_background);
        vd.multi_field_validated_sc("icon", sc, validate_theme_icon);
        vd.multi_field_validated_block_sc("sound", sc, validate_theme_sound);
        vd.multi_field_validated_block_sc("transition", sc, validate_theme_transition);
        vd.multi_field_validated_sc("effect_2d", sc, validate_theme_effect_2d);
    }
}

#[derive(Debug)]
pub struct EventBackground {
    validated_scopes: RwLock<Scopes>,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::EventBackground, EventBackground::add)
}

impl EventBackground {
    pub fn new() -> Self {
        let validated_scopes = RwLock::new(Scopes::empty());
        Self { validated_scopes }
    }

    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventBackground, key, block, Box::new(Self::new()));
    }
}

impl DbKind for EventBackground {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {}

    /// Like `EventTheme`, `EventBackground` are validated through the events (and themes) that use them.
    fn validate_call(
        &self,
        key: &Token,
        block: &Block,
        _caller: &Token,
        _caller_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        if self.validated_scopes.read().unwrap().contains(sc.scopes()) {
            return;
        }
        *self.validated_scopes.write().unwrap() |= sc.scopes();

        data.mark_used(Item::Localization, key.as_str());

        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.req_field("background");
        vd.multi_field_validated_block("background", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);
            vd.field_trigger("trigger", Tooltipped::No, sc);
            vd.field_item("reference", Item::File);
            vd.field_bool("video");
            vd.field_item("environment", Item::PortraitEnvironment);
            vd.field_value("ambience");
            vd.field_item("video_mask", Item::File);
        });
    }
}

#[derive(Debug)]
pub struct EventTransition {
    validated_scopes: RwLock<Scopes>,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::EventTransition, EventTransition::add)
}

impl EventTransition {
    pub fn new() -> Self {
        let validated_scopes = RwLock::new(Scopes::empty());
        Self { validated_scopes }
    }

    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EventTransition, key, block, Box::new(Self::new()));
    }
}

impl DbKind for EventTransition {
    fn validate(&self, _key: &Token, _block: &Block, _data: &Everything) {}

    /// Like `EventTheme`, `EventTransition` are validated through the events (and themes) that use them.
    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _caller: &Token,
        _caller_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        if self.validated_scopes.read().unwrap().contains(sc.scopes()) {
            return;
        }
        *self.validated_scopes.write().unwrap() |= sc.scopes();

        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.req_field("transition");
        vd.multi_field_validated_block("transition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);
            vd.field_trigger("trigger", Tooltipped::No, sc);
            vd.field_item("reference", Item::File);
            vd.field_bool("video");
            vd.field_value("ambience");
            vd.field_item("video_mask", Item::File);
            vd.field_numeric("duration");
        });
    }
}

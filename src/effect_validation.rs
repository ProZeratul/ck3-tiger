//! This module contains validators for effects that are generic across multiple games.

use crate::block::validator::Validator;
use crate::block::{Block, Comparator, Eq::Single, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_control};
use crate::everything::Everything;
use crate::item::Item;
use crate::report::Severity;
use crate::scopes::Scopes;
use crate::scriptvalue::validate_scriptvalue;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target_ok_this, validate_trigger_key_bv};
use crate::validate::validate_optional_duration;

pub fn validate_add_to_list(
    _key: &Token,
    value: &Token,
    _data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    sc.define_or_expect_list(value);
}

/// A specific validator for the three `add_to_variable_list` effects (`global`, `local`, and default).
pub fn validate_add_to_variable_list(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("target");
    vd.field_value("name");
    vd.field_target_ok_this("target", sc, Scopes::all_but_none());
}

/// A specific validator for the three `change_variable` effects (`global`, `local`, and default).
pub fn validate_change_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_value("name");
    vd.field_script_value("add", sc);
    vd.field_script_value("subtract", sc);
    vd.field_script_value("multiply", sc);
    vd.field_script_value("divide", sc);
    vd.field_script_value("modulo", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
}

/// A specific validator for the three `clamp_variable` effects (`global`, `local`, and default).
pub fn validate_clamp_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_value("name");
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
}

/// A specific validator for the `random_list` effect, which has a unique syntax.
pub fn validate_random_list(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = key.as_str();
    vd.field_integer("pick");
    vd.field_bool("unique"); // don't know what this does
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.unknown_block_fields(|key, block| {
        if key.expect_number().is_some() {
            validate_effect_control(caller, block, data, sc, tooltipped);
        }
    });
}

pub fn validate_remove_from_list(
    _key: &Token,
    value: &Token,
    _data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    sc.expect_list(value);
}

/// A specific validator for the three `round_variable` effects (`global`, `local`, and default).
pub fn validate_round_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("nearest");
    vd.field_value("name");
    vd.field_script_value("nearest", sc);
}

pub fn validate_save_scope(
    _key: &Token,
    value: &Token,
    _data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    sc.save_current_scope(value.as_str());
}

/// A specific validator for the `save_scope_value` effect.
pub fn validate_save_scope_value(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("value");
    if let Some(name) = vd.field_value("name") {
        // TODO: examine `value` field to check its real scope type
        sc.define_name_token(name.as_str(), Scopes::primitive(), name);
    }
    vd.field_script_value_or_flag("value", sc);
}

/// A specific validator for the three `set_variable` effects (`global`, `local`, and default).
pub fn validate_set_variable(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(_token) => (),
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("name");
            vd.field_value("name");
            vd.field_validated("value", |bv, data| match bv {
                BV::Value(token) => {
                    validate_target_ok_this(token, data, sc, Scopes::all_but_none());
                }
                BV::Block(_) => validate_scriptvalue(bv, data, sc),
            });
            validate_optional_duration(&mut vd, sc);
        }
    }
}

/// A specific validator for the `switch` effect, which has a unique syntax.
pub fn validate_switch(
    _key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    vd.set_case_sensitive(true);
    vd.req_field("trigger");
    if let Some(target) = vd.field_value("trigger") {
        // clone to avoid calling vd again while target is still borrowed
        let target = target.clone();
        vd.unknown_block_fields(|key, block| {
            if !key.is("fallback") {
                // Pretend the switch was written as a series of trigger = key lines
                let synthetic_bv = BV::Value(key.clone());
                validate_trigger_key_bv(
                    &target,
                    Comparator::Equals(Single),
                    &synthetic_bv,
                    data,
                    sc,
                    tooltipped,
                    false,
                    Severity::Error,
                );
            }

            validate_effect(block, data, sc, tooltipped);
        });
    }
}

pub fn validate_trigger_event(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::Event, token);
            data.events.check_scope(token, sc);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.field_item("id", Item::Event);
            #[cfg(feature = "ck3")]
            vd.field_item("on_action", Item::OnAction);
            #[cfg(feature = "ck3")]
            vd.field_target("saved_event_id", sc, Scopes::Flag);
            #[cfg(feature = "ck3")]
            vd.field_date("trigger_on_next_date");
            #[cfg(feature = "ck3")]
            vd.field_bool("delayed");
            #[cfg(feature = "vic3")]
            vd.field_bool("popup");
            validate_optional_duration(&mut vd, sc);
            if let Some(token) = block.get_field_value("id") {
                data.events.check_scope(token, sc);
            }
        }
    }
}
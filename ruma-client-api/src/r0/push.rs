//! Endpoints for push notifications.
use std::{convert::TryFrom, error::Error, fmt};

use ruma_common::push::{
    Action, ConditionalPushRule, ConditionalPushRuleInit, PatternedPushRule, PatternedPushRuleInit,
    PushCondition, PusherData, SimplePushRule, SimplePushRuleInit,
};
use ruma_serde::StringEnum;
use serde::{Deserialize, Serialize};

pub mod delete_pushrule;
pub mod get_notifications;
pub mod get_pushers;
pub mod get_pushrule;
pub mod get_pushrule_actions;
pub mod get_pushrule_enabled;
pub mod get_pushrules_all;
pub mod get_pushrules_global_scope;
pub mod set_pusher;
pub mod set_pushrule;
pub mod set_pushrule_actions;
pub mod set_pushrule_enabled;

/// Like `SimplePushRule`, but may represent any kind of push rule
/// thanks to `pattern` and `conditions` being optional.
///
/// To create an instance of this type, use one of its `From` implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct PushRule {
    /// The actions to perform when this rule is matched.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an
    /// event. A rule with no conditions always matches.
    ///
    /// Only applicable to underride and override rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against. Only applicable to content rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

impl From<SimplePushRule> for PushRule {
    fn from(push_rule: SimplePushRule) -> Self {
        let SimplePushRule { actions, default, enabled, rule_id, .. } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: None, pattern: None }
    }
}

impl From<PatternedPushRule> for PushRule {
    fn from(push_rule: PatternedPushRule) -> Self {
        let PatternedPushRule { actions, default, enabled, rule_id, pattern, .. } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: None, pattern: Some(pattern) }
    }
}

impl From<ConditionalPushRule> for PushRule {
    fn from(push_rule: ConditionalPushRule) -> Self {
        let ConditionalPushRule { actions, default, enabled, rule_id, conditions, .. } = push_rule;
        Self { actions, default, enabled, rule_id, conditions: Some(conditions), pattern: None }
    }
}

impl From<SimplePushRuleInit> for PushRule {
    fn from(init: SimplePushRuleInit) -> Self {
        let SimplePushRuleInit { actions, default, enabled, rule_id } = init;
        Self { actions, default, enabled, rule_id, pattern: None, conditions: None }
    }
}

impl From<ConditionalPushRuleInit> for PushRule {
    fn from(init: ConditionalPushRuleInit) -> Self {
        let ConditionalPushRuleInit { actions, default, enabled, rule_id, conditions } = init;
        Self { actions, default, enabled, rule_id, pattern: None, conditions: Some(conditions) }
    }
}

impl From<PatternedPushRuleInit> for PushRule {
    fn from(init: PatternedPushRuleInit) -> Self {
        let PatternedPushRuleInit { actions, default, enabled, rule_id, pattern } = init;
        Self { actions, default, enabled, rule_id, pattern: Some(pattern), conditions: None }
    }
}

impl From<PushRule> for SimplePushRule {
    fn from(push_rule: PushRule) -> Self {
        let PushRule { actions, default, enabled, rule_id, .. } = push_rule;
        SimplePushRuleInit { actions, default, enabled, rule_id }.into()
    }
}

/// An error that happens when `PushRule` cannot
/// be converted into `PatternedPushRule`
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct MissingPatternError;

impl fmt::Display for MissingPatternError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Push rule does not have a pattern.")
    }
}

impl Error for MissingPatternError {}

impl TryFrom<PushRule> for PatternedPushRule {
    type Error = MissingPatternError;

    fn try_from(push_rule: PushRule) -> Result<Self, Self::Error> {
        if let PushRule { actions, default, enabled, rule_id, pattern: Some(pattern), .. } =
            push_rule
        {
            Ok(PatternedPushRuleInit { actions, default, enabled, rule_id, pattern }.into())
        } else {
            Err(MissingPatternError)
        }
    }
}

/// An error that happens when `PushRule` cannot
/// be converted into `ConditionalPushRule`
#[derive(Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct MissingConditionsError;

impl fmt::Display for MissingConditionsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Push rule has no conditions.")
    }
}

impl Error for MissingConditionsError {}

impl TryFrom<PushRule> for ConditionalPushRule {
    type Error = MissingConditionsError;

    fn try_from(push_rule: PushRule) -> Result<Self, Self::Error> {
        if let PushRule {
            actions, default, enabled, rule_id, conditions: Some(conditions), ..
        } = push_rule
        {
            Ok(ConditionalPushRuleInit { actions, default, enabled, rule_id, conditions }.into())
        } else {
            Err(MissingConditionsError)
        }
    }
}

/// The kinds of push rules that are available.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum RuleKind {
    /// User-configured rules that override all other kinds.
    Override,

    /// Lowest priority user-defined rules.
    Underride,

    /// Sender-specific rules.
    Sender,

    /// Room-specific rules.
    Room,

    /// Content-specific rules.
    Content,

    #[doc(hidden)]
    _Custom(String),
}

/// Defines a pusher.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pusher {
    /// This is a unique identifier for this pusher. Max length, 512 bytes.
    pub pushkey: String,

    /// The kind of the pusher. If set to None in a call to set_pusher, this
    /// will delete the pusher
    pub kind: Option<PusherKind>,

    /// This is a reverse-DNS style identifier for the application. Max length, 64 chars.
    pub app_id: String,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// This string determines which set of device specific rules this pusher executes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
    pub lang: String,

    /// Information for the pusher implementation itself.
    pub data: PusherData,
}

/// Which kind a pusher is.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum PusherKind {
    /// A pusher that sends HTTP pokes.
    Http,

    /// A pusher that emails the user with unread notifications.
    Email,

    #[doc(hidden)]
    _Custom(String),
}

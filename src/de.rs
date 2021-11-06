use std::str::FromStr;
use std::time::Duration;

use measurements::{Frequency, Power};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::parse::{BoolStr, CardIds, DurationStr, FrequencyStr, Indices, PowerStr, Toggles};
use crate::{CardId, Chain, Knobs};

// Custom deserializers for parsed types.

impl<'de> Deserialize<'de> for BoolStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for CardId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for CardIds {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Vec<Knobs> = Deserialize::deserialize(deserializer)?;
        Ok(s.into())
    }
}

impl<'de> Deserialize<'de> for DurationStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for FrequencyStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Indices {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for PowerStr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Toggles {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

// Helper functions which map parsed types to the types expected by `struct
// Knobs` fields.

pub(super) fn bool<'de, D>(deserializer: D) -> std::result::Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: BoolStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn card_ids<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vec<CardId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: CardIds = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn duration<'de, D>(deserializer: D) -> std::result::Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: DurationStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn frequency<'de, D>(deserializer: D) -> std::result::Result<Option<Frequency>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: FrequencyStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn indices<'de, D>(deserializer: D) -> std::result::Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Indices = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn power<'de, D>(deserializer: D) -> std::result::Result<Option<Power>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: PowerStr = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub(super) fn toggles<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vec<(u64, bool)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Toggles = Deserialize::deserialize(deserializer)?;
    Ok(Some(v.into()))
}

pub mod card_data;
pub mod custom_data;
pub mod fsrs;
pub mod legacy;

use crate::domain::card::CardMeta;

#[must_use]
pub fn inject_schedule(cards: Vec<CardMeta>) -> Vec<CardMeta> {
    cards
}

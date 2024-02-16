pub mod algorithm;
mod node;
mod rollout_policy;
mod utils;

const WIN: i32 = i32::MAX;
// Make sure they negate to each other, unlike i32::MIN.
const LOSS: i32 = -WIN;

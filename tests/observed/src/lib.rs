//! Observed (DSL) twins of the subscriptions integration tests.
//!
//! This crate is a sibling of `tests-subscriptions` (`../integration-tests`):
//! the same on-chain flows, re-expressed in the anchor-litesvm testing DSL.
//! Where the integration tests assert on returned values, these tests render the
//! full observability surface of each transaction (a structured CPI tree, a
//! sequence diagram with and without lifelines, an authority graph, and an
//! ownership graph) into a narrative report under `target/md-reports/`.
//!
//! It reuses the integration crate's account builders for setup (the mint, the
//! ATAs, the subscription authority, the plans and delegations are not the point
//! we are making), and adds only the observed `send` and the rendering. The
//! shared pieces (`world()` and `render_all()`) live in [`harness`].

pub mod harness;

#[cfg(test)]
mod test_recurring;
#[cfg(test)]
mod test_smoke;
#[cfg(test)]
mod test_subscribe;


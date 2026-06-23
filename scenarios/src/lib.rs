//! Engine-neutral scenario vocabulary for the subscriptions suite.
//!
//! One set of test bodies, expressed against the `testsvm` neutral types,
//! driving the program through its wire interface. Each engine workspace binds
//! a concrete `B: TestSVM` via a generated `#[test]` shim.

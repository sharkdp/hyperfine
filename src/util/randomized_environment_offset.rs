/// Returns a string with a random length. This value will be set as an environment
/// variable to account for offset effects. See [1] for more details.
///
/// [1] Mytkowicz, 2009. Producing Wrong Data Without Doing Anything Obviously Wrong!.
///     Sigplan Notices - SIGPLAN. 44. 265-276. 10.1145/1508284.1508275.
pub fn value() -> String {
    "X".repeat(rand::random::<usize>() % 4096usize)
}

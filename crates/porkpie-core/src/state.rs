/// Current lock state for a vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultState {
    /// The vault key is present in memory and item operations are allowed.
    Unlocked,
    /// Decrypted items and the vault key are absent from memory.
    Locked,
}

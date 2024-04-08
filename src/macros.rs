#[macro_export]
macro_rules! use_later {
    $($_tree:tt) => {}
}

/// The magic sequence that identifies the init sequence
/// (the phase "gay gay homosexual gay" encoded in Baudot code)
const MAGIC_SEQ_GAY: [u8; 22] = [0x1A, 0x03, 0x15, 0x00, 0x1A, 
                                 0x03, 0x15, 0x00, 0x14, 0x18,
                                 0x1C, 0x18, 0x05, 0x01, 0x1D,
                                 0x07, 0x03, 0x12, 0x00, 0x1A,
                                 0x03, 0x15];


// this special sequence identifies the init superblock.
// The init superblock continues with the corpus body hash.
// It is followed by the maximum amount of CPU threads,
// Then the size of RAM follows it
// After that the corpus compiler version is embedded as 
// - MINOR (2 bytes)
// - MAJOR (2 bytes)
// - BUILD (2 bytes)
// - FLAGS (ALPHA-BETA-THETA-RELEASE)
// 
// Then the count of targets come. After that the offset of the first target
// finishes the init superblock.
//
// The init superblock is followed by the init corpus body.
// The init corpus body is a list of targets.
// Each target is a list of units.
// The targets end with the offset of the target they depend on.
// All these units are encoded as BSON documents and linked together.
// The corpus is also linked against the superblock and the controller API.

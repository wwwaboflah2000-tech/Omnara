pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; // 4096

// معايير الارتفاع: من -64 إلى 320 (384 بلوكة = 24 SubChunk)
pub const MIN_WORLD_Y: i32 = -64;
pub const MAX_WORLD_Y: i32 = 320;
pub const WORLD_HEIGHT: usize = (MAX_WORLD_Y - MIN_WORLD_Y) as usize; // 384
pub const SUBCHUNKS_PER_COLUMN: usize = WORLD_HEIGHT / CHUNK_SIZE; // 24
pub const SEA_LEVEL: i32 = 63; // مستوى سطح البحر الدقيق

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockId(pub u8);

impl BlockId {
    pub const AIR: BlockId = BlockId(0);
    pub const BEDROCK: BlockId = BlockId(1);
    pub const STONE: BlockId = BlockId(2);
    pub const DIRT: BlockId = BlockId(3);
    pub const GRASS: BlockId = BlockId(4);
    pub const WATER: BlockId = BlockId(5);
    pub const SAND: BlockId = BlockId(6);

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        self.0 != BlockId::AIR.0 && self.0 != BlockId::WATER.0
    }
}

pub struct SubChunk {
    pub blocks: [u8; CHUNK_VOLUME],
}

impl SubChunk {
    pub fn new() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
        }
    }

    #[inline(always)]
    pub fn get_index(x: usize, y: usize, z: usize) -> usize {
        x + (z << 4) + (y << 8)
    }

    #[inline(always)]
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return BlockId::AIR;
        }
        let index = Self::get_index(x, y, z);
        BlockId(self.blocks[index])
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            let index = Self::get_index(x, y, z);
            self.blocks[index] = block.0;
        }
    }
}

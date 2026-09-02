pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId(pub u8);

impl BlockId {
    pub const AIR: BlockId = BlockId(0);
    pub const STONE: BlockId = BlockId(1);
    pub const DIRT: BlockId = BlockId(2);
    pub const GRASS: BlockId = BlockId(3);

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        self.0 != 0
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
    fn get_index(x: usize, y: usize, z: usize) -> usize {
        x + (z << 4) + (y << 8)
    }

    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        if x < 0 || x >= CHUNK_SIZE as i32 ||
           y < 0 || y >= CHUNK_SIZE as i32 ||
           z < 0 || z >= CHUNK_SIZE as i32 {
            return BlockId::AIR;
        }
        let index = Self::get_index(x as usize, y as usize, z as usize);
        BlockId(self.blocks[index])
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            let index = Self::get_index(x, y, z);
            self.blocks[index] = block.0;
        }
    }

    // توليد تضاريس ثلاثية الأبعاد مع تموجات وتلال (3D Rolling Hills)
    pub fn generate_test_terrain(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                // معادلة موجية بسيطة لتوليد ارتفاعات متغيرة بين 4 إلى 12 بلوكة
                let wave = ((x as f32 * 0.4).sin() * 2.5 + (z as f32 * 0.4).cos() * 2.5) as i32;
                let height = (7 + wave).clamp(2, (CHUNK_SIZE - 2) as i32) as usize;

                for y in 0..CHUNK_SIZE {
                    if y < height.saturating_sub(3) {
                        self.set_block(x, y, z, BlockId::STONE); // الحجر في الأسفل
                    } else if y < height {
                        self.set_block(x, y, z, BlockId::DIRT);  // التراب في المنتصف
                    } else if y == height {
                        self.set_block(x, y, z, BlockId::GRASS); // العشب على السطح
                    }
                }
            }
        }
    }
}

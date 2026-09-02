pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; // 4096

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId(pub u8);

impl BlockId {
    pub const AIR: BlockId = BlockId(0);
    pub const STONE: BlockId = BlockId(1);
    pub const DIRT: BlockId = BlockId(2);
    pub const GRASS: BlockId = BlockId(3);

    #[inline(always)]
    pub fn is_opaque(&self) -> bool {
        self.0 != 0 // الهواء هو الوحيد الشفاف حالياً
    }
}

pub struct SubChunk {
    // 4096 بايت في الذاكرة فقط!
    pub blocks: [u8; CHUNK_VOLUME],
}

impl SubChunk {
    pub fn new() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
        }
    }

    // حساب الـ Index بسرعة البرق باستخدام الإزاحة الثنائية (Bit Shift)
    #[inline(always)]
    fn get_index(x: usize, y: usize, z: usize) -> usize {
        x + (z << 4) + (y << 8)
    }

    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        if x < 0 || x >= CHUNK_SIZE as i32 ||
           y < 0 || y >= CHUNK_SIZE as i32 ||
           z < 0 || z >= CHUNK_SIZE as i32 {
            return BlockId::AIR; // خارج حدود الـ Chunk نعتبره هواء مؤقتاً
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

    // توليد تضاريس تجريبية (تلة من العشب والتراب والحجر)
    pub fn generate_test_terrain(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    if y < 4 {
                        self.set_block(x, y, z, BlockId::STONE);
                    } else if y < 7 {
                        self.set_block(x, y, z, BlockId::DIRT);
                    } else if y == 7 {
                        self.set_block(x, y, z, BlockId::GRASS);
                    }
                }
            }
        }
    }
          }

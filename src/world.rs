use std::collections::HashMap;
use crate::voxel::{
    SubChunk, BlockId, CHUNK_SIZE, SUBCHUNKS_PER_COLUMN,
    MIN_WORLD_Y, MAX_WORLD_Y, SEA_LEVEL
};

pub struct ChunkColumn {
    pub sub_chunks: [Option<Box<SubChunk>>; SUBCHUNKS_PER_COLUMN],
}

impl ChunkColumn {
    pub fn new() -> Self {
        const INIT: Option<Box<SubChunk>> = None;
        Self {
            sub_chunks: [INIT; SUBCHUNKS_PER_COLUMN],
        }
    }

    #[inline(always)]
    pub fn get_or_create_subchunk(&mut self, sub_y: usize) -> &mut SubChunk {
        if self.sub_chunks[sub_y].is_none() {
            self.sub_chunks[sub_y] = Some(Box::new(SubChunk::new()));
        }
        self.sub_chunks[sub_y].as_mut().unwrap()
    }
}

pub struct VoxelWorld {
    pub columns: HashMap<u64, ChunkColumn>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn pack_column_key(chunk_x: i32, chunk_z: i32) -> u64 {
        ((chunk_x as u32 as u64) << 32) | (chunk_z as u32 as u64)
    }

    pub fn get_block_global(&self, gx: i32, gy: i32, gz: i32) -> BlockId {
        if gy < MIN_WORLD_Y || gy >= MAX_WORLD_Y {
            return BlockId::AIR;
        }

        let cx = gx.div_euclid(CHUNK_SIZE as i32);
        let cz = gz.div_euclid(CHUNK_SIZE as i32);
        let key = Self::pack_column_key(cx, cz);

        if let Some(column) = self.columns.get(&key) {
            let relative_y = (gy - MIN_WORLD_Y) as usize;
            let sub_y = relative_y / CHUNK_SIZE;
            let local_y = relative_y % CHUNK_SIZE;

            if let Some(sub_chunk) = &column.sub_chunks[sub_y] {
                let lx = gx.rem_euclid(CHUNK_SIZE as i32) as usize;
                let lz = gz.rem_euclid(CHUNK_SIZE as i32) as usize;
                return sub_chunk.get_block(lx, local_y, lz);
            }
        }

        BlockId::AIR
    }

    pub fn set_block_global(&mut self, gx: i32, gy: i32, gz: i32, block: BlockId) {
        if gy < MIN_WORLD_Y || gy >= MAX_WORLD_Y {
            return;
        }

        let cx = gx.div_euclid(CHUNK_SIZE as i32);
        let cz = gz.div_euclid(CHUNK_SIZE as i32);
        let key = Self::pack_column_key(cx, cz);

        let column = self.columns.entry(key).or_insert_with(ChunkColumn::new);

        let relative_y = (gy - MIN_WORLD_Y) as usize;
        let sub_y = relative_y / CHUNK_SIZE;
        let local_y = relative_y % CHUNK_SIZE;

        let sub_chunk = column.get_or_create_subchunk(sub_y);
        let lx = gx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let lz = gz.rem_euclid(CHUNK_SIZE as i32) as usize;

        sub_chunk.set_block(lx, local_y, lz, block);
    }

    // توليد تضاريس متموجة مع بحار وشواطئ وجبال وبيدروك
    pub fn generate_test_world(&mut self, radius_chunks: i32) {
        for cx in -radius_chunks..=radius_chunks {
            for cz in -radius_chunks..=radius_chunks {
                for lx in 0..CHUNK_SIZE as i32 {
                    for lz in 0..CHUNK_SIZE as i32 {
                        let gx = cx * CHUNK_SIZE as i32 + lx;
                        let gz = cz * CHUNK_SIZE as i32 + lz;

                        // تضاريس ترتفع وتنخفض بين 54 (بحر) و 78 (جبال)
                        let wave = ((gx as f32 * 0.08).sin() * 8.0 + (gz as f32 * 0.08).cos() * 8.0) as i32;
                        let terrain_height = 64 + wave;

                        // 1. قاع العالم (البيدروك عند -64)
                        self.set_block_global(gx, MIN_WORLD_Y, gz, BlockId::BEDROCK);

                        // 2. طبقات الصخور والتراب
                        for gy in (MIN_WORLD_Y + 1)..=terrain_height {
                            if gy < terrain_height - 4 {
                                self.set_block_global(gx, gy, gz, BlockId::STONE);
                            } else if gy < terrain_height {
                                if terrain_height <= SEA_LEVEL + 1 {
                                    self.set_block_global(gx, gy, gz, BlockId::SAND); // شاطئ رملي
                                } else {
                                    self.set_block_global(gx, gy, gz, BlockId::DIRT);
                                }
                            } else if gy == terrain_height {
                                if gy < SEA_LEVEL {
                                    self.set_block_global(gx, gy, gz, BlockId::SAND); // قاع البحر رمل
                                } else if gy <= SEA_LEVEL + 1 {
                                    self.set_block_global(gx, gy, gz, BlockId::SAND); // شاطئ
                                } else {
                                    self.set_block_global(gx, gy, gz, BlockId::GRASS); // عشب يابس
                                }
                            }
                        }

                        // 3. ملء مياه البحر حتى المستوى 63 بدقة
                        if terrain_height < SEA_LEVEL {
                            for gy in (terrain_height + 1)..=SEA_LEVEL {
                                self.set_block_global(gx, gy, gz, BlockId::WATER);
                            }
                        }
                    }
                }
            }
        }
    }
}

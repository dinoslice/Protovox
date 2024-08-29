use game::chunk::data::ChunkData;

pub struct ClientChunk {
    pub data: ChunkData,
    pub dirty: bool,
}

impl ClientChunk {
    pub fn new_dirty(data: ChunkData) -> Self {
        Self {
            data,
            dirty: true,
        }
    }
}
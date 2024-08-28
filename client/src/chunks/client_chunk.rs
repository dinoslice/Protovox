use game::chunk::data::ChunkData;

pub struct ClientChunk {
    pub data: ChunkData,
    pub dirty: bool,
}
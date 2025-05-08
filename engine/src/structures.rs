use std::sync::Arc;
use game::block::Block;
use game::chunk::pos::ChunkPos;

#[derive(Debug, Clone)]
pub struct Structure {
    data: Arc<[Block]>,
    size: ChunkPos,
    origin: ChunkPos,
}
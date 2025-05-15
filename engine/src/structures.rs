use game::block::Block;
use game::chunk::pos::ChunkPos;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    data: Box<[Block]>, // TODO: maybe switch to Arc
    size: ChunkPos,
    origin: ChunkPos,
}
use glm::Vec3;
use shipyard::{EntityId, IntoIter, IntoWithId, View};
use crate::components::{Entity, Hitbox, Transform};

pub fn collides_with_any_entity(corner_a: Vec3, corner_b: Vec3, v_entity: View<Entity>, v_transform: View<Transform>, v_hitbox: View<Hitbox>) -> Option<EntityId> {
    let min = glm::min2(&corner_a, &corner_b);
    let max = glm::max2(&corner_a, &corner_b);
    
    for (id, (_, transform, hitbox)) in (&v_entity, &v_transform, &v_hitbox).iter().with_id() {
        let half_hitbox = hitbox.0 * 0.5;
        
        let e_min = transform.position - half_hitbox;
        let e_max = transform.position + half_hitbox;
        
        if min <= e_max && max >= e_min {
            return Some(id);
        }
    }
    
    None
}
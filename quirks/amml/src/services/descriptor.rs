// the descriptor the manager registers the mod from; it names the mod and the settings schema
pub struct QuirksDescriptor;

impl QuirksDescriptor {
    pub const TEXT: &'static str = include_str!("../../descriptor.hcl");
}

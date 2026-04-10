#[derive(Debug, Clone)]
pub struct Schema<C, L> {
    pub collections: C,
    pub links: L,
}

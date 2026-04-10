/// this is a less stable trait
/// I was trying to avoid any heap allocation for types that don't need it
/// instead of `Vec<String>` I tried `impl Iterator<Item = &str>`
/// but I got brutally blocked by lifetime problems
pub trait Members<S> {
    fn members_names(&self) -> Vec<String>;
}

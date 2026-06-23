pub trait LifetimeTrait<'a>: 'a {}

// impl<'a> LifetimeTrait<'a> for &'a str {}
impl<'a, 'b> LifetimeTrait<'a> for &'b str where 'b: 'a {}

fn main<T>()
where
    for<'q> T: LifetimeTrait<'q>,
{
}

#[test]
fn wait() {
    main::<&str>();
}

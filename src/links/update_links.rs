pub struct SetNew<Relation, Data> {
    pub relation: Relation,
    pub data: Data,
}

pub struct SetId<Relation, Id> {
    pub relation: Relation,
    pub id: Id,
}

pub struct Unset<Relation> {
    pub relation: Relation,
}

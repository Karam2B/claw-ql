use crate::{
    QueryBuilder,
    collections::{Collection, CollectionBasic, OnMigrate},
    json_client::{DynamicLink, JsonClientBuilder, ReturnAsJsonMap, SelectOneJsonFragment},
    operations::select_one_op::SelectOneFragment,
};

pub trait DynamicLinkForRelation<S> {
    fn global_ident(&self) -> &'static str;
    fn on_each_select_one_request(
        &self,
        input: Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, String>;
}

impl<F, T> JsonClientBuilder for Relation<F, T>
where
    F: CollectionBasic + 'static,
    T: CollectionBasic + 'static,
{
    type BuildEntry = RelationEntry;

    fn init(&self) -> Self::BuildEntry {
        RelationEntry {
            from: self.from.table_name().to_string(),
            to: self.to.table_name().to_string(),
            ty: "unkown".to_string(),
        }
    }

    type RuntimeEntry = RelationEntries;
    fn finish(&self, build_ctx: &Vec<Box<dyn Any>>) -> Result<Self::RuntimeEntry, String> {
        Ok(RelationEntries {
            entries: build_ctx
                .iter()
                .filter_map(|e| {
                    let s = (**e).downcast_ref::<RelationEntry>()?;
                    Some(s.clone())
                })
                .collect(),
            private_to_construct_hack: (),
        })
    }
}

impl<S, F, T> DynamicLink<S> for Relation<F, T>
where
    S: QueryBuilder,
    Relation<F, T>: LinkData<
            F,
            Spec: Any
                      + SelectOneFragment<S, Output: Serialize>
                      // + InsertOneFragment<S, Output: Serialize>
                      + DynamicLinkForRelation<S>,
        >,
    Relation<F, T>: Clone,
    F: Clone + 'static,
    F: Collection<S>,
    T: Collection<S> + 'static,
{
    fn json_entry(&self) -> Vec<&'static str> {
        vec!["relation", self.to.table_name()]
    }

    type SelectOneInput = HashMap<String, Value>;
    type SelectOne = ReturnAsJsonMap<Box<dyn SelectOneJsonFragment<S>>>;
    fn on_select_one(
        &self,
        base_col: String,
        input: Self::SelectOneInput,
        entry: &Self::RuntimeEntry,
    ) -> Result<Option<Self::SelectOne>, String> {
        let base = base_col.to_case(Case::Snake);
        let spec = self.clone().spec(self.from.clone());

        // make sure the base collection have the said relation
        let rels = entry.entries.iter().filter(|e| e.from == base);

        let mut not_related = Vec::default();

        let s = input
            .into_iter()
            .filter_map(|(to, input)| {
                // make sure base collection is related to `to`
                if rels.clone().any(|rel| rel.to == to).not() {
                    not_related.push(format!("{base} is not related to {to}",))
                }

                let specr = spec.on_each_select_one_request(input);

                // propegate all errors
                // todo: for now I'm displaying last error only! how to make this better?
                match specr {
                    Ok(s) => {
                        return Some((to, s));
                    }
                    Err(err) => {
                        not_related
                            .push(format!("invalid input for relation `{base}->{to}`: {err}",));
                        return None;
                    }
                };
            })
            .collect::<Vec<_>>();

        if not_related.is_empty().not() {
            return Err(not_related.last().unwrap().clone());
        }

        Ok(Some(ReturnAsJsonMap(s)))
    }

    type InsertOneInput = ();

    type InsertOne = ();

    type DeleteOneInput = ();

    type DeleteOne = ();

    type UpdateOneInput = ();

    type UpdateOne = ();

    fn on_insert_one(
        &self,
        _base_col: String,
        _input: Self::InsertOneInput,
        _entry: &Self::RuntimeEntry,
    ) -> Result<Option<Self::InsertOne>, String> {
        todo!()
    }

    fn on_delete_one(
        &self,
        _base_col: String,
        _input: Self::DeleteOneInput,
        _entry: &Self::RuntimeEntry,
    ) -> Result<Option<Self::DeleteOne>, String> {
        todo!()
    }

    fn on_update_one(
        &self,
        _base_col: String,
        _input: Self::UpdateOneInput,
        _entry: &Self::RuntimeEntry,
    ) -> Result<Option<Self::UpdateOne>, String> {
        todo!()
    }
}

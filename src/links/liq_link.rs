pub trait DynCollVection {
    fn clone_self(&self) -> Box<dyn DynCollection>;
    fn table_name_lowercase(&self) -> &str;
    fn members_names(&self) -> Vec<String>;
}

impl<T> DynCollection for T
where
    T: CollectionBasic + Collection + Clone,
    // T::Members: LocalTrait
    for<'a> &'a T::Members: Into<Vec<Box<dyn MemberBasic>>>,
{
    fn clone_self(&self) -> Box<dyn DynCollection> {
        Box::new(self.clone())
    }
    fn members_names(&self) -> Vec<String> {
        self.members().into().iter().map(|e| e.name()).collect()
    }
    fn table_name_lowercase(&self) -> &str {
        CollectionBasic::table_name_lower_case(&self)
    }
}

// impl LocalTrait for (T0, T1, T2)

pub trait LiqLink: Send + Sync {
    type This;
    type CreateLinkInput;
    type CreateLinkError;
    type CreateLinkOk;
    fn create_link(
        &mut self,
        collections: &HashMap<String, Box<dyn DynCollection>>,
        base: &dyn DynCollection,
        input: Self::CreateLinkInput,
    ) -> Result<(Self::CreateLinkOk, Self::This), Self::CreateLinkError>;

    type OnRequestInput;
    type OnRequestError;
    fn on_request(
        &self,
        base: &dyn DynCollection,
        input: Self::OnRequestInput,
    ) -> Result<Self::This, Self::OnRequestError>;
}

pub trait LiqLinkExt<S>: Send + Sync {
    fn create_link(
        &mut self,
        collections: &HashMap<String, Box<dyn JsonCollection<S>>>,
        base: &dyn JsonCollection<S>,
        input: serde_json::Value,
    ) -> Result<(serde_json::Value, Vec<String>), LiqError>;

    fn on_select_one(
        &self,
        base: &dyn JsonCollection<S>,
        input: serde_json::Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, LiqError>;
}

#[simple_enum]
#[derive(Debug)]
pub enum LiqError {
    RegisteredError,
    FailedToParseBody,
}

impl<S, T> LiqLinkExt<S> for T
where
    T: LiqLink,
    S: QueryBuilder,
    T::This: OnMigrate<S>,
    T::This: SelectOneFragment<S>,
    (T::This, <T::This as SelectOneFragment<S>>::Inner): SelectOneJsonFragment<S>,
    T::CreateLinkInput: DeserializeOwned,
    T::CreateLinkOk: Serialize,
    T::CreateLinkError: Serialize + HttpError,
    T::OnRequestInput: DeserializeOwned,
    T::OnRequestError: Serialize + HttpError,
{
    fn create_link(
        &mut self,
        collections: &HashMap<String, Box<dyn JsonCollection<S>>>,
        base: &dyn JsonCollection<S>,
        input: serde_json::Value,
    ) -> Result<(JsonValue, Vec<String>), LiqError> {
        let input = from_value::<T::CreateLinkInput>(input)
            .map_err(|e| LiqError::FailedToParseBody(FailedToParseBody(e.to_string())))?;

        LiqLink::create_link(self, collections, base, input)
            .map(|(this, mig)| {
                (
                    serde_json::to_value(this).unwrap(),
                    mig.custom_migrate_statements(),
                )
            })
            .map_err(|e| {
                LiqError::RegisteredError(RegisteredError(
                    todo!(),
                    serde_json::to_value(e).unwrap(),
                ))
            })
    }
    fn on_select_one(
        &self,
        base: &dyn JsonCollection<S>,
        input: serde_json::Value,
    ) -> Result<Box<dyn SelectOneJsonFragment<S>>, LiqError> {
        let input = from_value::<T::OnRequestInput>(input)
            .map_err(|e| LiqError::FailedToParseBody(FailedToParseBody(e.to_string())))?;

        let re = LiqLink::on_request(self, base, input).map_err(|e| {
            let s = e.status_code();
            LiqError::RegisteredError(RegisteredError(s, serde_json::to_value(e).unwrap()))
        })?;

        Ok(Box::new((re, Default::default())))
    }
}

// #[derive(Debug)]
// pub struct RegisteredError(StatusCode, JsonValue);

// #[derive(Debug)]
// pub struct EntryIsNotFound {
//     pub filters: JsonValue,
// }

// impl HttpError for EntryIsNotFound {
//     fn status_code(&self) -> StatusCode {
//         StatusCode::NOT_FOUND
//     }
// }
// should be moved to other place?
// impl HttpError for RegisteredError {
//     fn status_code(&self) -> StatusCode {
//         self.0.clone()
//     }
// }

pub trait CollectionsStore {
    type Store;
}

pub trait DynamicLink<DynamicBase: CollectionsStore, S> {
    type OnRequest;
    type OnRequestInput;
    type OnRequestError;

    fn on_request(
        &self,
        base: DynamicBase,
        input: Self::OnRequestInput,
    ) -> Result<Self::OnRequest, Self::OnRequestError>;

    type CreateLinkOk;
    type CreateLinkInput;
    type CreateLinkError;

    fn create_link(
        &self,
        store: &DynamicBase::Store,
        input: Self::CreateLinkInput,
    ) -> Result<Self::CreateLinkOk, Self::CreateLinkError>;

    type ModifyLinkOk;
    type ModifyLinkInput;
    type ModifyLinkError;

    fn modify_link(
        &self,
        store: &DynamicBase::Store,
        input: Self::ModifyLinkInput,
    ) -> Result<Self::ModifyLinkOk, Self::ModifyLinkError>;
}

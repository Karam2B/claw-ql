#![allow(unused)]

mod client_interface {
    //*******************
    //*
    //* SupportedType
    //*
    //*******************
    #[derive(Debug)]
    pub enum SupportedType {
        String,
        Boolean,
    }

    //*******************
    //*
    //* AddCollection
    //*
    //*******************
    #[derive(Debug)]
    pub struct AddCollectionInput {
        pub name: String,
        pub fields: Vec<DynamicField<SupportedType>>,
    }

    #[derive(Debug)]
    pub struct DynamicField<T> {
        pub name: String,
        pub type_info: T,
        pub is_optional: bool,
    }

    pub type AddCollectionOutput = ();

    //*******************
    //*
    //* JsonClient
    //*
    //*******************
    pub struct ClientError {}
    impl ClientError {
        pub fn disconnected_channel() -> Self {
            Self {}
        }
    }
    pub struct Client {
        pub(crate) sender: tokio::sync::mpsc::UnboundedSender<(
            OperationInput,
            oneshot::Sender<Result<OperationOutput, ClientError>>,
        )>,
    }

    macro_rules! ops {
        ($([$name:ident, $upper_case:ident]),*) => {
            paste::paste!{
                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationOutput {
                    $(
                        $upper_case ([<$upper_case Output>]),
                    )*
                }

                #[derive(Debug)]
                #[non_exhaustive]
                pub enum OperationInput {
                    $(
                        $upper_case ([<$upper_case Input>]),
                    )*
                }

                impl Client {
                    $(
                        pub fn
                            $name(&self, input: [<$upper_case Input>])
                        -> impl Future<Output = Result<[<$upper_case Output>], ClientError>> {
                            let (tx, rx) = oneshot::async_channel::<Result<OperationOutput, ClientError>>();
                            self.sender.send((OperationInput::$upper_case(input), tx)).unwrap();
                            async move {
                                let output = rx.await.map_err(|_| ClientError::disconnected_channel())?;
                                let mapp = match output {
                                    Ok(OperationOutput::$upper_case(e)) => Ok(e),
                                    Ok(_) => panic!("invalid mapping"),
                                    Err(e) => Err(e),
                                };

                                return mapp;
                            }
                        }
                    )*
                }

                crate::default_executor!($([$name, $upper_case]),*);
            }
        };
    }

    ops!([add_collection, AddCollection]);
}

mod add_collection_mod {
    use crate::json_client::client_interface::AddCollectionInput;
    use crate::json_client::client_interface::AddCollectionOutput;
    use crate::json_client::client_interface::ClientError;
    use crate::json_client::sqlx_executor::SqlxExecutorData;
    use sqlx::Database;
    use std::future::Future;
    use std::sync::Arc;

    pub fn add_collection<S: Database>(
        this: Arc<SqlxExecutorData<S>>,
        input: AddCollectionInput,
    ) -> impl Future<Output = Result<AddCollectionOutput, ClientError>> + 'static + Send + use<S>
    {
        async move { todo!() }
    }
}

mod sqlx_executor {
    use crate::json_client::{
        client_interface::{ClientError, DynamicField, OperationInput, OperationOutput},
        supported_types_vtable::SupportedTypeVTable,
    };
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::RwLock as Trw;

    pub struct SqlxExecutor<S> {
        pub(crate) reciever: tokio::sync::mpsc::UnboundedReceiver<(
            OperationInput,
            oneshot::Sender<Result<OperationOutput, ClientError>>,
        )>,
        pub(crate) data: Arc<SqlxExecutorData<S>>,
    }

    pub(crate) struct SqlxExecutorData<S> {
        pub(crate) collections: Trw<HashMap<String, Trw<DynamicCollection<S>>>>,
        _s: S,
    }

    pub(crate) struct DynamicCollection<S> {
        pub(crate) name: String,
        pub(crate) fields: Vec<DynamicField<SupportedTypeVTable<S>>>,
    }

    #[macro_export]
    macro_rules! default_executor {
        ($([$name:ident, $upper_case:ident]),*) => {
            impl<S> $crate::json_client::sqlx_executor::SqlxExecutor<S>
            where S: ::sqlx::Database,
            {
                pub fn run(mut self) -> impl Future<Output = ::std::convert::Infallible> {
                    async move {
                        loop {
                            let operation = self.reciever.recv().await.unwrap();

                            paste::paste!{
                            match operation.0 {
                                $(OperationInput::$upper_case(input) => {
                                    let future = $crate::json_client::[<$name _mod>]::[<$name>](self.data.clone(), input);
                                    tokio::spawn(async move {
                                        let resolve_future = future.await;
                                        operation.1.send(resolve_future.map(|e| OperationOutput::$upper_case(e))).unwrap();
                                    });
                                })*
                            }}
                        }
                    }
                }
            }
        };
    }
}
mod supported_types_vtable {
    use std::marker::PhantomData;

    pub struct SupportedTypeVTable<S> {
        pub type_name: fn() -> &'static str,
        _s: PhantomData<S>,
    }

    impl<S> SupportedTypeVTable<S> {
        fn new_as<T>() -> Self
        where
            T: Send + Sync + 'static,
        {
            let type_name = || {
                return std::any::type_name::<T>();
            };

            Self {
                type_name,
                _s: PhantomData,
            }
        }
    }
}

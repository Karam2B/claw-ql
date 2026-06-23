macro_rules! define_collection {
    (struct $pascal_case:ident $size:literal {$(
        $member:ident: $type:ty,
    )*}) => {
        const _: ()  ={
            if $size == 0 {
                panic!("size must be greater than 0");
            }
        };

        paste::paste! {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $pascal_case {
            $(
                pub $member: $type,
            )*
        }

        #[derive(Default, Debug, PartialEq, Eq, Clone)]
        pub struct [<$pascal_case Partial>] {
            $(
                pub $member: $crate::update_mod::Update<$type>,
            )*
        }

        #[derive(Clone, Copy, Default)]
        pub struct [<$pascal_case Handler>];

        impl AsRef<str> for [<$pascal_case Handler>] {
            fn as_ref(&self) -> &str {
                stringify!($pascal_case)
            }
        }

        // impl Singleton for $name
        const _: () = {
            use crate::singleton::Singleton;
            impl Singleton for [<$pascal_case Handler>] {
                fn singleton() -> &'static Self {
                    &[<$pascal_case Handler>]
                }
            }
        };

        // impl AsTuple for $name
        const _: () = {
            use crate::tuple_trait::AsTuple;
            impl AsTuple for $pascal_case {
                type Tuple = ($($type,)*);
                const NAMES: &'static [&'static str] = &[$(stringify!($member),)*];
                fn into_tuple(self) -> Self::Tuple {
                    ($(self.$member,)*)
                }
                fn from_tuple(($($member,)*): Self::Tuple) -> Self {
                    Self {
                        $( $member, )*
                    }
                }
            }

            impl AsTuple for [<$pascal_case Partial>] {
                type Tuple = ($($crate::update_mod::Update<$type>,)*);
                const NAMES: &'static [&'static str] = &[$(stringify!($member),)*];
                fn into_tuple(self) -> Self::Tuple {
                    ($(self.$member,)*)
                }
                fn from_tuple(($($member,)*): Self::Tuple) -> Self {
                    Self {
                        $( $member, )*
                    }
                }
            }
        };

        // impl Collection for $pascal_case
        const _: () = {
            use crate::collections::Collection;
            use crate::collections::SingleIncremintalInt;
            impl Collection for [<$pascal_case Handler>] {
                fn table_name(&self) -> &str {
                    stringify!($pascal_case)
                }
                fn table_name_lower_case(&self) -> &str {
                    stringify!([<$pascal_case:snake>])
                }
                type InputData = $pascal_case;
                type UpdateData = [<$pascal_case Partial>];
                type OutputData = $pascal_case;
                type Id = SingleIncremintalInt<&'static str>;
                fn id(&self) -> Self::Id {
                    SingleIncremintalInt(stringify!($pascal_case))
                }
            }
        };

        // impl HasHandler for $pascal_case
        const _: () = {
            use crate::collections::HasHandler;
            impl HasHandler for $pascal_case {
                type Handler = [<$pascal_case Handler>];
            }
            impl HasHandler for [<$pascal_case Partial>] {
                type Handler = [<$pascal_case Handler>];
            }
        };

        // impl FromRowAlias
        const _: () = {
            use sqlx::{ColumnIndex, Decode, Row, Type};
            use crate::from_row::{FromRowAlias, FromRowData, FromRowError, RowPostAliased, RowPreAliased, RowTwoAliased};

            impl FromRowData for [<$pascal_case Handler>] {
                type RData = $pascal_case;
            }

            impl<'r, R> FromRowAlias<'r, R> for [<$pascal_case Handler>]
            where
                R: Row + 'r,
                $(
                    $type: Type<R::Database> + Decode<'r, R::Database>,
                )*
                for<'q> &'q str: ColumnIndex<R>,

            {
                fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
                    Ok(
                        $pascal_case {
                            $(
                                $member: row.try_get(stringify!($member))?,
                            )*
                        }
                    )
                }
                fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                    Ok(
                        $pascal_case {
                            $(
                                $member: row.try_get(stringify!($member))?,
                            )*
                        }
                    )
                }
                fn post_alias(&self, _: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                    panic!("to depricate");
                }
                fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                    Ok(
                        $pascal_case {
                            $(
                                $member: row.try_get(stringify!($member))?,
                            )*
                        }
                    )
                }
            }

        };


        // impl gen_serde::Deserialize for $pascal_case
        const _: () = {
            use crate::gen_serde::{Deserialize, DeserializeMap, DeserializeSpec, Deserializer, KnownKey};

            impl DeserializeSpec for $pascal_case {
                type Handler = ();
            }
            impl<'de, S> Deserialize<'de, S> for $pascal_case
            where
                S: Deserializer<'de>,
                S: DeserializeMap<'de>,
                $(
                    $type: Deserialize<'de, S>,
                )*
                S: KnownKey<&'static str>,
            {
                fn deserialize(_: Self::Handler, serialized: &mut S) -> Result<Self, S::Err> {
                    let mut map = serialized.start_map()?;
                    let res = Self {
                        $(
                            $member: serialized.deserialize_with_known_key(&mut map, stringify!($member), ())?,
                        )*
                    };
                    serialized.finish(map)?;

                    Ok(res)
                }
            }
        };

        // impl gen_serde::Serialize for $pascal_case
        const _: () = {
            use crate::gen_serde::{Serialize, ObjectEncoding};

            impl<F> Serialize<F> for $pascal_case
            where
                F: ObjectEncoding,
                $(
                    $type: Serialize<F>,
                )*
                str: Serialize<F>,
            {
                fn serialize(&self, fmt: &mut F) {
                    let mut object  = F::serialize_start(fmt);
                    $(
                        F::serialize_pair(fmt, &mut object, stringify!($member), &self.$member);
                    )*
                    F::serialize_end(fmt, object);
                }
            }
        };

        // impl ExpressionsForOperation for $pascal_case
        const _: () = {
            use crate::operations::operations_expressions_crossover::ExpressionsForOperation;
            use crate::sqlx_query_builder::basic_expressions::{
                AliasedScopedColumn,  ScopedColumn,
            };

            impl ExpressionsForOperation for [<$pascal_case Handler>] {
                type Identifier = [&'static str; $size];

                fn identifier(&self) -> Self::Identifier {
                    [$(stringify!($member),)*]
                }

                type Scoped = [
                    ScopedColumn<(&'static str,), (&'static str,)> ;$size
                ];

                fn scoped(&self) -> Self::Scoped {
                    [
                        $(
                            ScopedColumn {
                                table: (stringify!($pascal_case),),
                                col: (stringify!($member),),
                            },
                        )*
                    ]
                }

                type ScopedAliased = [
                    AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, &'static str)> ; $size
                ];

                fn scoped_aliased(&self, alias: &'static str) -> Self::ScopedAliased {
                    [
                        $(
                            AliasedScopedColumn {
                                table: (stringify!($pascal_case),),
                                column: (stringify!($member),),
                                alias: (alias, stringify!($member)),
                            },
                        )*
                    ]
                }

                type NumScopedAliased = [
                    AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, usize, &'static str)> ; $size
                ];

                fn num_scoped_aliased(&self, num: usize, alias: &'static str) -> Self::NumScopedAliased {
                    [
                        $(
                            AliasedScopedColumn {
                                table: (stringify!($pascal_case),),
                                column: (stringify!($member),),
                                alias: (alias, num, stringify!($member)),
                            },
                        )*
                    ]
                }
            }

            use crate::operations::operations_expressions_crossover::TableExpressions;

            impl TableExpressions for [<$pascal_case Handler>] {
                type SnakeCase = &'static str;
                type PascalCase = &'static str;
                fn table_name_snake_case(&self) -> Self::SnakeCase {
                    stringify!([<$pascal_case:snake>])
                }
                fn table_name_pascal_case(&self) -> Self::PascalCase {
                    stringify!($pascal_case)
                }
                type Migrate = ();
                fn migrate(&self) -> Self::Migrate {
                    todo!()
                }
            }


        };

        // impl OnInsert for $pascal_case
        const _: () = {
            use crate::operations::operations_expressions_crossover::OnInsert;
            use crate::operations::operations_expressions_crossover::ExpressionsForOperation;
            use crate::sqlx_query_builder::IsOpExpression;
            use crate::sqlx_query_builder::{ManyExpressions, StatementBuilder};
            use crate::database_extention::DatabaseExt;
            use sqlx::{Encode, Type};

            impl OnInsert<$pascal_case> for [<$pascal_case Handler>] {
                type InsertExpression = $pascal_case;
                fn on_insert(&self, input: $pascal_case) -> Self::InsertExpression {
                    input
                }
                type InsertId = [&'static str; $size];
                fn on_insert_with_id(&self, input: $pascal_case) -> (Self::InsertId, Self::InsertExpression) {
                    (self.identifier(), input)
                }
            }

            impl IsOpExpression for $pascal_case {
                fn is_op(&self) -> bool {
                    true
                }
            }

            impl<'q, S> ManyExpressions<'q, S> for $pascal_case
            where
                S: DatabaseExt,
                $(
                    $type: Encode<'q, S> + 'q + Type<S>,
                )*
            {
                fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
                where
                    S: DatabaseExt,
                {
                    ctx.syntax(start);

                    $(
                        ctx.bind(self.$member);
                        ctx.syntax(join);
                    )*
                }
            }
        };

        // impl OnUpdate for $pascal_case
        const _: () = {
            use crate::operations::operations_expressions_crossover::OnUpdate;
            use crate::update_mod::Update;
            use crate::sqlx_query_builder::IsOpExpression;
            use crate::sqlx_query_builder::{ManyExpressions, StatementBuilder};
            use crate::database_extention::DatabaseExt;
            use sqlx::{Encode, Type};

            impl OnUpdate<[<$pascal_case Partial>]> for [<$pascal_case Handler>] {
                type UpdateExpression = [<$pascal_case Partial>];
                fn on_update(&self, input: [<$pascal_case Partial>]) -> Self::UpdateExpression {
                    input
                }
            }

            impl IsOpExpression for [<$pascal_case Partial>] {
                fn is_op(&self) -> bool {
                    false
                    $(
                        ||
                        match self.$member {
                            Update::Set(_) => true,
                            _ => false,
                        }
                    )*
                }
            }

            impl<'q, S> ManyExpressions<'q, S> for [<$pascal_case Partial>]
            where
                S: DatabaseExt,
                $(
                    $type: Encode<'q, S> + 'q + Type<S>,
                )*
            {
                fn expression(self, start: &'static str, join: &'static str, ctx: &mut StatementBuilder<'q, S>)
                where
                    S: DatabaseExt,
                {
                    if self.is_op() { ctx.syntax(start); }

                    $(
                        if let Update::Set(value) = self.$member {
                            ctx.syntax(stringify!($member));
                            ctx.syntax(" = ");
                            ctx.bind(value);
                            ctx.syntax(join);
                        }
                    )*
                }
            }
        };

        // members
        pub mod [<$pascal_case:snake _members>] {
            use crate::operations::operations_expressions_crossover::ExpressionsForOperation;
            use crate::sqlx_query_builder::basic_expressions::ScopedColumn;
            use crate::sqlx_query_builder::basic_expressions::AliasedScopedColumn;

            #[allow(non_camel_case_types)]
            pub struct id;
            impl ExpressionsForOperation for id {
                type Identifier = &'static str;
                fn identifier(&self) -> Self::Identifier {
                    "id"
                }
                type Scoped = ScopedColumn<(&'static str,), (&'static str,)>;
                fn scoped(&self) -> Self::Scoped {
                    ScopedColumn {
                        table: (stringify!($pascal_case),),
                        col: ("id",),
                    }
                }
                type ScopedAliased = AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, &'static str)>;
                fn scoped_aliased(&self, alias: &'static str) -> Self::ScopedAliased {
                    AliasedScopedColumn {
                        table: (stringify!($pascal_case),),
                        column: ("id",),
                        alias: (alias, "id"),
                    }
                }
                type NumScopedAliased = AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, usize, &'static str)>;
                fn num_scoped_aliased(&self, num: usize, alias: &'static str) -> Self::NumScopedAliased {
                    AliasedScopedColumn {
                        table: (stringify!($pascal_case),),
                        column: ("id",),
                        alias: (alias, num, "id"),
                    }
                }
            }

            $(
                #[allow(non_camel_case_types)]
                #[derive(Debug, Clone)]
                pub struct $member;

                impl AsRef<str> for $member {
                    fn as_ref(&self) -> &str {
                        stringify!($member)
                    }
                }

                impl $member {
                    pub fn bind(value: $type) ->
                    crate::operations::operations_expressions_crossover::NamedBind<
                        super::[<$pascal_case Handler>],
                        $member,
                        $type,
                    >
                    {
                        crate::operations::operations_expressions_crossover::NamedBind {
                            table: super::[<$pascal_case Handler>],
                            name: $member,
                            value: value,
                        }
                    }
                }

                impl ExpressionsForOperation for $member {
                    type Identifier = &'static str;
                    fn identifier(&self) -> Self::Identifier {
                        stringify!($member)
                    }
                    type Scoped = ScopedColumn<(&'static str,), (&'static str,)>;
                    fn scoped(&self) -> Self::Scoped {
                        ScopedColumn {
                            table: (stringify!($pascal_case),),
                            col: (stringify!($member),),
                        }
                    }
                    type ScopedAliased = AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, &'static str)>;
                    fn scoped_aliased(&self, alias: &'static str) -> Self::ScopedAliased {
                        AliasedScopedColumn {
                            table: (stringify!($pascal_case),),
                            column: (stringify!($member),),
                            alias: (alias, stringify!($member)),
                        }
                    }
                    type NumScopedAliased = AliasedScopedColumn<(&'static str,), (&'static str,), (&'static str, usize, &'static str)>;
                    fn num_scoped_aliased(&self, num: usize, alias: &'static str) -> Self::NumScopedAliased {
                        AliasedScopedColumn {
                            table: (stringify!($pascal_case),),
                            column: (stringify!($member),),
                            alias: (alias, num, stringify!($member)),
                        }
                    }
                }
            )*

            $(
                const _: () = {
                    use crate::from_row::{FromRowData, FromRowAlias, FromRowError, RowPreAliased, RowPostAliased, RowTwoAliased};
                    use sqlx::{Row, Type, Decode, ColumnIndex};

                    impl FromRowData for $member {
                        type RData = $type;
                    }

                    impl<'r, R> FromRowAlias<'r, R> for $member
                    where
                        R: Row + 'r,
                        $type: Type<R::Database> + Decode<'r, R::Database>,
                        for<'q> &'q str: ColumnIndex<R>,
                    {
                        fn no_alias(&self, row: &'r R) -> Result<Self::RData, FromRowError> {
                            Ok(row.try_get(stringify!($member))?)
                        }
                        fn pre_alias(&self, row: RowPreAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                            Ok(row.try_get(stringify!($member))?)
                        }
                        fn post_alias(&self, row: RowPostAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                            Ok(row.try_get(stringify!($member))?)
                        }
                        fn two_alias(&self, row: RowTwoAliased<'r, R>) -> Result<Self::RData, FromRowError> {
                            Ok(row.try_get(stringify!($member))?)
                        }
                    }
                };
            )*
        }


    }};
}

define_collection!(
    struct Todo 3 {
        title: String,
        done: bool,
        description: Option<String>,
    }
);

define_collection!(
    struct Category 1 {
        title: String,
    }
);

define_collection!(
    struct Tag 1 {
        title: String,
    }
);

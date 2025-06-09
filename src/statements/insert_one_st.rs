use sqlx::{Arguments, Database, Encode, Type};

pub struct InsertOneSt<S: Database> {
    pub(crate) input: Vec<String>,
    pub(crate) returning: Option<Vec<String>>,
    pub(crate) table_name: String,
    pub(crate) buffer: S::Arguments<'static>,
}

impl<S: Database> InsertOneSt<S> {
    pub fn init(table_name: String) -> Self {
        Self {
            table_name,
            returning: Default::default(),
            buffer: Default::default(),
            input: Default::default(),
        }
    }
}

impl<S: Database> InsertOneSt<S> {
    pub fn col<T>(&mut self, col_name: String, ty: T)
    where
        T: Type<S> + Encode<'static, S> + 'static,
    {
        self.input.push(col_name);
        self.buffer.add(ty).unwrap();
    }
    pub fn returning(mut self, cols: Vec<String>) -> Self {
        self.returning = Some(cols);
        Self { ..self }
    }
}

impl<S: Database> crate::execute::Execute<S> for InsertOneSt<S> {
    fn build(self) -> (String, S::Arguments<'static>) {
        let column_num = self.input.len();
        let Self {
            input,
            returning,
            table_name,
            buffer,
        } = self;
        (
            format!(
                "INSERT INTO {table_name} ({input}) VALUES ({placements}) {returning};",
                input = input.join(", "),
                placements = {
                    let mut binds = 1;
                    let mut s_inner = Vec::new();
                    for _ in 0..column_num {
                        s_inner.push(format!("${}", binds));
                        binds += 1;
                    }

                    s_inner.join(", ")
                },
                returning = {
                    match returning {
                        Some(returning) => format!("RETURNING {}", returning.join(", ")),
                        None => "".to_string(),
                    }
                }
            ),
            buffer,
        )
    }
}

// pub struct InsertManySt<S: Database> {
//     pub(crate) input: Vec<String>,
//     pub(crate) returning: Option<Vec<String>>,
//     pub(crate) table_name: String,
//     pub(crate) col_count: usize,
//     pub(crate) buffer: S::Arguments<'static>,
// }
//
// impl<S: Database> InsertManySt<S> {
//     pub fn init(table_name: String, cols: Vec<String>) -> Self {
//         Self {
//             table_name,
//             col_count: cols.len(),
//             returning: Default::default(),
//             buffer: Default::default(),
//
//             input: Default::default(),
//         }
//     }
// }

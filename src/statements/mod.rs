use crate::QueryBuilder;

pub mod select_st;
pub mod update_st;
pub mod delete_st;
pub mod insert_one_st;
pub mod create_table_st;

    pub fn build_where<S: QueryBuilder>(
        clause: Vec<S::Fragment>,
        ctx2: &mut S::Context2,
        str: &mut String,
    ) {
        let mut where_str = Vec::default();
        for item in clause {
            let item = S::build_sql_part_back(ctx2, item);
            if item.is_empty() {
                continue;
            }

            where_str.push(item);
        }
        for (index, where_item) in where_str.into_iter().enumerate() {
            if index == 0 {
                str.push_str(" WHERE ");
            } else {
                str.push_str(" AND ");
            }
            str.push_str(&where_item);
        }
    }
